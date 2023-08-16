use log::{info};
use std::cmp::Ordering;
use derivative::Derivative;

use crate::util::{bids_comparator, asks_comparator};
use crate::common::{Decimal64, Side};

pub trait LadderEventListener {
    fn on_add(&self, side: Side, level: &PriceLevel);

    fn on_update(&self, side: Side, old_level: &PriceLevel, new_level: &PriceLevel);

    fn on_remove(&self, side: Side, level: &PriceLevel);
}

#[derive(Debug, Copy, Clone)]
pub struct PriceLevel {
    price: Decimal64,
    qty: Decimal64,
    no_of_orders: u16,
}

impl PartialEq<Self> for PriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.qty == other.qty && self.no_of_orders == other.no_of_orders
    }
}

impl Eq for PriceLevel {}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Ladder<'comparator, 'listener> {
    side: Side,
    in_batch: bool,
    #[derivative(Debug = "ignore")]
    event_listener: &'listener dyn LadderEventListener,
    levels: Vec<PriceLevel>,
    #[derivative(Debug = "ignore")]
    comparator: &'comparator dyn Fn(i64, i64) -> Ordering,
}

impl Ladder<'_, '_> {
    pub fn iter(&self) -> impl Iterator<Item=&PriceLevel> {
        self.levels.iter().rev()
    }

    pub fn count(&self) -> usize {
        self.levels.len()
    }

    pub fn best(&self) -> Option<&PriceLevel> {
        if self.levels.len() > 0 {
            return self.levels.last();
        }
        None
    }

    pub fn worst(&self) -> Option<&PriceLevel> {
        if self.levels.len() > 0 {
            return self.levels.first();
        }
        None
    }

    pub fn get(&self, idx: usize) -> Option<&PriceLevel> {
        // Ladders are reversed, so our offset is the tail
        return self.levels.get(self.levels.len() - 1 - idx);
    }

    fn ensure_in_batch(&self) {
        if !self.in_batch {
            panic!("unable to update the ladder until begin is called")
        }
    }

    fn begin(&mut self) {
        if self.in_batch {
            panic!("begin has already been called");
        }
        self.in_batch = true;
    }

    fn end(&mut self) {
        if !self.in_batch {
            panic!("begin has not been called");
        }
        self.in_batch = false;
    }

    #[cfg(not(search_policy = "linear_search_policy"))]
    fn find(&mut self, price: Decimal64) -> Result<usize, usize> {
        self
            .levels
            .binary_search_by(|level| (self.comparator)(price, level.price).reverse())
    }

    #[cfg(search_policy = "linear_search_policy")]
    fn find(&mut self, price: Decimal64) -> Result<usize, usize> {
        if self.levels.is_empty() {
            return Err(0);
        }
        let mut i: i32 = (self.levels.len() as i32) - 1;
        while i >= 0 {
            let level = self.levels.get(i as usize);
            let cmp = (self.comparator)(price, level.unwrap().price).reverse();
            if cmp == Ordering::Equal {
                return Ok(i as usize);
            } else if cmp == Ordering::Greater {
                i -= 1;
            } else {
                return Err((i + 1) as usize);
            }
        }
        Err(0)
    }

    fn add_or_update(&mut self, price: Decimal64, qty: Decimal64, no_of_orders: u16) {
        self.ensure_in_batch();
        // Most updates would happen at the top of the book, so it makes sense to reverse the ladders.
        match self.find(price) {
            Ok(pos) => {
                if qty == 0 {
                    // remove the level
                    let level = &self.levels[pos];
                    self.event_listener.on_remove(self.side, level);
                    self.levels.remove(pos);
                } else {
                    // copy the existing level and update the qty
                    if self.levels[pos].qty != qty {
                        let old_level = self.levels[pos]; // copy, should be cheap
                        self.levels[pos].qty = qty; // update the qty
                        self.levels[pos].no_of_orders = no_of_orders; // update the orders
                        self.event_listener.on_update(self.side, &old_level, &self.levels[pos]);
                    }
                }
            }
            Err(pos) => {
                if qty > 0 {
                    let level = PriceLevel {
                        price: price,
                        qty: qty,
                        no_of_orders: no_of_orders,
                    };
                    self.levels.insert(pos, level);
                    self.event_listener.on_add(self.side, &level);
                }
            }
        }
    }

    fn clear(&mut self) {
        self.levels.clear();
    }

    fn remove_levels_before(&mut self, price: Decimal64) {
        self.ensure_in_batch();
        let pos = match self.find(price) {
            Ok(pos) => pos,
            Err(pos) => pos
        };
        for level in &self.levels[pos..] {
            self.event_listener.on_remove(self.side, level);
        }
        self.levels.truncate(pos); // ladders are reversed, so we can simply truncate
    }

    fn apply_trade(&mut self, price: Decimal64, qty: Decimal64) {
        self.ensure_in_batch();
        let result = self.find(price);
        let pos = match result {
            Ok(pos) => pos,
            Err(pos) => pos
        };

        let mut offset: usize;
        if result.is_ok() {
            // we found the price level
            offset = (pos + 1).max(self.levels.len() - 1);
            for level in &self.levels[offset..] {
                self.event_listener.on_remove(self.side, level);
            }
            if qty >= self.levels[pos].qty {
                // remove the level
                self.event_listener.on_remove(self.side, &self.levels[pos]);
                offset -= 1;
            } else {
                // copy the existing level and update the qty
                let old_level = self.levels[pos]; // copy, should be cheap
                self.levels[pos].qty = qty; // update the qty
                self.event_listener.on_update(self.side, &old_level, &self.levels[pos]);
            }
        } else {
            // we did not find the price level ...
            offset = pos;
            for level in &self.levels[offset..] {
                self.event_listener.on_remove(self.side, level);
            }
        }

        self.levels.truncate(offset);
    }
}

pub struct Book<'comparator, 'listener> {
    bids: Ladder<'comparator, 'listener>,
    asks: Ladder<'comparator, 'listener>,
}

impl<'comparator, 'listener> Book<'comparator, 'listener> {
    pub fn new(event_listener: &'listener dyn LadderEventListener) -> Self {
        Book {
            bids: Ladder {
                side: Side::BUY,
                levels: vec![],
                comparator: &bids_comparator,
                in_batch: false,
                event_listener: event_listener,
            },
            asks: Ladder {
                side: Side::SELL,
                levels: vec![],
                comparator: &asks_comparator,
                in_batch: false,
                event_listener: event_listener,
            },
        }
    }

    pub fn begin(&mut self) {
        self.bids.begin();
        self.asks.begin();
    }

    pub fn end(&mut self) {
        self.bids.end();
        self.asks.end();
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }

    pub fn remove_levels_before_ask(&mut self, price: Decimal64) {
        self.asks.remove_levels_before(price);
    }

    pub fn remove_levels_before_bid(&mut self, price: Decimal64) {
        self.bids.remove_levels_before(price);
    }

    pub fn update_ask(&mut self, price: Decimal64, qty: Decimal64, no_of_orders: u16) {
        self.asks.add_or_update(price, qty, no_of_orders);
    }

    pub fn update_bid(&mut self, price: Decimal64, qty: Decimal64, no_of_orders: u16) {
        self.bids.add_or_update(price, qty, no_of_orders);
    }

    pub fn apply_trade(&mut self, price: Decimal64, qty: Decimal64, is_buyer_mm: bool) {
        if is_buyer_mm {
            self.bids.apply_trade(price, qty);
        } else {
            self.asks.apply_trade(price, qty);
        }
    }
}

struct DefaultBookEventListener {}

impl LadderEventListener for DefaultBookEventListener {
    fn on_add(&self, side: Side, level: &PriceLevel) {
        info!("Added new {:?} level: {:?}", side, level);
    }

    fn on_update(&self, side: Side, old_level: &PriceLevel, new_level: &PriceLevel) {
        info!("Updated existing {:?} level: {:?} -> {:?}", side, old_level, new_level);
    }

    fn on_remove(&self, side: Side, level: &PriceLevel) {
        info!("Removed {:?} level: {:?}", side, level);
    }
}

#[cfg(test)]
mod test {
    use crate::ladder::{Book, PriceLevel};
    use crate::ladder::DefaultBookEventListener;

    #[test]
    fn test_initialize_book() {
        let event_listener = DefaultBookEventListener {};
        let book = Book::new(&event_listener);
        assert_eq!(0, book.asks.count());
        assert_eq!(0, book.bids.count());
        assert_eq!(true, book.asks.best().is_none());
        assert_eq!(true, book.asks.worst().is_none());
        assert_eq!(true, book.bids.best().is_none());
        assert_eq!(true, book.bids.worst().is_none());
    }

    #[test]
    fn test_add_bids() {
        let event_listener = DefaultBookEventListener {};
        let mut book = Book::new(&event_listener);
        book.begin();
        book.update_bid(7, 99, 0);
        book.update_bid(8, 100, 0);
        book.update_bid(6, 98, 0);
        book.update_bid(9, 101, 0);
        book.end();

        // verify len
        assert_eq!(4, book.bids.count());

        // verify best bid, worst bid
        assert_eq!(PriceLevel { price: 9, qty: 101, no_of_orders: 0 }, *book.bids.best().unwrap());
        assert_eq!(PriceLevel { price: 6, qty: 98, no_of_orders: 0 }, *book.bids.worst().unwrap());

        // verify levels
        assert_eq!(PriceLevel { price: 9, qty: 101, no_of_orders: 0 }, *book.bids.get(0).unwrap());
        assert_eq!(PriceLevel { price: 8, qty: 100, no_of_orders: 0 }, *book.bids.get(1).unwrap());
        assert_eq!(PriceLevel { price: 7, qty: 99, no_of_orders: 0 }, *book.bids.get(2).unwrap());
        assert_eq!(PriceLevel { price: 6, qty: 98, no_of_orders: 0 }, *book.bids.get(3).unwrap());

        // perform some updates
        book.begin();
        book.update_bid(7, 0, 0);
        book.update_bid(9, 200, 0);
        book.end();

        // verify len
        assert_eq!(3, book.bids.count());
        assert_eq!(PriceLevel { price: 9, qty: 200, no_of_orders: 0 }, *book.bids.best().unwrap());
    }

    #[test]
    fn test_add_asks() {
        let event_listener = DefaultBookEventListener {};
        let mut book = Book::new(&event_listener);
        book.begin();
        book.update_ask(13, 99, 0);
        book.update_ask(11, 100, 0);
        book.update_ask(12, 98, 0);
        book.update_ask(10, 101, 0);
        book.end();

        // verify len
        assert_eq!(4, book.asks.count());

        // verify best ask, worst ask
        assert_eq!(PriceLevel { price: 10, qty: 101, no_of_orders: 0 }, *book.asks.best().unwrap());
        assert_eq!(PriceLevel { price: 13, qty: 99, no_of_orders: 0 }, *book.asks.worst().unwrap());

        // verify levels
        assert_eq!(PriceLevel { price: 10, qty: 101, no_of_orders: 0 }, *book.asks.get(0).unwrap());
        assert_eq!(PriceLevel { price: 11, qty: 100, no_of_orders: 0 }, *book.asks.get(1).unwrap());
        assert_eq!(PriceLevel { price: 12, qty: 98, no_of_orders: 0 }, *book.asks.get(2).unwrap());
        assert_eq!(PriceLevel { price: 13, qty: 99, no_of_orders: 0 }, *book.asks.get(3).unwrap());

        // perform some updates
        book.begin();
        book.update_ask(12, 0, 0);
        book.update_ask(10, 200, 0);
        book.end();

        // verify len
        assert_eq!(3, book.asks.count());
        assert_eq!(PriceLevel { price: 10, qty: 200, no_of_orders: 0 }, *book.asks.best().unwrap());
    }
}