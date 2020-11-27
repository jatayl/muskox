type Mask = u32;

static MASK_L3: Mask = 0x07070707;
static MASK_L5: Mask = 0x00e0e0e0;
static MASK_R3: Mask = 0xe0e0e0e0;
static MASK_R5: Mask = 0x07070700;

pub enum Color {
	Black,
	White,
}

pub struct Action();

pub struct Bitboard {
	blacks: Mask,
	whites: Mask,
	kings: Mask,
	turn: Color,  // maybe use a boolean instead
}

impl Bitboard {
	pub fn new() -> Bitboard {
		// initial state for a blank board
		Bitboard {
			blacks: 0x00030fff,
			whites: 0xfff00000,
			kings: 0,
			turn: Color::Black,
		}
	}
	// new from a certain string

	pub fn is_valid_move(&self, action: &Action) -> bool {
		true
	}

	pub fn make_move(&self, action: &Action) -> Result<Bitboard, &'static str> {
		if !self.is_valid_move(&action) {
			return Err("Invalid move")
		}

		Ok(Bitboard::new())
	}


	pub fn get_movers_white(&self) -> Mask {
		let not_occupied = !(self.whites | self.blacks);
		let white_kings = self.whites & self.kings;

		let mut movers = (not_occupied << 4 ) & self.whites;

		movers |= ((not_occupied & MASK_L3) << 3) & self.whites;
		movers |= ((not_occupied & MASK_L5) << 5) & self.whites;

		if white_kings != 0 {
			movers |= (not_occupied >> 4) & white_kings;
			movers |= ((not_occupied & MASK_R3) >> 3) & white_kings;
			movers |= ((not_occupied & MASK_R3) >> 5) & white_kings;
		}

		movers
	}

	pub fn get_jumpers_white(&self) -> Mask {
		let not_occupied = !(self.whites | self.blacks);
		let white_kings = self.whites & self.kings;

		let mut movers = 0;
		let mut temp = (not_occupied << 4) & self.blacks;

		if temp != 0 {
			movers |= ((temp & MASK_L3) << 3) | ((temp & MASK_L5) << 5) & self.whites;
		}

		temp = (((not_occupied & MASK_L3) << 3) | ((not_occupied & MASK_L5) >> 5)) & self.blacks;
		movers |= (temp << 4) & self.whites;

		if white_kings != 0 {
			temp = (not_occupied >> 4) & self.blacks;
			if temp != 0 {
				movers |= (((temp & MASK_R3) >> 3) | ((temp & MASK_R5) >> 5)) & self.whites;
			}
			temp = (((not_occupied & MASK_R3) >> 3) | ((not_occupied & MASK_R5) >> 5)) & self.blacks;
			if temp != 0 {
				movers |= (temp >> 4) & self.whites;
			}
		}
		movers
	}

	/// Returns a string graphically representing the board. The `b`'s represent
	/// the black pieces and the `w`'s represent the white pieces. A capital letters
	/// indicate that a certain piece is a king.
	///
	/// # Examples
	///
	/// ```
	/// use muskox::bitboard::Bitboard;
	/// 
	/// let board = Bitboard::new();
	/// println!("{}", board.pretty());
	/// ```
	pub fn pretty(&self) -> String {
		let mut out = String::with_capacity(1024);
		
		// maybe change this model
		let mut blacks_iter = self.blacks;
		let mut whites_iter = self.whites;
		let mut kings_iter = self.kings;

		// maybe get rid of i and j for something else
		for i in 0_u8..8 {      // rows
			out.push_str("+---+---+---+---+---+---+---+---+\n");
			for j in 0_u8..8 {  // cols
				if (i + j) % 2 == 0 {
					out.push_str("|   ");
					continue;
				}
				let c = {
					let mut c = ' ';
					if blacks_iter % 2 == 1 {
						c = 'b';
					}
					if whites_iter % 2 == 1 {
						c = 'w';
					}
					if kings_iter % 2 == 1 {
						c = c.to_uppercase().next().unwrap();
					}
					c
				};
				out.push_str("| ");
				out.push(c);
				out.push(' ');
				blacks_iter /= 2;
				whites_iter /= 2;
				kings_iter /= 2;
			}
			out.push_str("|\n");
		}

		out.push_str("+---+---+---+---+---+---+---+---+");

		out.chars().rev().collect()
	}

	// print that certain string
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_movers_white() {
		let board = Bitboard::new();

		assert_eq!(board.get_movers_white(), 0x00f00000);
	}
}