use crible_core::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("score_hand", |b| {
        let cards = parse_cards("Th Qh Jh 5h").unwrap();
        let hand = Hand::from_slice(&cards).unwrap();
        b.iter(|| hand.score(black_box(Card::new(Number::T, Suit::S)), true));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
