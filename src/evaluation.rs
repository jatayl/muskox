use std::default;
use std::sync::Arc;
use std::ops::Fn;

use crate::board::Bitboard;
use crate::search::Evaluator;

#[allow(dead_code)]
#[derive(Clone)]
pub enum BoardEvaluator {
    Classical(Arc<dyn Fn(&Bitboard) -> f32 + Send +Sync>),
    Nnue,
}
use BoardEvaluator::*;

impl Evaluator<Bitboard> for BoardEvaluator {
    #[inline]
    fn eval(&self, board: &Bitboard) -> f32 {
        match self {
            Classical(f) => f(&board),
            Nnue => panic!("Cannot use NNUE evaluation yet!"),
        }
    }
}

impl default::Default for BoardEvaluator {
    fn default() -> Self {
        Classical(Arc::new(|board: &Bitboard| {
            // need the evaluation for the finished game.,..
            // reaccess this as mask
            let count_ones = |mut mask: u32| {
                let mut count = 0.;
                while mask != 0 {
                    if mask & 1 == 1 {
                        count += 1.;
                    }
                    mask = mask >> 1;
                }
                count
            };

            let black_kings = board.blacks() & board.kings();
            let white_kings = board.whites() & board.kings();

            count_ones(board.blacks()) - count_ones(board.whites()) +
                count_ones(black_kings) - count_ones(white_kings)
        }))
    }
}