use std::default;
use std::mem;
use std::sync::{Arc, RwLock};

use crate::search::{Score, Searchable};

const DEFAULT_FLAG: u8 = 255;
const CLUSTER_SIZE: usize = 3;

#[derive(Clone, Copy)]
struct TTEntry<S: Searchable> {
    state: S,
    depth: u8,
    score: Score,
    generation: u8,
}

impl<S: Searchable> default::Default for TTEntry<S> {
    fn default() -> Self {
        TTEntry {
            state: S::default(),
            depth: DEFAULT_FLAG,
            score: Score::from(0.),
            generation: 0,
        }
    }
}

impl<S: Searchable> TTEntry<S> {
    fn replace_value(&self, current_generation: u8) -> u8 {
        // stockfish uses 8 as the multipler
        self.depth - 4 * (current_generation - self.generation)
    }
}

type Cluster<S> = RwLock<[TTEntry<S>; CLUSTER_SIZE]>;

#[derive(Clone)]
pub struct TranspositionTable<S: Searchable> {
    clusters: Arc<[Cluster<S>]>,
    n_clusters: usize,
    generation: u8,
}

impl<S: Searchable> TranspositionTable<S> {
    pub fn new(size_mb: usize) -> Self {
        let size_b = size_mb * 1024 * 1024;
        let cluster_size = mem::size_of::<Cluster<S>>();
        let n_clusters = size_b / cluster_size;

        let clusters = (0..n_clusters)
            .map(|_| RwLock::new([TTEntry::default(); 3]))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let clusters = Arc::from(clusters);

        let generation = 1;

        TranspositionTable {
            clusters,
            n_clusters,
            generation,
        }
    }

    pub fn new_search(&mut self) {
        self.generation += 1;
    }

    pub fn save(&self, zobrist_hash: u64, &state: &S, depth: u8, score: Score) {
        let generation = self.generation;
        let entry = TTEntry {
            state,
            depth,
            score,
            generation,
        };

        let key = zobrist_hash as usize % self.n_clusters;
        let mut cluster = self.clusters[key].write().unwrap();

        for i in 0..CLUSTER_SIZE {
            if entry.replace_value(generation) > cluster[i].replace_value(generation)
                || cluster[i].depth == DEFAULT_FLAG
            {
                cluster[i] = entry;
                return;
            }
        }

        // // The proposed implementation below checks to see if any in cluster are in default state
        // // before overwriting based on replacement value
        // // However, it isnt implemented do to performance concerns with two iterations

        // // first iteration checks if any are defaults using the DEFAULT_FLAG
        // for i in 0..CLUSTER_SIZE {
        //     if cluster[i].depth == DEFAULT_FLAG {
        //         cluster[i] = entry;
        //         return;
        //     }
        // }

        // // second one checks replacement value
        // for i in 0..CLUSTER_SIZE {
        //     if entry.replace_value(generation) > cluster[i].replace_value(generation) {
        //         cluster[i] = entry;
        //         return;
        //     }
        // }
    }

    pub fn probe(&self, zobrist_hash: u64, state: &S, depth: u8) -> Option<Score> {
        let key = zobrist_hash as usize % self.n_clusters;
        let cluster = self.clusters[key].read().unwrap();

        // iterate over the cluster
        for i in 0..CLUSTER_SIZE {
            if cluster[i].depth >= depth && cluster[i].state == *state {
                // its a match!
                return Some(cluster[i].score);
            }
        }

        None
    }

    pub fn resize(&mut self, size_mb: usize) {
        let size_b = size_mb * 1024 * 1024;
        let cluster_size = mem::size_of::<Cluster<S>>();
        let n_clusters = size_b / cluster_size;

        let clusters = (0..n_clusters)
            .map(|_| RwLock::new([TTEntry::default(); 3]))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        self.clusters = Arc::from(clusters);
        self.n_clusters = n_clusters;
        self.generation = 1;
    }
}
