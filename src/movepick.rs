use std::cmp;
use std::default;

use ordered_float::OrderedFloat;

use crate::Action;
use crate::Bitboard;
use crate::bitboard::GameState;
use crate::bitboard::Color;

// give this its own module with abstration later
type Evaluator = Box<dyn Fn(&Bitboard) -> f32>;

// this evaluator is the simplist one and only temporary
fn simple_eval(board: &Bitboard) -> f32 {
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

    count_ones(board.blacks()) + count_ones(board.whites()) +
        count_ones(black_kings) - count_ones(white_kings)
}

static MAX_DEPTH: u32 = 25;
static MAX_TIME: u32 = 300;

pub struct MovePicker {
    // can attach evaluation functions and parameters in here later when we have more
    evaluator: Evaluator,
}

impl default::Default for MovePicker {
    fn default() -> MovePicker {
        MovePicker { evaluator: Box::new(simple_eval) }
    }
}

impl MovePicker {
    pub fn pick(&self, board: &Bitboard, constraint: &PickContraint) -> Option<Action> {
        // will only do the depth one now...
        // only going to be single threaded at first
        // no tranposition tables yet..

        // need to ensure that there is a valid action

        board.generate_all_actions()
            .iter()
            .map(|a| a.0)  // get rid of the boards
            .max_by_key(|a| OrderedFloat(self.evaluate_action(&board, &a, &constraint)))
    }

    #[inline]
    pub fn evaluate_action(&self, board: &Bitboard, action: &Action, constraint: &PickContraint) -> f32 {
        let board_p = board.take_action(&action).unwrap();
        self.evaluate_board(&board_p, &constraint)
    }

    // get list of top 5 moves and ratings
    #[inline]
    pub fn evaluate_board(&self, board: &Bitboard, _constraint: &PickContraint) -> f32 {
        // name of this might be a bit confusing
        self.minmax_helper(&board, 5, f32::NEG_INFINITY, f32::INFINITY)
        
    }

    fn minmax_helper(&self, board: &Bitboard, depth: u32, mut alpha: f32, mut beta: f32) -> f32 {
        if let GameState::Completed(_) = board.get_game_state() {
            return (self.evaluator)(&board);
        }
        // ideally merge this above when we figure out why it wont work
        if depth == 0 {
            return (self.evaluator)(&board);
        }

        match board.turn() {
            Color::Black => {
                let mut max_eval = f32::NEG_INFINITY;

                // isolate only the next board...
                for board_p in board.generate_all_actions().iter().map(|a| a.1) {
                    let eval = self.minmax_helper(&board_p, depth - 1, alpha, beta);
                    max_eval = *cmp::max(OrderedFloat(max_eval), OrderedFloat(eval));
                    alpha = *cmp::max(OrderedFloat(alpha), OrderedFloat(eval));
                    if beta <= alpha {
                        break;
                    }
                }

                max_eval
            },

            Color::White => {
                let mut min_eval = f32::INFINITY;

                for board_p in board.generate_all_actions().iter().map(|a| a.1) {
                    let eval = self.minmax_helper(&board_p, depth - 1, alpha, beta);
                    min_eval = *cmp::min(OrderedFloat(min_eval), OrderedFloat(eval));
                    beta = *cmp::min(OrderedFloat(beta), OrderedFloat(eval));
                    if beta <= alpha {
                        break
                    }
                }

                min_eval
            },
        }
    }
}

pub enum PickContraint {
    Depth(u32),
    Time(u32),
    None,
}

impl PickContraint {
    pub fn depth(d: u32) -> Result<PickContraint, &'static str> {
        if d > MAX_DEPTH {
            return Err("Depth too large! Pick lower than 25");
        }
        Ok(PickContraint::Depth(d))
    }

    pub fn time(t: u32) -> Result<PickContraint, &'static str> {
        if t > MAX_TIME {
            return Err("Time to large! Pick lower than 300 seconds")
        }
        Ok(PickContraint::Time(t))
    }

    pub fn none() -> PickContraint {
        PickContraint::None
    }
}
