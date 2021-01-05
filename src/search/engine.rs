use std::cmp;
use std::thread;
use std::sync::{mpsc, Arc};
use std::time::Duration;

use rayon::{ThreadPool, ThreadPoolBuilder};
use ordered_float::OrderedFloat;

use crate::search::{TranspositionTable, Searchable, Evaluator, Optim, Side, GameState};

const MAX_DEPTH: u32 = 25;
const MAX_TIME: u32 = 300000;
const NUM_THREADS: usize = 8;

#[derive(Clone)]
pub struct Engine<S: Searchable> {
    evaluator: Arc<dyn Evaluator<S>>,
    tt: TranspositionTable<S>,
    pool: Arc<ThreadPool>,
}

impl<S: Searchable> Engine<S> {
    pub fn new(evaluator: Arc<dyn Evaluator<S>>) -> Self {
        let tt = TranspositionTable::new(256);
        let pool = Arc::new(ThreadPoolBuilder::new().num_threads(NUM_THREADS - 1).build().unwrap());

        Engine { evaluator, tt, pool }
    }

    pub fn search(&mut self, state: &S, constraint: &SearchConstraint) -> Vec<ActionScorePair<S>> {
        self.tt.new_search();  // increment the generation

        let me = self.clone();
        let state = state.clone();

        // set the initial zobrist hash
        let zobrist_hash = state.zobrist_hash();  // this is relatively expensive function to call

        let compute_at_depth = move |d| {
            let action_states = state.generate_all_actions();
            let evals: Vec<_> = action_states.iter()
                .map(|p| me.minmax_helper(&p.state(), d, f32::NEG_INFINITY, f32::INFINITY, zobrist_hash ^ p.zobrist_diff()))
                .map(|f| OrderedFloat(f))
                .collect();
            let mut out: Vec<_> = action_states.iter()
                .map(|p| p.action())
                .zip(evals.iter())
                .collect();
            // sort based on the evaluations
            out.sort_by(|a, b| match state.turn().optim() {
                Optim::Min => a.1.cmp(b.1),
                Optim::Max => b.1.cmp(a.1),
            });
            out.iter()
                .map(|(&a, &s)| ActionScorePair {action: a, score: *s})  // copy all of the values and get rid of ordered float wrapper
                // .take(5) // only take the top fives moves.
                .collect()
        };

        match constraint {
            // have iterative deepening for None as well..
            SearchConstraint::None => compute_at_depth(13),
            SearchConstraint::Depth(dep) => compute_at_depth(*dep),
            SearchConstraint::Time(dur) => self.iddfs_helper(compute_at_depth, *dur, None),
        }
    }

    pub fn reset(&mut self) {
        self.tt.resize(256);
    }

    #[allow(dead_code, unused_variables)]
    fn shard_helper(&self, state: &S) -> ActionScorePair<S> {
        // this will break up a task into multiple shards that each thread in the threadpool can tackle
        // NOT IMPLEMENTED YET! :)

        // assert that depth is greater than 2

        let next_actions = state.generate_all_actions();

        ActionScorePair {
            action: *state.generate_all_actions()[0].action(),
            score: 0.
        }
    }

    fn minmax_helper(&self, state: &S, depth: u32, mut alpha: f32, mut beta: f32, zobrist_hash: u64) -> f32 {
        if let Some(value) = self.tt.probe(zobrist_hash, &state, depth as u8) {
            return value;
        }

        if (depth == 0) | (state.get_game_state() != GameState::InProgress) {
            return self.evaluator.eval(&state);
        }

        let eval = match state.turn().optim() {
            Optim::Max => {
                let mut max_eval = f32::NEG_INFINITY;

                for (state_p, zobrist_diff) in state.generate_all_actions().iter().map(|a| (a.state(), a.zobrist_diff())) {
                    let zobrist_hash_p = zobrist_hash ^ zobrist_diff;
                    let eval = self.minmax_helper(&state_p, depth - 1, alpha, beta, zobrist_hash_p);
                    max_eval = *cmp::max(OrderedFloat(max_eval), OrderedFloat(eval));
                    alpha = *cmp::max(OrderedFloat(alpha), OrderedFloat(max_eval));
                    if beta <= alpha {
                        break;
                    }
                }

                max_eval
            },

            Optim::Min => {
                let mut min_eval = f32::INFINITY;

                for (state_p, zobrist_diff) in state.generate_all_actions().iter().map(|a| (a.state(), a.zobrist_diff())) {
                    let zobrist_hash_p = zobrist_hash ^ zobrist_diff;
                    let eval = self.minmax_helper(&state_p, depth - 1, alpha, beta, zobrist_hash_p);
                    min_eval = *cmp::min(OrderedFloat(min_eval), OrderedFloat(eval));
                    beta = *cmp::min(OrderedFloat(beta), OrderedFloat(min_eval));
                    if beta <= alpha {
                        break
                    }
                }

                min_eval
            },
        };

        // POTENTIAL NEGAMAX IMPLEMENTATION
        // check out: https://stackoverflow.com/questions/41182117/modify-minimax-to-alpha-beta-pruning-pseudo-code
        // currently performance is problematically inconsistent

        // let mut max_eval = f32::NEG_INFINITY;

        // for (state_p, zobrist_diff) in state.generate_all_actions().iter().map(|a| (a.state(), a.zobrist_diff())) {
        //     let zobrist_hash_p = zobrist_hash ^ zobrist_diff;
        //     let eval = -self.minmax_helper(&state_p, depth - 1, -beta, -alpha, zobrist_hash_p);
        //     max_eval = *cmp::max(OrderedFloat(max_eval), OrderedFloat(eval));
        //     if max_eval >= beta {
        //         return max_eval;
        //     }
        //     if max_eval > alpha {
        //         alpha = max_eval;
        //     }
        // }

        self.tt.save(zobrist_hash, &state, depth as u8, eval);

        eval
    }

    fn iddfs_helper<T, F>(&self, f: F, duration: Duration, depth_limit: Option<u32>) -> T
    where
        T: 'static + Send,
        F: Fn(u32) -> T + 'static + Send + Sync,
    {
        // also this quitting method hangs some extra computation
        // will need to find out how to cut that down
        // might add a receiver into the type F

        let (eval_tx, eval_rx) = mpsc::channel();
        let (quit_tx, quit_rx) = mpsc::channel();

        self.pool.spawn(move || {
            let depths_iter: Box<dyn Iterator<Item = u32>> = match depth_limit {
                Some(d) => Box::new(1..d),
                None => Box::new(1..),
            };

            for d in depths_iter {
                let eval = f(d);

                // this does not do well enought at all. we are running way to much extra computation
                match quit_rx.try_recv() {
                    Ok(()) | Err(mpsc::TryRecvError::Disconnected) => break,
                    Err(mpsc::TryRecvError::Empty) => (),
                }

                // send result
                eval_tx.send(eval).unwrap();
            }
        });

        // maybe make duration optional later..
        thread::sleep(duration);

        quit_tx.send(()).unwrap();

        // get the most recent move suggested by the engine
        // will only panic if iddps didnt find a result (almost impossible)
        eval_rx.try_iter().last().unwrap()
    }
}

pub struct ActionScorePair<S: Searchable> {
    action: S::Action,
    score: f32,
}

impl<S: Searchable> ActionScorePair<S> {
    #[inline]
    pub fn action(&self) -> S::Action {
        self.action
    }

    #[inline]
    pub fn score(&self) -> f32 {
        self.score
    }
}

pub enum SearchConstraint {
    Depth(u32),
    Time(Duration),
    None,
}

impl SearchConstraint {
    pub fn depth(d: u32) -> Result<Self, &'static str> {
        if d > MAX_DEPTH {
            return Err("Depth too large! Pick lower than 25");
        }
        Ok(SearchConstraint::Depth(d))
    }

    pub fn time(t: u32) -> Result<Self, &'static str> {
        if t > MAX_TIME {
            return Err("Time to large! Pick lower than 300 seconds")
        }
        Ok(SearchConstraint::Time(Duration::from_millis(t.into())))
    }

    pub fn none() -> Self {
        SearchConstraint::None
    }
}
