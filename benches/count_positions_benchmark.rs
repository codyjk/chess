use chess::board::color::Color;
use chess::board::Board;
use chess::moves::count_positions;
use chess::moves::targets::Targets;

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("count all possible positions to depth 4", |b| {
        b.iter(|| {
            count_positions(
                4,
                &mut Board::starting_position(),
                &mut Targets::new(),
                Color::White,
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
