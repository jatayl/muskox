use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use muskox::board::{Bitboard, BoardEvaluator};
use muskox::search::{Engine, SearchConstraint, Searchable};

static BOARDS_FENS: [&'static str; 4] = [
    "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12",
    "B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29",
    "W:W9,K11,19,K26,27,30:B15,22,25,K32",
    "B:WK11,3:B",
];

pub fn movepick_benchmarker(c: &mut Criterion) {
    let evaluator = Arc::new(BoardEvaluator::default());
    let engine = Engine::new(evaluator);
    let constraint = SearchConstraint::none();

    let mut group = c.benchmark_group("engine");
    for (i, board) in BOARDS_FENS.iter().map(|s| Bitboard::new_from_fen(s).unwrap()).enumerate() {
        group.bench_with_input(i.to_string(), &board, |b, &board| {
            b.iter(|| engine.search(&board, &constraint));
        });
    }
}

pub fn generate_benchmarker(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate all moves");
    for (i, board) in BOARDS_FENS.iter().map(|s| Bitboard::new_from_fen(s).unwrap()).enumerate() {
        group.bench_with_input(i.to_string(), &board, |b, &board| {
            b.iter(|| board.generate_all_actions());
        });
    }
}

criterion_group!(benches, movepick_benchmarker, generate_benchmarker);
criterion_main!(benches);