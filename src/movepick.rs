use std::cmp;
use std::default;
use std::sync::Arc;
use std::time::Duration;
#[cfg(target_family = "unix")]
use {
    std::thread,
    std::sync::mpsc,
    std::os::unix::thread::JoinHandleExt,
};

use ordered_float::OrderedFloat;

use crate::board::{Bitboard, Action, GameState, Color};
use crate::search::TranspositionTable;

// give this its own module with abstration later
type Evaluator = Arc<dyn Fn(&Bitboard) -> f32 + Send + Sync>;

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

#[derive(Clone)]
pub struct MovePicker {
    evaluator: Evaluator,
    tt: Arc<TranspositionTable>,
}

impl default::Default for MovePicker {
    fn default() -> Self {
        let evaluator = Arc::new(simple_eval);
        let tt = Arc::new(TranspositionTable::new());

        MovePicker { evaluator, tt }
    }
}

impl MovePicker {
    pub fn pick(&self, board: &Bitboard, constraint: &PickConstraint) -> Option<Action> {
        // will only do the depth one now...
        // only going to be single threaded at first

        let me = self.clone();
        let board = board.clone();

        let compute_at_depth = move |d| {
            board.generate_all_actions()
                .iter()
                .map(|a| a.0)  // get rid of the boards
                .max_by_key(|a| OrderedFloat(me.evaluate_action(&board, &a, &PickConstraint::Depth(d))))
        };

        match constraint {
            PickConstraint::None => compute_at_depth(6),
            PickConstraint::Depth(d) => compute_at_depth(*d),
            PickConstraint::Time(t) => Self::iddfs_helper(compute_at_depth, &Duration::from_secs((*t).into())),
        }
    }

    #[inline]
    pub fn evaluate_action(&self, board: &Bitboard, action: &Action, constraint: &PickConstraint) -> f32 {
        let board_p = board.take_action(&action).unwrap();
        self.evaluate_board(&board_p, &constraint)
    }

    // get list of top 5 moves and ratings

    #[inline]
    pub fn evaluate_board(&self, board: &Bitboard, constraint: &PickConstraint) -> f32 {
        let me = self.clone();
        let board = board.clone();

        let compute_at_depth = move |d| me.minmax_helper(&board, d, f32::NEG_INFINITY, f32::INFINITY);

        match constraint {
            // have iterative deepening for None as well..
            PickConstraint::None => compute_at_depth(6),
            PickConstraint::Depth(d) => compute_at_depth(*d),
            PickConstraint::Time(t) => Self::iddfs_helper(compute_at_depth, &Duration::from_secs((*t).into())),
        }
    }

    fn minmax_helper(&self, board: &Bitboard, depth: u32, mut alpha: f32, mut beta: f32) -> f32 {
        if let Some(value) = self.tt.get(&board, depth) {
            return value;
        }

        if let GameState::Completed(_) = board.get_game_state() {
            return (self.evaluator)(&board);
        }
        // ideally merge this above when we figure out why it wont work
        if depth == 0 {
            return (self.evaluator)(&board);
        }

        let eval = match board.turn() {
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
        };

        self.tt.insert(&board, depth, eval);

        eval
    }

    #[cfg(target_family = "unix")]
    fn iddfs_helper<T, F>(f: F, duration: &Duration) -> T
    where
        T: 'static + Send,
        F: Fn(u32) -> T + 'static + Send + Sync,
    {
        // not super enthused with the implementation of this feature
        // i dont like how the closure are set up in each of the functions above to make this work
        // seems like unneccesary overhead imo..
        let (tx, rx) = mpsc::channel();

        let thread_id = thread::spawn(move || {
            for d in 1.. {
                let eval = f(d);
                // send result
                tx.send(eval).unwrap();
            }
        }).into_pthread_t();

        thread::sleep(*duration);

        unsafe { libc::pthread_cancel(thread_id) };

        // get the most recent move suggested by the engine
        // will only panic if iddps didnt find a result (almost impossible)
        rx.try_iter().last().unwrap()
    }

    #[cfg(target_family = "windows")]
    fn iddfs_helper<F>(_: F, _: &Duration) -> ! {
        panic!("Cannot run iterative deepening depth first search on Windows yet!");
    }
}

pub enum PickConstraint {
    Depth(u32),
    Time(u32),
    None,
}

impl PickConstraint {
    pub fn depth(d: u32) -> Result<Self, &'static str> {
        if d > MAX_DEPTH {
            return Err("Depth too large! Pick lower than 25");
        }
        Ok(PickConstraint::Depth(d))
    }

    pub fn time(t: u32) -> Result<Self, &'static str> {
        if t > MAX_TIME {
            return Err("Time to large! Pick lower than 300 seconds")
        }
        Ok(PickConstraint::Time(t))
    }

    pub fn none() -> Self {
        PickConstraint::None
    }
}
