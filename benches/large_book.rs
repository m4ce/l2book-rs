use criterion::{black_box, criterion_group, criterion_main, Criterion};
use l2book_rs::ladder::{Book, DefaultBookEventListener};

fn large_book(c: &mut Criterion) {
    let event_listener = DefaultBookEventListener {};
    let mut book = Book::new(&event_listener);
    book.begin();

    c.bench_function("large book", |b| {
        b.iter(|| {
            book.update_bid(black_box(10), black_box(99), 0);
        })
    });

    book.end();
}

criterion_group!(benches, large_book);
criterion_main!(benches);
