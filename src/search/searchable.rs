use std::fmt::{self, Debug};
use std::hash::Hash;

use crate::error;

pub enum Optim {
    Max,
    Min,
}

pub trait Searchable: 'static + Sized + Copy + Eq + Hash + Default + Send + Sync {
    type Action: Copy + Send + PartialEq;
    type Side: Side;

    fn generate_all_actions(&self) -> Vec<ActionStatePair<Self>>;
    fn take_action(&self, _: &Self::Action) -> Result<Self, error::ActionError>;
    fn get_game_state(&self) -> GameState<Self>;
    fn turn(&self) -> Self::Side;
    fn evaluate(&self) -> super::Score;
    fn zobrist_hash(&self) -> u64;
}

pub struct ActionStatePair<S: Searchable> {
    action: S::Action,
    state: S,
    zobrist_diff: u64,
}

impl<S: Searchable> ActionStatePair<S> {
    pub fn new(action: S::Action, state: S, zobrist_diff: u64) -> ActionStatePair<S> {
        ActionStatePair {
            action,
            state,
            zobrist_diff,
        }
    }

    #[inline]
    pub fn action(&self) -> &S::Action {
        &self.action
    }

    #[inline]
    pub fn state(&self) -> &S {
        &self.state
    }

    #[inline]
    pub fn zobrist_diff(&self) -> &u64 {
        &self.zobrist_diff
    }
}

/// Represents a winner of a checkers game. The winner can either be a particular
/// player (denoted by [Color](enum.Color.html)) or a draw
#[derive(Debug, PartialEq)]
pub enum Winner<S: Searchable> {
    Player(S::Side),
    Draw,
}

/// Represents the current state of a  game. It is either completed with a
/// [winner](enum.Winner.html) or still in progress.
#[derive(Debug, PartialEq)]
pub enum GameState<S: Searchable> {
    Completed(Winner<S>),
    InProgress,
}

impl<S: Searchable> fmt::Display for GameState<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            GameState::Completed(winner) => match winner {
                Winner::Player(player) => write!(f, "Winner: {:?}", player),
                Winner::Draw => write!(f, "Draw"),
            },
            GameState::InProgress => write!(f, "In progress"),
        }
    }
}

pub trait Side: Debug + PartialEq {
    fn optim(&self) -> Optim;
}
