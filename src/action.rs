use std::cmp;
use std::mem;

// need lookup table for square index for next direction

#[derive(Debug, PartialEq)]
pub enum Direction {
	UpLeft,
	UpRight,
	DownLeft,
	DownRight,
}

// source: 5, destination: 5, jump length: 5, jump directions: 8 * 2 bits (four directions), unused: 1
pub struct Action(u32);

impl Action {
	pub fn new_from_vector(positions: Vec<u8>) -> Result<Action, &'static str> {
		// rename method

		let positions: Vec<_> = positions.iter().map(|x| x - 1).collect();

		// check that all of the position numbers are in the right range
		if positions.iter().any(|&x| x > 31) {
			return Err("Invalid position number!");
		}

		// check to see if it is a valid length of position vector with max number of moves is 8
		if positions.len() < 2 || positions.len() > 9 {
			return Err("Invalid number of moves!");
		}

		let source = positions[0];
		let destination = *positions.last().unwrap();

		let mut data = source as u32;                       // source
		data |= (destination as u32) << 5;                  // destination

		let diff = cmp::max(source, destination) - cmp::min(source, destination);

		// check if this action has jumps in it
		if positions.len() > 2 || (diff != 4 && diff != 5) {
			data |= ((positions.len() - 1) << 10) as u32;  // jump length

			for i in 0..(positions.len() - 1) {
				let diff = (positions[i + 1] as i8) - (positions[i] as i8);
				let direction = match diff {
					-9 => Direction::UpLeft,
					-7 => Direction::UpRight,
					7 => Direction::DownLeft,
					9 => Direction::DownRight,
					_ => return Err("Invalid jump!"),
				};

				let shift = i * 2 + 15;
				data |= (direction as u32) << shift;       // jump direction
			}
		}

		Ok(Action(data))
	}

	pub fn new_from_movetext(movetext: &str) -> Result<Action, &'static str> {
		let positions: Vec<_> = movetext.split("-")
			.map(|x| x.parse::<u8>().expect("Not valid board square"))
			.collect();

		Action::new_from_vector(positions)
	}

	// getters

	#[inline]
	pub fn source(&self) -> u8 {
		(self.0 & 31) as u8
	}

	#[inline]
	pub fn destination(&self) -> u8 {
		((self.0 >> 5) & 31) as u8
	}

	#[inline]
	pub fn jump_len(&self) -> u8 {
		((self.0 >> 10) & 15) as u8
	}

	#[inline]
	pub fn direction(&self, i: u8) -> Option<Direction> {
		if i >= self.jump_len() {
			return None
		}
		let val = (self.0 >> (i * 2 + 15)) & 3;
		Some(unsafe { mem::transmute(val as u8) })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_from_movetext_test() {
		let action = Action::new_from_movetext("1-10-17").unwrap();
		assert_eq!(action.source(), 0);
		assert_eq!(action.destination(), 16);
		assert_eq!(action.jump_len(), 2);
		assert_eq!(action.direction(0), Some(Direction::DownRight));
		assert_eq!(action.direction(1), Some(Direction::DownLeft));
		assert_eq!(action.direction(2), None);

		let action = Action::new_from_movetext("1-6").unwrap();
		assert_eq!(action.source(), 0);
		assert_eq!(action.destination(), 5);
		assert_eq!(action.jump_len(), 0);
		assert_eq!(action.direction(0), None);

		let action = Action::new_from_movetext("10-19-12-3").unwrap();
		assert_eq!(action.source(), 9);
		assert_eq!(action.destination(), 2);
		assert_eq!(action.jump_len(), 3);
		assert_eq!(action.direction(1), Some(Direction::UpRight));
		assert_eq!(action.direction(2), Some(Direction::UpLeft));
		assert_eq!(action.direction(4), None);
	}
}