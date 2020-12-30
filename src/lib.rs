pub mod app;
pub mod error;

mod bitboard;
mod action;
mod evaluation;

pub mod search {
    mod search;
    mod engine;
    mod tt;
    pub use search::*;
    pub use engine::*;
    pub use tt::*;
}

pub mod board {
    pub use super::bitboard::*;
    pub use super::action::*;
    pub use super::evaluation::*;
}