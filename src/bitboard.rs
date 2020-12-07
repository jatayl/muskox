use crate::Action;

type Mask = u32;

static MASK_L3: Mask = 0x07070707;
static MASK_L5: Mask = 0x00e0e0e0;
static MASK_R3: Mask = 0xe0e0e0e0;
static MASK_R5: Mask = 0x07070700;

#[derive(Debug, PartialEq)]
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
        if self.turn == Black && self.get_movers(Black) == 0 && self.get_jumpers(Black) == 0 {
            return GameState::Completed(Winner::Player(White));
        }
        if self.turn == White && self.get_movers(White) == 0 && self.get_jumpers(White) == 0 {
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

    pub fn is_valid_move(&self, _action: &Action) -> bool {
        true
    }

    pub fn make_move(&self, action: &Action) -> Result<Bitboard, &'static str> {
        if !self.is_valid_move(&action) {
            return Err("Invalid move")
        }

        Ok(Bitboard::new())
    }

    /// Returns a u32 mask that represents all of the white pieces that can move.
    /// Recognize that this does not include the white pieces that can jump. To
    /// access those use `get_jumpers_white`.
    fn get_movers(&self, color: Color) -> Mask {
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

    fn get_jumpers(&self, color: Color) -> Mask {
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
                    movers |= (((temp & MASK_L3) >> 3) | ((temp & MASK_L5) >> 5    )) & white_kings;
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
        assert_eq!(board.get_movers(White), 0x00f00000);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(White), 0x04824200);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(White), 0x06040500);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(White), 0x07000000);
    }

    #[test]
    fn get_movers_black_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_movers(Black), 0x00000f00);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(Black), 0x01208000);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(Black), 0x81004000);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(Black), 0x000600e0);
    }

    #[test]
    fn get_jumpers_white_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_jumpers(White), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(White), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(White), 0x22040400);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(White), 0x00400404);
    }

    #[test]
    fn get_jumpers_black_test() {
        let board = Bitboard::new();
        assert_eq!(board.get_jumpers(Black), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(Black), 0);

        let board = Bitboard::new_from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(Black), 0x80004000);

        let board = Bitboard::new_from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(Black), 0x401000c0);  // this one is failing
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
}