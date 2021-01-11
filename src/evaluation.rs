use std::default;
use std::ops::Fn;

use lazy_static::lazy_static;

use crate::board::Bitboard;
use crate::search::Score;

// honestly not 100% sure what to do with this module as there are many different approaches
// each with their own benefits. when i get a better sense with what i want out of this module
// (with regard to NNUE and different evaluators) ill come to a good, more permanent idea

lazy_static! {
    pub static ref GLOBAL_EVAL: BoardEvaluator = BoardEvaluator::default();
}

#[allow(dead_code)]
pub enum BoardEvaluator {
    Classical(Box<dyn Fn(&Bitboard) -> Score + Send + Sync>),
    Nnue,
}
use BoardEvaluator::*;

impl BoardEvaluator {
    #[inline]
    pub fn eval(&self, board: &Bitboard) -> Score {
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
                let mut count = Score::from(0.);
                while mask != 0 {
                    if mask & 1 == 1 {
                        count += Score::from(1.);
                    }
                    mask >>= 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_BOARD: &'static str =
        "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12";
    const TEST_BOARD_1: &'static str = "B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29";
    const TEST_BOARD_2: &'static str = "W:W9,K11,19,K26,27,30:B15,22,25,K32";
    const TEST_BOARD_3: &'static str = "B:WK3,11,23,25,26,27:B6,7,8,18,19,21,K31";

    #[test]
    fn default_evaluator_test() {
        use crate::search::{Score, Searchable};

        let board = Bitboard::from_fen(DEFAULT_BOARD).unwrap();
        assert_eq!(board.evaluate(), Score::from(0.));

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.evaluate(), Score::from(1.));

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.evaluate(), Score::from(-3.));

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.evaluate(), Score::from(1.));
    }
}
