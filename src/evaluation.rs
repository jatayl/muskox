use std::default;
use std::ops::Fn;

use lazy_static::lazy_static;
use ordered_float::OrderedFloat;

use crate::board::Bitboard;

// honestly not 100% sure what to do with this module as there are many different approaches
// each with their own benefits. when i get a better sense with what i want out of this module
// (with regard to NNUE and different evaluators) ill come to a good, more permanent idea

lazy_static! {
    pub static ref GLOBAL_EVAL: BoardEvaluator = BoardEvaluator::default();
}

#[allow(dead_code)]
pub enum BoardEvaluator {
    Classical(Box<dyn Fn(&Bitboard) -> OrderedFloat<f32> + Send + Sync>),
    Nnue,
}
use BoardEvaluator::*;

impl BoardEvaluator {
    #[inline]
    pub fn eval(&self, board: &Bitboard) -> OrderedFloat<f32> {
        match self {
            Classical(f) => f(&board),
            Nnue => panic!("Cannot use NNUE evaluation yet!"),
        }
    }
}

impl default::Default for BoardEvaluator {
    fn default() -> Self {
        Classical(Box::new(|board: &Bitboard| {
            // need the evaluation for the finished game.,..
            // reaccess this as mask
            let count_ones = |mut mask: u32| {
                let mut count = OrderedFloat(0.);
                while mask != 0 {
                    if mask & 1 == 1 {
                        count = count + OrderedFloat(1.);
                    }
                    mask = mask >> 1;
                }
                count
            };

            let black_kings = board.blacks() & board.kings();
            let white_kings = board.whites() & board.kings();

            count_ones(board.blacks()) - count_ones(board.whites()) + count_ones(black_kings)
                - count_ones(white_kings)
        }))
    }
}
