use std::fmt::{self, Debug};
use std::hash::Hash;
use crate::error;

pub enum Optim {
    Max,
    Min,
}

pub trait Searchable: 'static + Sized + Copy + Eq + Hash + Send + Sync {
    type Action: Send + Copy;
    type Side: Side;

    fn generate_all_actions(&self) -> Vec<ActionStatePair<Self>>;
    fn take_action(&self, _: &Self::Action) -> Result<Self, error::ActionError>;
    fn get_game_state(&self) -> GameState<Self>;
    fn turn(&self) -> Self::Side;
}

pub struct ActionStatePair<S: Searchable> {
    action: S::Action,
    state: S,
}

impl<S: Searchable> ActionStatePair<S> {
    pub fn new(action: S::Action, state: S) -> ActionStatePair<S> {
        ActionStatePair { action, state }
    }

    #[inline]
    pub fn action(&self) -> &S::Action {
        &self.action
    }

    #[inline]
    pub fn state(&self) -> &S {
        &self.state
    }
}

pub trait Evaluator<S: Searchable>: 'static + Send + Sync {
    fn eval(&self, _: &S) -> f32;
}

/// Represents a winner of a checkers game. The winner can either be a particular
/// player (denoted by [Color](enum.Color.html)) or a draw
#[derive(Debug, PartialEq)]
pub enum Winner<S: Searchable> {
    Player(S::Side),
    Draw
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
            GameState::Completed(winner) => {
                match winner {
                    Winner::Player(player) => write!(f, "Winner: {:?}", player),
                    Winner::Draw => write!(f, "Draw"),
                }
            },
            GameState::InProgress => write!(f, "In progress"),
        }
    }
}

pub trait Side: Debug + PartialEq {
    fn optim(&self) -> Optim;
}