use std::sync::RwLock;

use hashbrown::HashMap;

use crate::search::Searchable;

// will need way of regulating size

struct TTValue {
    depth: u32,
    score: f32,
}

// maybe use dashmap instead if it is threadsafe...
pub struct TranspositionTable<S: Searchable> {
    data: RwLock<HashMap<S, TTValue>>,
}

impl<S: Searchable> TranspositionTable<S> {
    pub fn new() -> Self {
        let data = RwLock::new(HashMap::new());
        TranspositionTable { data }
    }

    pub fn save(&self, state: &S, depth: u32, score: f32) {
        let value = TTValue { depth, score };
        // copying of the state isnt great but think it is the only thing i can do
        self.data.write().unwrap().insert(*state, value);
    }

    pub fn probe(&self, state: &S, depth: u32) -> Option<f32> {
        let data = self.data.read().unwrap();
        let value = data.get(&state)?;

        // if the score was captured too deep then we dont want it
        if value.depth < depth {
            return None;
        }

        Some(value.score)
    }
}