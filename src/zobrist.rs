use lazy_static::lazy_static;

use crate::bitboard::Color;

const SEED: u64 = 25184470690726;

// maybe make an init function
// i dont want to have to deal with mutability of static though
lazy_static! {
    static ref ZOBRIST_TABLE: [u64; 97] = {
        let mut table = [0; 97];

        // maybe make the seed time or something
        let mut prng = Prng::new(SEED);

        for entry in table.iter_mut() {
            *entry = prng.rand64()
        }

        table
    };
}

#[inline]
pub fn get_position_hash(position: u8, color: Color, is_king: bool) -> u64 {
    // maybe check the bounds on position and return and option
    let position = position as usize;

    // get a particular hash for a color and a position
    let mut hash = match color {
        Color::Black => ZOBRIST_TABLE[position],
        Color::White => ZOBRIST_TABLE[32 + position],
    };

    if is_king {
        hash ^= ZOBRIST_TABLE[64 + position];
    }

    hash
}

#[inline]
pub fn get_turn_hash() -> u64 {
    // get the hash for the turn
    ZOBRIST_TABLE[96]
}

struct Prng {
    s: u64,
}

impl Prng {
    fn new(seed: u64) -> Self {
        Prng { s: seed }
    }

    fn rand64(&mut self) -> u64 {
        self.s ^= self.s >> 12;
        self.s ^= self.s << 25;
        self.s ^= self.s >> 27;
        self.s.wrapping_mul(2685821657736338717)
    }
}
