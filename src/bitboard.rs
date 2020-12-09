use std::mem;

use crate::Action;
use crate::ActionType;
use crate::Direction;

type Mask = u32;

static MASK_L3: Mask = 0x07070707;
static MASK_L5: Mask = 0x00e0e0e0;
static MASK_R3: Mask = 0xe0e0e0e0;
static MASK_R5: Mask = 0x07070700;

// maybe want to consider making a piece enum so we can abstract the king

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Black,
    White,
}
use Color::*;

#[derive(Debug, PartialEq)]
pub enum Winner {
    Player(Color),
    Draw
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    Completed(Winner),
    InProgress,
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    blacks: Mask,
    whites: Mask,
    kings: Mask,
    turn: Color,
}

impl Bitboard {
    pub fn new() -> Bitboard {
        // initial state for a blank board
        Bitboard {
            blacks: 0x00000fff,
            whites: 0xfff00000,
            kings: 0,
            turn: Black,
        }
    }

    /// Creates a new bitboard from a string FEN tag according to Portable Draughts Notation.
    /// (PDN). Read more about the notation [here](https://en.wikipedia.org/wiki/Portable_Draughts_Notation).
    ///
    /// # Arguments
    ///
    /// * `fen_string` - A string slice that that represents the checkers board from
    /// a fen tag in PDN Notation
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::bitboard::Bitboard;
    ///
    /// let b = Bitboard::new_from_fen("B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29");
    /// // will put proof that it works here
    /// ```
    pub fn new_from_fen(fen_string: &str) -> Result<Bitboard, &'static str> {
        // maybe clean up errors handling here. they are rather rigid could make them more useful

        let fen_string: String = fen_string.chars().filter(|c| !c.is_whitespace()).collect();

        let d: Vec<_> = fen_string.split(":").collect();

        if d.len() != 3 {
            return Err("There should be two ':'s in the FEN string!")
        }

        let turn = match d[0] {
            "B" => Black,
            "W" => White,
            _ => return Err("Not a valid turn color!"),
        };

        let mut blacks: Mask = 0;
        let mut whites: Mask = 0;
        let mut kings: Mask = 0;

        for position in d.iter().skip(1) {
            let pieces_string: String = position.chars().skip(1).collect();
            let pieces: Vec<_> = pieces_string.split(",").collect();

            let mut temp: Mask = 0;

            for piece in pieces.iter() {
                let (piece_n, is_king) = match piece.chars().next() {
                    Some('K') => (piece.chars().skip(1).collect::<String>().parse::<u8>()
                        .or_else(|_| Err("Invalid piece number!"))? - 1, true),
                    Some(_) => (piece.parse::<u8>().or_else(|_| Err("Invalid piece number!"))? - 1, false),
                    None => break,  // if no pieces exist. not super pretty or clear like this...
                };

                if piece_n > 31 {  // already had been converted to computer numbers
                    return Err("Invalid piece number");
                }

                temp |= 1 << piece_n;

                if is_king {
                    kings |= 1 << piece_n;
                }
            }

            // match the color to assign it to
            match position.chars().next().ok_or("Not a valid position color")? {
                'B' => blacks |= temp,
                'W' => whites |= temp,
                _ => return Err("Not a valid position color")
            };
        }

        Ok(Bitboard{ blacks, whites, kings, turn })
    }

    pub fn get_game_state(&self) -> GameState {
        // check if somebody can't move
        if self.turn == Black && self.get_movers(&Black) == 0 && self.get_jumpers(&Black) == 0 {
            return GameState::Completed(Winner::Player(White));
        }
        if self.turn == White && self.get_movers(&Black) == 0 && self.get_jumpers(&Black) == 0 {
            return GameState::Completed(Winner::Player(Black));
        }

        // need to figure out how to determine if there is a draw
        // maybe make it so the computer can agree to a draw
        if false {
            return GameState::Completed(Winner::Draw);
        }

        // if none of these are satisfied, then the game is still in progress
        GameState::InProgress
    }

    pub fn validate_action(&self, action: &Action) -> Result<(), &'static str> {
        self.take_action(&action)?;
        Ok(())
    }

    pub fn take_action(&self, action: &Action) -> Result<Bitboard, &'static str> {
        // this is an epic monolith of a function. need to break it into smaller methods/
        // and apply clever techniques such as bitshifting and more rust like stuff.

        // make some of these lambdas private methods
        let coloring_eq = |loc: u8, color: &Color| {
            let color_mask = match color {
                White => self.whites,
                Black => self.blacks,
            };
            (color_mask >> loc) % 2 == 1
        };

        let is_empty = |loc: u8| (self.whites >> loc) % 2 == 0 && (self.blacks >> loc) % 2 == 0;

        let remove_piece = |loc: u8, board: &mut Bitboard| {
            let mask = !(1 << loc);
            board.whites &= mask;
            board.blacks &= mask;
            board.kings &= mask;
        };

        let add_piece = |loc: u8, color: &Color, is_king: bool, board: &mut Bitboard| {
            let mask = 1 << loc;
            match color {
                Black => board.blacks |= mask,
                White => board.whites |= mask,
            };
            if is_king {
                board.kings |= mask;
            }
        };
        let is_king = |loc: u8| (self.kings >> loc) % 2 == 1;


        let mut board_p = self.clone();

        let source = action.source();
        let destination = action.destination();

        let starts_as_king = is_king(source);

        let ends_as_king = {
            let dest_row = destination / 4;
            // will be a king if it was a king or will be in end row last
            starts_as_king || dest_row == 0 || dest_row == 7
        };

        // sketchy way of flipping the turn color enum
        // maybe just match with the opposite color instead
        let opponent_color: Color = unsafe { mem::transmute((self.turn as u8 + 1) % 2) };

        // erase color from source
        remove_piece(source, &mut board_p);

        // add color to destination
        add_piece(destination, &self.turn, ends_as_king, &mut board_p);

        // ensure that the source has turn color
        if !coloring_eq(source, &self.turn) {
            return Err("Source must be occupied by the actor!");
        }

        // ensure that destination is empty.
        if !is_empty(action.destination()) {
            return Err("Destination must be empty!");
        }

        match action.action_type() {
            ActionType::Move => {
                // ensure that no jumpers are available
                if self.get_jumpers(&self.turn) != 0 {
                    return Err("Have to jump a piece!");
                }

                // ensure that it only moves backwards if source is a king
                // reformat the next dozen or so lines
                let move_direction = action.move_direction().unwrap();

                if (move_direction == Direction::UpLeft || move_direction == Direction::UpRight) &&
                        self.turn == Black && !is_king(source) {
                    return Err("Only kings can move backwards!")
                }

                if (move_direction == Direction::DownLeft || move_direction == Direction::DownRight) &&
                        self.turn == White && !is_king(source) {
                    return Err("Only kings can move backwards!")
                }
            },

            ActionType::Jump => {
                let mut curr = source;
                // maybe make a jump iterator. that would be super cool!
                for i in 0..action.jump_len() {
                    let jump_direction = action.jump_direction(i).unwrap();

                    // ensure that only jump backwards if it is a king
                    if (jump_direction == Direction::UpLeft || jump_direction == Direction::UpRight) &&
                            self.turn == Black && !starts_as_king {
                        return Err("Only kings can move backwards!")
                    }

                    if (jump_direction == Direction::DownLeft || jump_direction == Direction::DownRight) &&
                            self.turn == White && !starts_as_king {
                        return Err("Only kings can move backwards!")
                    }

                    // see if we can try to use bitmasking to get this next time
                    let skipped_over = {
                        if source / 4 % 2 == 0 {  // even rows
                            match jump_direction {
                                Direction::UpLeft => curr - 4,
                                Direction::UpRight => curr - 3,
                                Direction::DownLeft => curr + 4,
                                Direction::DownRight => curr + 5,
                            }
                        } else {  // odd rows
                            match jump_direction {
                                Direction::UpLeft => curr - 5,
                                Direction::UpRight => curr - 4,
                                Direction::DownLeft => curr + 3,
                                Direction::DownRight => curr + 4,
                            }
                        }
                    };  // need to get that skipped over piece...

                    // ensure that it actually jumps over another piece that is not its own color
                    if !coloring_eq(skipped_over, &opponent_color) {
                        return Err("You can only skip over a square with an opponent on it!");
                    }

                    remove_piece(skipped_over, &mut board_p);

                    curr = match jump_direction {
                        Direction::UpLeft => curr - 9,
                        Direction::UpRight => curr - 7,
                        Direction::DownLeft => curr + 7,
                        Direction::DownRight => curr + 9,
                    };
                }
                // ensure that it there isnt another jump for it to do at destination
                if (board_p.get_jumpers(&self.turn) & 1 << destination != 0) & !(!starts_as_king & ends_as_king) {
                    return Err("Need to keep jumping!");
                }
            }
        }

        board_p.turn = opponent_color;

        Ok(board_p)
    }

    /// Returns a u32 mask that represents all of the white pieces that can move.
    /// Recognize that this does not include the white pieces that can jump. To
    /// access those use `get_jumpers_white`.
    fn get_movers(&self, color: &Color) -> Mask {
        let not_occupied = !(self.whites | self.blacks);

        match color {
            White => {
                let white_kings = self.whites & self.kings;

                let mut movers = (not_occupied << 4) & self.whites;

                movers |= ((not_occupied & MASK_R3) << 3) & self.whites;
                movers |= ((not_occupied & MASK_R5) << 5) & self.whites;

                if white_kings != 0 {
                    movers |= (not_occupied >> 4) & white_kings;
                    movers |= ((not_occupied & MASK_L3) >> 3) & white_kings;
                    movers |= ((not_occupied & MASK_L5) >> 5) & white_kings;
                }

                movers
            },
            Black => {
                let black_kings = self.blacks & self.kings;

                let mut movers = (not_occupied >> 4) & self.blacks;

                movers |= ((not_occupied & MASK_L3) >> 3) & self.blacks;
                movers |= ((not_occupied & MASK_L5) >> 5) & self.blacks;

                if black_kings != 0 {
                    movers |= (not_occupied << 4) & black_kings;
                    movers |= ((not_occupied & MASK_R3) << 3) & black_kings;
                    movers |= ((not_occupied & MASK_R5) << 5) & black_kings;
                }

                movers
            }
        }
    }

    fn get_jumpers(&self, color: &Color) -> Mask {
        let not_occupied = !(self.whites | self.blacks);

        match color {
            White => {
                let white_kings = self.whites & self.kings;

                let mut movers = 0;
                let mut temp = (not_occupied << 4) & self.blacks;

                movers |= ((temp & MASK_R3) << 3) | ((temp & MASK_R5) << 5);

                temp = (((not_occupied & MASK_R3) << 3) | ((not_occupied & MASK_R5) << 5)) & self.blacks;
                movers |= temp << 4;

                movers &= self.whites;

                if white_kings != 0 {
                    temp = (not_occupied >> 4) & self.blacks;
                    movers |= (((temp & MASK_L3) >> 3) | ((temp & MASK_L5) >> 5)) & white_kings;
                    temp = (((not_occupied & MASK_L3) >> 3) | ((not_occupied & MASK_L5) >> 5)) & self.blacks;
                    movers |= (temp >> 4) & white_kings;
                }

                movers
            },
            Black => {
                let black_kings = self.blacks & self.kings;

                let mut movers = 0;
                let mut temp = (not_occupied >> 4) & self.whites;

                movers |= ((temp & MASK_L3) >> 3) | ((temp & MASK_L5) >> 5);

                temp = (((not_occupied & MASK_L3) >> 3) | ((not_occupied & MASK_L5) >> 5)) & self.whites;
                movers |= temp >> 4;

                movers &= self.blacks;

                if black_kings != 0 {
                    temp = (not_occupied << 4) & self.whites;
                    movers |= (((temp & MASK_R3) << 3) | ((temp & MASK_R5) << 5)) & black_kings;
                    temp = (((not_occupied & MASK_R3) << 3) | ((not_occupied & MASK_R5) << 5)) & self.whites;
                    movers |= (temp << 4) & black_kings;
                }

                movers
            }
        }
    }

    /// Creates string FEN tag according to Portable Draughts Notation (PDN). Read more
    /// about the notation [here](https://en.wikipedia.org/wiki/Portable_Draughts_Notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::bitboard::Bitboard;
    ///
    /// let b = Bitboard::new();
    /// assert_eq!(b.fen(), "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12");
    /// ```
    pub fn fen(&self) -> String {
        let mut out = String::new();

        // turn
        out.push(match self.turn {
            Black => 'B',
            White => 'W',
        });

        let write_pieces = |mut it: Mask, out: &mut String| {
            let mut pos = 1;
            while it != 0 {
                if it % 2 == 1 {
                    if (self.kings >> (pos - 1)) % 2 == 1 {
                        out.push('K');
                    }
                    out.push_str(&pos.to_string());
                    out.push(',');
                }
                it = it >> 1;
                pos += 1;
            }
            out.pop();  // remove unnecessary last comma
        };

        out.push_str(":W");
        write_pieces(self.whites, &mut out);

        out.push_str(":B");
        write_pieces(self.blacks, &mut out);

        out
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
                blacks_iter = blacks_iter >> 1;
                whites_iter = whites_iter >> 1;
                kings_iter = kings_iter >> 1;
            }
            out.push_str("|\n");
        }

        out.push_str("+---+---+---+---+---+---+---+---+");

        out
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    static DEFAULT_BOARD: &'static str = "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12";
    static TEST_BOARD_1: &'static str = "B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29";
    static TEST_BOARD_2: &'static str = "W:W9,K11,19,K26,27,30:B15,22,25,K32";
    static TEST_BOARD_3: &'static str = "B:WK3,11,23,25,26,27:B6,7,8,18,19,21,K31";
    static TEST_BOARD_4: &'static str = "B:WK11,3:B";
    static TEST_BOARD_5: &'static str = "W:B:W";
    static TEST_BOARD_6: &'static str = "W:B11:W6";
    static TEST_BOARD_7: &'static str = "B:W11,18,26,27:B8";

    #[test]
    fn new_from_fen_test() {
        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.blacks, 0x11288800);
        assert_eq!(board.whites, 0x0c824200);
        assert_eq!(board.kings, 0x11204200);
        assert_eq!(board.turn, Black);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.blacks, 0x81204000);
        assert_eq!(board.whites, 0x26040500);
        assert_eq!(board.kings, 0x82000400);
        assert_eq!(board.turn, White);

        let board = Bitboard::new_from_fen(TEST_BOARD_4).unwrap();
        assert_eq!(board.blacks, 0);
        assert_eq!(board.whites, 0x00000404);
        assert_eq!(board.kings, 0x00000400);
        assert_eq!(board.turn, Black);
    }

    #[test]
    fn fen_test() {
        let board = Bitboard::new();
        assert_eq!(board.fen(), DEFAULT_BOARD);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.fen(), "B:WK10,K15,18,24,27,28:B12,16,20,K22,K25,K29");
    }

    #[test]
    fn get_movers_white_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_movers(&White), 0x00f00000);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(&White), 0x04824200);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(&White), 0x06040500);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(&White), 0x07000000);
    }

    #[test]
    fn get_movers_black_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_movers(&Black), 0x00000f00);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(&Black), 0x01208000);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(&Black), 0x81004000);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(&Black), 0x000600e0);
    }

    #[test]
    fn get_jumpers_white_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_jumpers(&White), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(&White), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(&White), 0x22040400);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(&White), 0x00400404);
    }

    #[test]
    fn get_jumpers_black_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_jumpers(&Black), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0x80004000);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0x401000c0);  // this one is failing
    }

    #[test]
    fn get_game_state_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_game_state(), GameState::InProgress);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_game_state(), GameState::InProgress);

        let board = Bitboard::new_from_fen(TEST_BOARD_4).unwrap();
        assert_eq!(board.get_game_state(), GameState::Completed(Winner::Player(White)));

        let board = Bitboard::new_from_fen(TEST_BOARD_5).unwrap();
        assert_eq!(board.get_game_state(), GameState::Completed(Winner::Player(Black)));
    }

    #[test]
    fn validate_action_move_test() {
        let board = Bitboard::new();
        let action = Action::new_from_movetext("10-14").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::new_from_movetext("23-18").unwrap();
        assert_eq!(board.validate_action(&action), Err("Source must be occupied by the actor!"));

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        let action = Action::new_from_movetext("16-19").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::new_from_movetext("22-17").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::new_from_movetext("12-8").unwrap();
        assert_eq!(board.validate_action(&action), Err("Only kings can move backwards!"));
        let action = Action::new_from_movetext("22-18").unwrap();
        assert_eq!(board.validate_action(&action), Err("Destination must be empty!"));

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        let action = Action::new_from_movetext("9-6").unwrap();
        assert_eq!(board.validate_action(&action), Err("Have to jump a piece!"));
    }

    #[test]
    fn take_action_move_test() {
        let board = Bitboard::new();
        let action = Action::new_from_movetext("10-14").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.whites, 0xfff00000);
        assert_eq!(board_p.blacks, 0x00002dff);
        assert_eq!(board_p.kings, 0);
        assert_eq!(board_p.turn, White);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        let action = Action::new_from_movetext("16-19").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x112c0800);
        assert_eq!(board_p.whites, 0x0c824200);
        assert_eq!(board_p.kings, 0x11204200);

        // king moving backwards
        let action = Action::new_from_movetext("22-17").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x11098800);
        assert_eq!(board_p.whites, 0x0c824200);
        assert_eq!(board_p.kings, 0x11014200);
        
        // kinging of single piece
        let board = Bitboard::new_from_fen(TEST_BOARD_6).unwrap();
        let action = Action::new_from_movetext("6-2").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x00000400);
        assert_eq!(board_p.whites, 0x00000002);
        assert_eq!(board_p.kings, 0x00000002);
        assert_eq!(board_p.turn, Black);
    }

    #[test]
    fn validate_action_jump_test() {
        // need to show failure for jumping over own piece and blank space
        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        let action = Action::new_from_movetext("30-21").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::new_from_movetext("30-23").unwrap();
        assert_eq!(board.validate_action(&action), Err("You can only skip over a square with an opponent on it!"));
        let action = Action::new_from_movetext("27-20").unwrap();
        assert_eq!(board.validate_action(&action), Err("You can only skip over a square with an opponent on it!"));

        let board = Bitboard::new_from_fen(TEST_BOARD_7).unwrap();
        let action = Action::new_from_movetext("8-15-22").unwrap();
        assert_eq!(board.validate_action(&action), Err("Need to keep jumping!"));  // this is wrong
        let action = Action::new_from_movetext("8-15-22-31").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::new_from_movetext("8-15-22-31-24").unwrap();
        assert_eq!(board.validate_action(&action), Err("Only kings can move backwards!"));
    }

    #[test]
    fn take_action_jump_test() {
        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        let action = Action::new_from_movetext("11-18").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x81200000);
        assert_eq!(board_p.whites, 0x26060100);
        assert_eq!(board_p.kings, 0x82020000);

        let action = Action::new_from_movetext("19-10").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x81200000);
        assert_eq!(board_p.whites, 0x26000700);
        assert_eq!(board_p.kings, 0x82000400);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        let action = Action::new_from_movetext("21-30").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x600600e0);
        assert_eq!(board_p.whites, 0x06400404);
        assert_eq!(board_p.kings, 0x60000004);

        let board = Bitboard::new_from_fen(TEST_BOARD_7).unwrap();
        let action = Action::new_from_movetext("8-15-22-31").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x40000000);
        assert_eq!(board_p.whites, 0x04000000);
        assert_eq!(board_p.kings, 0x40000000);
    }
}