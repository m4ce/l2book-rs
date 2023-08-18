use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use simplerand::{Randomable, rand_range};
use l2book_rs::ladder::{Book, DefaultBookEventListener};

const MIN_BID: i64 = 1;
const MAX_BID: i64 = 1000;
const MIN_ASK: i64 = MAX_BID;
const MAX_ASK: i64 = 2000;

const LEVELS: usize = 1000;

fn random<T: Randomable>(min: T, max: T) -> T {
    rand_range::<T>(min, max)
}

fn large_book(c: &mut Criterion) {
    let bids: Vec<i64> = (0..LEVELS).map(|_| random::<i64>(MIN_BID, MAX_BID)).collect();
    let asks: Vec<i64> = (0..LEVELS).map(|_| random::<i64>(MIN_ASK, MAX_ASK)).collect();
    
    c.bench_function("large book", |b| {
        b.iter(|| {

            let listener = DefaultBookEventListener::default();
            let mut book = Book::new(&listener);
            book.begin();
            for i in 0..LEVELS {
                book.update_bid(bids[i], 10, 1);
                book.update_ask(asks[i], 10, 1);
            }
            book.end();
        })
    });

}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = large_book
}
criterion_main!(benches);
