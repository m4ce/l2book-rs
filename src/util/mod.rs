use std::cmp::Ordering;

pub const fn bids_comparator(lhs: i64, rhs: i64) -> Ordering {
    if lhs > rhs {
        Ordering::Greater
    } else if lhs < rhs {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

pub const fn asks_comparator(lhs: i64, rhs: i64) -> Ordering {
    if lhs > rhs {
        Ordering::Less
    } else if lhs < rhs {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}
