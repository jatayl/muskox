use std::cmp;
use std::fmt;

use crate::error::ParseActionError;

// need lookup table for square index for next direction

/// Represents one of the four directions one can move in the game of checkers
#[derive(PartialEq, Debug)] // dont need to keep debug
pub enum Direction {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    // these three methods need an assessment of implementation and naming

    pub(crate) fn between(source: u8, destination: u8) -> Option<Self> {
        // this function is not safe yet... and its really messy

        let diff = (destination as i8) - (source as i8);
        // should aim to try to change this or move it elsewhere

        // check jump
        match diff {
            -9 => return Some(Direction::UpLeft),
            -7 => return Some(Direction::UpRight),
            7 => return Some(Direction::DownLeft),
            9 => return Some(Direction::DownRight),
            _ => (),
        }

        // check move
        if source / 4 % 2 == 0 {
            // even rows
            match diff {
                -4 => Some(Direction::UpLeft),
                -3 => Some(Direction::UpRight),
                4 => Some(Direction::DownLeft),
                5 => Some(Direction::DownRight),
                _ => None,
            }
        } else {
            // odd rows
            match diff {
                -5 => Some(Direction::UpLeft),
                -4 => Some(Direction::UpRight),
                3 => Some(Direction::DownLeft),
                4 => Some(Direction::DownRight),
                _ => None,
            }
        }
    }

    pub(crate) fn relative_to(&self, position: u8) -> Option<u8> {
        // need to check boundaries
        if position > 31 {
            return None;
        }
        let position = position as i8;
        let out;
        // maybe make this a cache would be better in terms of simplicity and performance?
        // could defintely make it more compact
        if position / 4 % 2 == 0 {
            // even rows
            out = match *self {
                Direction::UpLeft => position - 4,
                Direction::UpRight => position - 3,
                Direction::DownLeft => position + 4,
                Direction::DownRight => position + 5,
            }
        } else {
            // odd rows
            out = match *self {
                Direction::UpLeft => position - 5,
                Direction::UpRight => position - 4,
                Direction::DownLeft => position + 3,
                Direction::DownRight => position + 4,
            }
        }
        if !(0..=31).contains(&out) {
            return None;
        }
        // ensure that we dont escape the bounds on the board.
        // maybe change this strat later
        let in_col = position % 4;
        let out_col = out % 4;
        if out_col - in_col != 0 && out_col - in_col != 1 && out_col - in_col != -1 {
            return None;
        }
        Some(out as u8)
    }

    pub(crate) fn relative_jump_from(&self, position: u8) -> Option<u8> {
        let position = position as i8;
        // maybe rename this method
        if position > 31 {
            return None;
        }
        let out = match *self {
            Direction::UpLeft => position - 9,
            Direction::UpRight => position - 7,
            Direction::DownLeft => position + 7,
            Direction::DownRight => position + 9,
        };
        if !(0..=31).contains(&out) {
            return None;
        }

        let in_col = (position % 4) as i8;
        let out_col = (out % 4) as i8;
        if out_col - in_col != -1 && out_col - in_col != 1 {
            return None;
        }
        Some(out as u8)
    }
}

/// Represents one of the two types of moves that exist in checkers
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActionType {
    Move,
    Jump,
}

// source: 5, destination: 5, jump length: 5, jump directions: 8 * 2 bits (four directions), unused: 1
/// Represents an action that can be made on a checkerboard
#[derive(PartialEq, Clone, Copy)]
pub struct Action(u32);

impl Action {
    /// Creates a new checkers action from a vector of positions.
    ///
    /// # Arguments
    ///
    /// * `positions` - A vector of standard position numbers representing a move
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::Action;
    ///
    /// let action = Action::from_vector(vec![19, 24]).unwrap();
    /// assert_eq!(action.source(), 18);  // note that internal representation starts from 0, no longer 1.
    pub fn from_vector(positions: Vec<u8>) -> Result<Self, ParseActionError> {
        // maybe make this method work for all iterators and not just vectors
        let positions: Vec<_> = positions.iter().map(|x| x - 1).collect();

        // check that all of the position numbers are in the right range
        if let Some(pos) = positions.iter().find(|&&x| x > 31) {
            return Err(ParseActionError::PositionValueError {
                position: pos.to_string(),
            });
        }

        // check to see if it is a valid length of position vector with max number of moves is 8
        if positions.len() < 2 || positions.len() > 9 {
            return Err(ParseActionError::MoveQuantityError {
                quantity: positions.len(),
            });
        }

        let source = positions[0];
        let destination = *positions.last().unwrap();

        let mut data = source as u32; // source
        data |= (destination as u32) << 5; // destination

        let abs_diff = cmp::max(source, destination) - cmp::min(source, destination);

        // check if this action has jumps in it
        if positions.len() > 2 || (abs_diff != 3 && abs_diff != 4 && abs_diff != 5) {
            data |= ((positions.len() - 1) << 10) as u32; // jump length

            for i in 0..(positions.len() - 1) {
                let diff = (positions[i + 1] as i8) - (positions[i] as i8);
                let direction = match diff {
                    -9 => Direction::UpLeft,
                    -7 => Direction::UpRight,
                    7 => Direction::DownLeft,
                    9 => Direction::DownRight,
                    _ => {
                        return Err(ParseActionError::PositionValueError {
                            position: positions[i].to_string(),
                        })
                    }
                };

                let shift = i * 2 + 15;
                data |= (direction as u32) << shift; // jump direction
            }
        }

        Ok(Action(data))
    }

    /// Creates a new checkers action from a string movetext according to Portable Draughts Notation.
    /// (PDN). Read more about the notation [here](https://en.wikipedia.org/wiki/Portable_Draughts_Notation).
    ///
    /// # Arguments
    ///
    /// * `movetext` - A string slice that that represents movetext written for PDN notation
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::Action;
    ///
    /// let action = Action::from_movetext("19-24").unwrap();
    /// assert_eq!(action.source(), 18);  // note that internal representation starts from 0, no longer 1.
    /// ```
    pub fn from_movetext(movetext: &str) -> Result<Self, ParseActionError> {
        let positions: Vec<_> = movetext
            .split('-')
            .map(|x| {
                x.parse::<u8>()
                    .map_err(|_| ParseActionError::PositionValueError {
                        position: x.to_string(),
                    })
            })
            .collect::<Result<_, ParseActionError>>()?;

        Action::from_vector(positions)
    }

    /// Returns the starting location of a particular action
    #[inline]
    pub fn source(&self) -> u8 {
        (self.0 & 31) as u8
    }

    /// Returns the ending location of a particular action
    #[inline]
    pub fn destination(&self) -> u8 {
        ((self.0 >> 5) & 31) as u8
    }

    /// Returns how many leaps were made in a particular action
    #[inline]
    pub fn jump_len(&self) -> u8 {
        ((self.0 >> 10) & 15) as u8
    }

    /// Returns the direction of a particular jump
    ///
    /// This is wrapped in an option, because if no jumps were performed then
    /// no jump directions can be retrieved (`None`).
    ///
    /// # Arguments
    ///
    /// * `i` - The index of the jump to find the direction for
    ///
    #[inline]
    pub fn jump_direction(&self, i: u8) -> Option<Direction> {
        // maybe rename to jump_direction
        if i >= self.jump_len() {
            return None;
        }
        match (self.0 >> (i * 2 + 15)) & 3 {
            0 => Some(Direction::UpLeft),
            1 => Some(Direction::UpRight),
            2 => Some(Direction::DownLeft),
            3 => Some(Direction::DownRight),
            _ => None,
        }
    }

    /// Returns the type of a particular action
    #[inline]
    pub fn action_type(&self) -> ActionType {
        match self.jump_len() {
            0 => ActionType::Move,
            _ => ActionType::Jump,
        }
    }

    /// Returns the direction of a move action.
    ///
    /// This is also wrapped in an option, because if the action represents a
    /// jump, then a notion of a move direction is not relevant.
    #[inline]
    pub fn move_direction(&self) -> Option<Direction> {
        if self.action_type() == ActionType::Jump {
            return None;
        }

        let source = self.source();
        let destination = self.destination();

        Direction::between(source, destination)
    }

    /// Generate movetext for a particular action
    pub fn movetext(&self) -> String {
        let source = self.source();

        match self.action_type() {
            ActionType::Move => format!("{}-{}", source + 1, self.destination() + 1),
            ActionType::Jump => {
                let mut out = format!("{}-", source + 1);
                let mut curr = source;

                for i in 0..self.jump_len() {
                    curr = self
                        .jump_direction(i)
                        .unwrap()
                        .relative_jump_from(curr)
                        .unwrap();
                    out.push_str(&format!("{}-", curr + 1));
                }

                out.pop(); // excess '-'

                out
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.movetext())
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Action({})", self.movetext())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MOVE_1: &'static str = "1-10-17";
    const TEST_MOVE_2: &'static str = "1-6";
    const TEST_MOVE_3: &'static str = "10-19-12-3";
    const TEST_MOVE_4: &'static str = "15-11";

    #[test]
    fn relative_position_test() {
        let dir = Direction::between(1, 10);
        assert_eq!(dir, Some(Direction::DownRight));

        let dir = Direction::between(15, 11);
        assert_eq!(dir, Some(Direction::UpRight));

        let pos = Direction::DownRight.relative_to(1);
        assert_eq!(pos, Some(6));

        let pos = Direction::UpLeft.relative_to(24);
        assert_eq!(pos, Some(20));

        let pos = Direction::DownLeft.relative_jump_from(10);
        assert_eq!(pos, Some(17));

        let pos = Direction::UpRight.relative_jump_from(43);
        assert_eq!(pos, None);

        let pos = Direction::UpRight.relative_jump_from(7);
        assert_eq!(pos, None); // does not work yet
    }

    #[test]
    fn action_overview_test() {
        let action = Action::from_movetext(TEST_MOVE_1).unwrap();
        assert_eq!(action.source(), 0);
        assert_eq!(action.destination(), 16);
        assert_eq!(action.jump_len(), 2);
        assert_eq!(action.action_type(), ActionType::Jump);

        let action = Action::from_movetext(TEST_MOVE_2).unwrap();
        assert_eq!(action.source(), 0);
        assert_eq!(action.destination(), 5);
        assert_eq!(action.jump_len(), 0);
        assert_eq!(action.action_type(), ActionType::Move);

        let action = Action::from_movetext(TEST_MOVE_3).unwrap();
        assert_eq!(action.source(), 9);
        assert_eq!(action.destination(), 2);
        assert_eq!(action.jump_len(), 3);
        assert_eq!(action.action_type(), ActionType::Jump);
    }

    #[test]
    fn direction_test() {
        let action = Action::from_movetext(TEST_MOVE_1).unwrap();
        assert_eq!(action.jump_direction(0), Some(Direction::DownRight));
        assert_eq!(action.jump_direction(1), Some(Direction::DownLeft));
        assert_eq!(action.jump_direction(2), None);

        let action = Action::from_movetext(TEST_MOVE_2).unwrap();
        assert_eq!(action.jump_direction(0), None);

        let action = Action::from_movetext(TEST_MOVE_3).unwrap();
        assert_eq!(action.jump_direction(1), Some(Direction::UpRight));
        assert_eq!(action.jump_direction(2), Some(Direction::UpLeft));
        assert_eq!(action.jump_direction(4), None);
    }

    #[test]
    fn move_direction_test() {
        let action = Action::from_movetext(TEST_MOVE_1).unwrap();
        assert_eq!(action.move_direction(), None);

        let action = Action::from_movetext(TEST_MOVE_2).unwrap();
        assert_eq!(action.move_direction(), Some(Direction::DownRight));

        let action = Action::from_movetext(TEST_MOVE_4).unwrap();
        assert_eq!(action.move_direction(), Some(Direction::UpRight));
    }
}
