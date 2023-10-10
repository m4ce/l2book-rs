use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use l2book_rs::ladder::{Book, DefaultBookEventListener};
use simplerand::{rand_range, Randomable};

const DEFAULT_WORST_BID: i64 = 1000;
const DEFAULT_BEST_BID: i64 = 2000;
const DEFAULT_WORST_ASK: i64 = 4000;
const DEFAULT_BEST_ASK: i64 = 3000;
const TICK_SIZE: usize = 1;

fn tob_benchmark(c: &mut Criterion) {
    let listener = DefaultBookEventListener::default();
    let mut book = Book::new(&listener);

    c.bench_function("best bid", |b| {
        book.clear();
        let mut px: i64 = DEFAULT_BEST_BID + TICK_SIZE as i64;
        b.iter(|| {
            book.begin();
            book.update_bid(px, 1, 1);
            book.end();
            px += TICK_SIZE as i64;
        });
    });

    c.bench_function("best ask", |b| {
        book.clear();
        let mut px: i64 = DEFAULT_BEST_ASK - TICK_SIZE as i64;
        b.iter(|| {
            book.begin();
            book.update_ask(px, 1, 1);
            book.end();
            px -= TICK_SIZE as i64;
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = tob_benchmark
}
criterion_main!(benches);
