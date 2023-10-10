use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use l2book_rs::ladder::{Book, DefaultBookEventListener};
use simplerand::rand_range;

const DEFAULT_WORST_BID: i64 = 1000;
const DEFAULT_BEST_BID: i64 = 2000;
const DEFAULT_WORST_ASK: i64 = 4000;
const DEFAULT_BEST_ASK: i64 = 3000;
const DEFAULT_MIN_QTY: i64 = 0;
const DEFAULT_MAX_QTY: i64 = 10;
const TICK_SIZE: usize = 1;

fn tob_benchmark(c: &mut Criterion) {
    c.bench_function("update best bid", |b| {
        let listener = DefaultBookEventListener::default();
        let mut book = Book::new(&listener);
        let mut px: i64 = DEFAULT_BEST_BID + TICK_SIZE as i64;
        b.iter(|| {
            book.begin();
            book.update_bid(px, 1, 1);
            book.end();
            px += TICK_SIZE as i64;
        });
    });

    c.bench_function("update best ask", |b| {
        let listener = DefaultBookEventListener::default();
        let mut book = Book::new(&listener);
        let mut px: i64 = DEFAULT_BEST_ASK - TICK_SIZE as i64;
        b.iter(|| {
            book.begin();
            book.update_ask(px, 1, 1);
            book.end();
            px -= TICK_SIZE as i64;
        });
    });
}

fn random_benchmark(c: &mut Criterion) {
    let levels = [1024, 8192, 65536];
    for size in levels {
        c.bench_with_input(
            BenchmarkId::new("random update bid", size),
            &size,
            |b, &size| {
                let listener = DefaultBookEventListener::default();
                let mut book = Book::new(&listener);
                let bids: Vec<(i64, i64)> = (0..size)
                    .map(|_| {
                        (
                            rand_range::<i64>(DEFAULT_WORST_BID, DEFAULT_BEST_BID),
                            rand_range::<i64>(DEFAULT_MIN_QTY, DEFAULT_MAX_QTY),
                        )
                    })
                    .collect();
                let mut i = 0;
                b.iter(|| {
                    book.begin();
                    let index = i & (size - 1);
                    book.update_bid(bids[index].0, bids[index].1, 1);
                    i += 1;
                    book.end();
                });
            },
        );

        c.bench_with_input(
            BenchmarkId::new("random update ask", size),
            &size,
            |b, &size| {
                let listener = DefaultBookEventListener::default();
                let mut book = Book::new(&listener);
                let asks: Vec<(i64, i64)> = (0..size)
                    .map(|_| {
                        (
                            rand_range::<i64>(DEFAULT_BEST_ASK, DEFAULT_WORST_ASK),
                            rand_range::<i64>(DEFAULT_MIN_QTY, DEFAULT_MAX_QTY),
                        )
                    })
                    .collect();
                let mut i = 0;
                b.iter(|| {
                    book.begin();
                    let index = i & (size - 1);
                    book.update_ask(asks[index].0, asks[index].1, 1);
                    i += 1;
                    book.end();
                });
            },
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = tob_benchmark, random_benchmark
}
criterion_main!(benches);
