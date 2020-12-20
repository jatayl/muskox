use std::sync::RwLock;

use hashbrown::HashMap;

use crate::Bitboard;

// will need way of regulating size

struct TTValue {
    depth: u32,
    score: f32,
}

// maybe use dashmap instead if it is threadsafe...
pub struct TranspositionTable {
    data: RwLock<HashMap<Bitboard, TTValue>>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let data = RwLock::new(HashMap::new());
        TranspositionTable { data }
    }

    pub fn insert(&self, board: &Bitboard, depth: u32, score: f32) {
        let value = TTValue { depth, score };
        // copying of the board isnt great but think it is the only thing i can do
        self.data.write().unwrap().insert(*board, value);
    }

    pub fn get(&self, board: &Bitboard, depth: u32) -> Option<f32> {
        let data = self.data.read().unwrap();
        let value = data.get(&board)?;

        // if the score was captured too deep then we dont want it
        if value.depth <= depth {
            return None;
        }

        Some(value.score)
    }
}