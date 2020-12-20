use hashbrown::HashMap;

use crate::Bitboard;

// will need way of regulating size

struct TTValue {
    depth: u32,
    score: f32,
}

pub struct TranspositionTable {
    data: HashMap<Bitboard, TTValue>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let data = HashMap::new();
        TranspositionTable { data }
    }

    pub fn insert(&mut self, board: &Bitboard, depth: u32, score: f32) {
        let value = TTValue { depth, score };
        // copying of the board isnt great but think it is the only thing i can do
        self.data.insert(*board, value);
    }

    pub fn get(&self, board: &Bitboard, depth: u32) -> Option<f32> {
        let value = self.data.get(&board)?;

        // if the score was captured too deep then we dont want it
        if value.depth <= depth {
            return None;
        }

        Some(value.score)
    }
}