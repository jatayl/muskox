pub mod app;
pub mod error;

mod action;
mod bitboard;
mod evaluation;
mod zobrist;

pub mod search {
    mod engine;
    mod score;
    mod searchable;
    mod tt;

    pub use engine::*;
    pub use score::*;
    pub use searchable::*;
}

pub mod board {
    pub use super::action::*;
    pub use super::bitboard::*;
}
