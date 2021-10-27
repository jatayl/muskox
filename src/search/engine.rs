use std::cmp::{self, Reverse};
use std::default::Default;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use rayon::{ThreadPool, ThreadPoolBuilder};

use super::{tt::TranspositionTable, GameState, Optim, Score, Searchable, Side};

const MAX_DEPTH: u32 = 25;
const MAX_TIME: u32 = 300000;
const NUM_THREADS: usize = 8;

#[derive(Clone)]
pub struct Engine<S: Searchable> {
    tt: TranspositionTable<S>,
    pool: Arc<ThreadPool>,
}

impl<S: Searchable> Default for Engine<S> {
    fn default() -> Self {
        Engine::new()
    }
}

impl<S: Searchable> Engine<S> {
    pub fn new() -> Self {
        let tt = TranspositionTable::new(256);
        let pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(NUM_THREADS - 1)
                .build()
                .unwrap(),
        );

        Engine { tt, pool }
    }

    pub fn search(&mut self, state: &S, constraint: &SearchConstraint) -> Vec<ActionScorePair<S>> {
        self.tt.new_search(); // increment the generation

        let me = self.clone();
        let state = *state;

        // set the initial zobrist hash
        let zobrist_hash = state.zobrist_hash(); // this is relatively expensive function to call

        let compute_at_depth = move |depth| {
            let action_states = state.generate_all_actions();
            let evals: Vec<_> = action_states
                .iter()
                .map(|p| {
                    me.minmax_helper(
                        p.state(),
                        depth,
                        Score::NEG_INFINITY,
                        Score::INFINITY,
                        zobrist_hash ^ p.zobrist_diff(),
                    )
                })
                .collect();
            let mut results: Vec<_> = action_states
                .iter()
                .map(|p| p.action())
                .zip(evals.iter())
                .collect();
            // sort based on the evaluations
            results.sort_by(|a, b| match state.turn().optim() {
                Optim::Min => a.1.cmp(b.1),
                Optim::Max => b.1.cmp(a.1),
            });
            // can get rid of this part..
            results
                .into_iter()
                .map(|(&a, &s)| ActionScorePair {
                    action: a,
                    score: s,
                }) // copy all of the values and get rid of ordered float wrapper
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
            score: Score::from(0.),
        }
    }

    fn minmax_helper(
        &self,
        state: &S,
        depth: u32,
        mut alpha: Score,
        mut beta: Score,
        zobrist_hash: u64,
    ) -> Score {
        if let Some(value) = self.tt.probe(zobrist_hash, state, depth as u8) {
            return value;
        }

        if (depth == 0) | (state.get_game_state() != GameState::InProgress) {
            return state.evaluate();
        }

        let eval = match state.turn().optim() {
            Optim::Max => {
                let mut max_eval = Score::NEG_INFINITY;

                // sort it in reverse so we get higest nodes first for the max optimizer
                let mut nodes = state.generate_all_actions();
                nodes.sort_by_key(|n| Reverse(n.state().evaluate()));

                for (state_p, zobrist_diff) in nodes.iter().map(|a| (a.state(), a.zobrist_diff())) {
                    let zobrist_hash_p = zobrist_hash ^ zobrist_diff;
                    let eval = self.minmax_helper(state_p, depth - 1, alpha, beta, zobrist_hash_p);
                    max_eval = cmp::max(max_eval, eval);
                    alpha = cmp::max(alpha, max_eval);
                    if beta <= alpha {
                        break;
                    }
                }

                max_eval
            }

            Optim::Min => {
                let mut min_eval = Score::INFINITY;

                let mut nodes = state.generate_all_actions();
                nodes.sort_by_key(|n| n.state().evaluate()); // we want lowest values first

                for (state_p, zobrist_diff) in nodes.iter().map(|a| (a.state(), a.zobrist_diff())) {
                    let zobrist_hash_p = zobrist_hash ^ zobrist_diff;
                    let eval = self.minmax_helper(state_p, depth - 1, alpha, beta, zobrist_hash_p);
                    min_eval = cmp::min(min_eval, eval);
                    beta = cmp::min(beta, min_eval);
                    if beta <= alpha {
                        break;
                    }
                }

                min_eval
            }
        };

        self.tt.save(zobrist_hash, state, depth as u8, eval);

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

            for depth in depths_iter {
                let eval = f(depth);

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
    score: Score,
}

impl<S: Searchable> ActionScorePair<S> {
    #[inline]
    pub fn action(&self) -> S::Action {
        self.action
    }

    #[inline]
    pub fn score(&self) -> Score {
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
            return Err("Time to large! Pick lower than 300 seconds");
        }
        Ok(SearchConstraint::Time(Duration::from_millis(t.into())))
    }

    pub fn none() -> Self {
        SearchConstraint::None
    }
}
