use std::cmp::Ordering;

pub const fn bids_comparator(lhs: i64, rhs: i64) -> Ordering {
    if lhs > rhs {
        return Ordering::Greater;
    } else if lhs < rhs {
        return Ordering::Less;
    } else {
        return Ordering::Equal;
    }
}

pub const fn asks_comparator(lhs: i64, rhs: i64) -> Ordering {
    if lhs > rhs {
        return Ordering::Less;
    } else if lhs < rhs {
        return Ordering::Greater;
    } else {
        return Ordering::Equal;
    }
}