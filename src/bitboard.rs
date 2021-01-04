use std::mem;
use std::default;
use std::collections::VecDeque;

use crate::board::{Action, ActionType, Direction};
use crate::error::{ActionError, ParseBoardError};
use crate::search::{Searchable, Optim, ActionStatePair, GameState, Winner, Side};
use crate::zobrist;

type Mask = u32;

// these values need rigorous testing to ensure they are right
// many problems have arisen from these.
const MASK_L3: Mask = 0x07070707;
const MASK_L5: Mask = 0xe0e0e0e0;
const MASK_R3: Mask = 0xe0e0e0e0;
const MASK_R5: Mask = 0x07070707;

// maybe want to consider making a piece enum so we can abstract the king

/// Represents of the two colors that exists on a checkerboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    White,
}
use Color::*;

impl Side for Color {
    fn optim(&self) -> Optim {
        match self {
            White => Optim::Min,
            Black => Optim::Max,
        }
    }
}

/// Represents a single state of a checkerboard
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bitboard {
    blacks: Mask,
    whites: Mask,
    kings: Mask,
    turn: Color,
}

impl default::Default for Bitboard {
    /// Creates a new bitboard in the default, starting position
    fn default() -> Self {
        // initial state for a blank board
        Bitboard {
            blacks: 0x00000fff,
            whites: 0xfff00000,
            kings: 0,
            turn: Black,
        }
    }
}

impl Bitboard {
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
    /// use muskox::board::Bitboard;
    ///
    /// let board = Bitboard::from_fen("B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29");
    /// // will put proof that it works here
    /// ```
    pub fn from_fen(fen_string: &str) -> Result<Self, ParseBoardError> {
        // maybe clean up errors handling here. they are rather rigid could make them more useful

        let fen_string: String = fen_string.chars().filter(|c| !c.is_whitespace()).collect();

        let d: Vec<_> = fen_string.split(":").collect();

        if d.len() != 3 {
            return Err(ParseBoardError::ColonQuantityError);
        }

        let turn = match d[0] {
            "B" => Black,
            "W" => White,
            ltr => return Err(ParseBoardError::ColorError { letter: ltr.to_string() }),
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
                    Some('K') => (piece.chars().skip(1).collect::<String>().parse::<u8>().or_else(|_|
                        Err(ParseBoardError::PositionError { position: piece.to_string() }))? - 1, true),
                    Some(_) => (piece.parse::<u8>().or_else(|_|
                        Err(ParseBoardError::PositionError { position: piece.to_string() }))? - 1, false),
                    None => break,  // if no pieces exist. not super pretty or clear like this...
                };

                if piece_n > 31 {  // already had been converted to computer numbers
                    return Err(ParseBoardError::PositionError { position: piece.to_string() });
                }

                temp |= 1 << piece_n;

                if is_king {
                    kings |= 1 << piece_n;
                }
            }

            // match the color to assign it to
            match position.chars().next()
                .ok_or(ParseBoardError::PositionError { position: "?".to_string() })? {
                'B' => blacks |= temp,
                'W' => whites |= temp,
                ltr => return Err(ParseBoardError::ColorError { letter: ltr.to_string() }),
            };
        }

        Ok(Bitboard{ blacks, whites, kings, turn })
    }

    /// Determines whether a certain action is valid or not.
    ///
    /// This information is encoded in a rust `Result`. If the action is valid, `Ok(())`
    /// is returned; if not, then the `Err` option is returned wrapping
    /// [ActionError](crate.error.enum.ActionError.html) with the reason for error.
    ///
    /// # Arguments
    ///
    /// * `action` - An [action](muskox/action/struct.Action.html) representing the particular
    /// move to validate
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::{Bitboard, Action};
    /// use muskox::error::ActionError;
    ///
    /// let board = Bitboard::from_fen("B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29").unwrap();
    ///
    /// let action = Action::from_movetext("22-17").unwrap();
    /// assert_eq!(board.validate_action(&action), Ok(()));
    ///
    /// let action = Action::from_movetext("12-8").unwrap();
    /// assert_eq!(board.validate_action(&action), Err(ActionError::SinglePieceBackwardsError));
    /// ```
    pub fn validate_action(&self, action: &Action) -> Result<(), ActionError> {
        self.take_action(&action)?;
        Ok(())
    }

    /// Creates string FEN tag according to Portable Draughts Notation (PDN). Read more
    /// about the notation [here](https://en.wikipedia.org/wiki/Portable_Draughts_Notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::Bitboard;
    ///
    /// let b = Bitboard::default();
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
    /// use muskox::board::Bitboard;
    ///
    /// let board = Bitboard::default();
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

    /// Returns a u32 mask that represents all of the white pieces that can move.
    /// Recognize that this does not include the white pieces that can jump. To
    /// access those use `get_jumpers`.
    fn get_movers(&self, color: &Color) -> Mask {
        let not_occupied = !(self.whites | self.blacks);

        match color {
            White => {
                let white_kings = self.whites & self.kings;

                let mut movers = not_occupied << 4;

                movers |= (not_occupied & MASK_R3) << 3;
                movers |= (not_occupied & MASK_R5) << 5;
                movers &= self.whites;

                if white_kings != 0 {
                    movers |= (not_occupied >> 4) & white_kings;
                    movers |= ((not_occupied & MASK_L3) >> 3) & white_kings;
                    movers |= ((not_occupied & MASK_L5) >> 5) & white_kings;
                }

                movers
            },
            Black => {
                let black_kings = self.blacks & self.kings;

                let mut movers = not_occupied >> 4;

                movers |= (not_occupied & MASK_L3) >> 3;
                movers |= (not_occupied & MASK_L5) >> 5;
                movers &= self.blacks;

                if black_kings != 0 {
                    movers |= (not_occupied << 4) & black_kings;
                    movers |= ((not_occupied & MASK_R3) << 3) & black_kings;
                    movers |= ((not_occupied & MASK_R5) << 5) & black_kings;
                }

                movers
            }
        }
    }

    /// Returns a u32 mask that represents all of the pieces of a certain color that can
    /// jump. Recognize that this does not include the white pieces that can move. To
    /// access those use `get_movers`.
    fn get_jumpers(&self, color: &Color) -> Mask {
        // not picking up moves forward left

        let not_occupied = !(self.whites | self.blacks);

        match color {
            White => {
                let white_kings = self.whites & self.kings;

                let mut jumpers = 0;
                let mut temp = (not_occupied << 4) & self.blacks;

                jumpers |= ((temp & MASK_R3) << 3) | ((temp & MASK_R5) << 5);

                temp = (((not_occupied & MASK_R3) << 3) | ((not_occupied & MASK_R5) << 5)) & self.blacks;
                jumpers |= temp << 4;

                jumpers &= self.whites;

                if white_kings != 0 {
                    temp = (not_occupied >> 4) & self.blacks;
                    jumpers |= (((temp & MASK_L3) >> 3) | ((temp & MASK_L5) >> 5)) & white_kings;
                    temp = (((not_occupied & MASK_L3) >> 3) | ((not_occupied & MASK_L5) >> 5)) & self.blacks;
                    jumpers |= (temp >> 4) & white_kings;
                }

                jumpers
            },
            Black => {
                let black_kings = self.blacks & self.kings;

                let mut jumpers = 0;
                let mut temp = (not_occupied >> 4) & self.whites;

                jumpers |= ((temp & MASK_L3) >> 3) | ((temp & MASK_L5) >> 5);

                temp = (((not_occupied & MASK_L3) >> 3) | ((not_occupied & MASK_L5) >> 5)) & self.whites;
                jumpers |= temp >> 4;

                jumpers &= self.blacks;

                if black_kings != 0 {
                    temp = (not_occupied << 4) & self.whites;
                    jumpers |= (((temp & MASK_R3) << 3) | ((temp & MASK_R5) << 5)) & black_kings;
                    temp = (((not_occupied & MASK_R3) << 3) | ((not_occupied & MASK_R5) << 5)) & self.whites;
                    jumpers |= (temp << 4) & black_kings;
                }

                jumpers
            }
        }
    }

    /// Retrives all of the possible next positions from a certain position given a particular action type
    fn next_position_possibilities(&self, position: u8, action_type: &ActionType) -> Vec<u8> {
        let mut directions = match self.turn {
            White => vec![Direction::UpLeft, Direction::UpRight],
            Black => vec![Direction::DownLeft, Direction::DownRight],
        };

        if self.is_king(position) {
            directions.extend(match self.turn {
                White => vec![Direction::DownLeft, Direction::DownRight],
                Black => vec![Direction::UpLeft, Direction::UpRight],
            });
        }

        let opponent_color: Color = unsafe { mem::transmute((self.turn as u8 + 1) % 2) };

        directions.iter()
            .filter_map(|d| {
                match action_type {
                    ActionType::Move => d.relative_to(position),
                    ActionType::Jump => d.relative_jump_from(position),
                }
            })
            .filter(|&p| {
                // check to ensure we are only jumping over opponents pieces
                // this is an inefficient way of getting dir..
                if *action_type == ActionType::Jump {
                    let dir = Direction::between(position, p).unwrap();
                    if !self.coloring_eq(dir.relative_to(position).unwrap(), &opponent_color) {
                        return false;
                    }
                }
                true
            })
            .filter(|&p| self.is_empty(p))  // must be landing in empty spot
            .collect::<Vec<_>>()
    }

    /// Returns whether a given position is empty or not
    #[inline]
    fn is_empty(&self, position: u8) -> bool {
        (self.whites >> position) % 2 == 0 && (self.blacks >> position) % 2 == 0
    }

    /// Returns whether a given position has a king or not
    #[inline]
    fn is_king(&self, position: u8) -> bool {
        (self.kings >> position) % 2 == 1
    }

    /// Returns whether or not a position has a particular color or not
    #[inline]
    fn coloring_eq(self, position: u8, color: &Color) -> bool {
        let color_mask = match color {
            White => self.whites,
            Black => self.blacks,
        };
        (color_mask >> position) % 2 == 1
    }

    /// Removes the piece from the given position on the board. Note that this mutates the board
    #[inline]
    fn remove_piece(&mut self, position: u8) {
        let mask = !(1 << position);
        self.whites &= mask;
        self.blacks &= mask;
        self.kings &= mask;
    }

    /// Adds a piece to a board. Note that this mutates the board
    #[inline]
    fn add_piece(&mut self, position: u8, color: &Color, is_king: bool) {
        let mask = 1 << position;
        match color {
            Black => self.blacks |= mask,
            White => self.whites |= mask,
        };
        if is_king {
            self.kings |= mask;
        }
    }

    #[inline]
    pub fn blacks(&self) -> Mask {
        self.blacks
    }

    #[inline]
    pub fn whites(&self) -> Mask {
        self.whites
    }

    #[inline]
    pub fn kings(&self) -> Mask {
        self.kings
    }
}

pub struct ActionBitboardPair {
    action: Action,
    board: Bitboard,
}

impl ActionBitboardPair {
    #[inline]
    pub fn action(&self) -> Action {
        self.action
    }

    #[inline]
    pub fn board(&self) -> Bitboard {
        self.board
    }
}



impl Searchable for Bitboard {
    type Action = Action;
    type Side = Color;

    /// Returns a [GameState](enum.GameState.html) enum that contains information about the
    /// state of the game on the current bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::Bitboard;
    /// use muskox::search::{GameState, Searchable};
    ///
    /// let board = Bitboard::default();
    /// assert_eq!(board.get_game_state(), GameState::InProgress);
    /// ```
    fn get_game_state(&self) -> GameState<Bitboard> {
        // check if somebody can't move
        if self.turn == Black && self.get_movers(&Black) == 0 && self.get_jumpers(&Black) == 0 {
            return GameState::Completed(Winner::Player(White));
        }
        if self.turn == White && self.get_movers(&White) == 0 && self.get_jumpers(&White) == 0 {
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

    fn generate_all_actions(&self) -> Vec<ActionStatePair<Bitboard>> {
        // returns the next piece to check moves for
        let pop_piece = |mask: &mut Mask, color: &Color| {
            let position = match *color {
                White => mask.trailing_zeros(),
                Black => (0x80000000 as u32 >> mask.leading_zeros()).trailing_zeros(),
            };
            *mask ^= 1 << position;
            position
        };

        if let GameState::Completed(_) = self.get_game_state() {
            return Vec::new();
        }

        let jumpers = self.get_jumpers(&self.turn);

        let action_type = match jumpers {
            0 => ActionType::Move,
            _ => ActionType::Jump,
        };

        let mut actions = Vec::new();

        // could check tree based search performance
        match action_type {
            ActionType::Move => {
                // append all of the possible moves
                let mut movers = self.get_movers(&self.turn);

                while movers != 0 {
                    let mover = pop_piece(&mut movers, &self.turn);

                    let base_action = vec![mover as u8];

                    let move_candidates = self.next_position_possibilities(mover as u8, &action_type);

                    let starts_as_king = self.is_king(mover as u8);

                    for candidate in move_candidates {
                        let mut action = base_action.clone();
                        action.push(candidate);
                        let action: Vec<_> = action.iter().map(|x| (x + 1) as u8).collect();
                        let action = Action::from_vector(action).unwrap();

                        let ends_as_king = {
                            let dest_row = candidate / 4;
                            // will be a king if it was a king or will be in end row last
                            starts_as_king || dest_row == 0 || dest_row == 7
                        };

                        let mut board_p = self.clone();

                        // apply move on pieces
                        board_p.add_piece(candidate, &self.turn, ends_as_king);
                        board_p.remove_piece(mover as u8);
                        board_p.turn = unsafe { mem::transmute((self.turn as u8 + 1) % 2) };

                        // find the zobrist hash of this action
                        let mut zobrist_hash = zobrist::get_position_hash(mover, self.turn, starts_as_king);
                        zobrist_hash ^= zobrist::get_position_hash(candidate as u32, self.turn, ends_as_king);
                        zobrist_hash ^= zobrist::get_turn_hash();

                        actions.push(ActionStatePair::new(action, board_p, zobrist_hash));
                    }
                }
            },
            ActionType::Jump => {
                // keep a running queue until it is empty
                let mut boards_in_progress = VecDeque::new();

                // set up the initial pieces to check.
                let mut jumpers = self.get_jumpers(&self.turn);

                while jumpers != 0 {
                    let position = pop_piece(&mut jumpers, &self.turn);

                    let base_action = vec![position as u8];

                    boards_in_progress.push_back((self.clone(), base_action, 0));
                }

                while let Some((board, base_action, mut zobrist_hash)) = boards_in_progress.pop_front() {
                    // can only pop the piece that has been jumping [last element in action]
                    let &jumper = base_action.last().unwrap();

                    // generate all possible new boards based on jumpers.
                    let jump_candidates = self.next_position_possibilities(jumper, &action_type);

                    for candidate in jump_candidates {
                        let mut action_vec = base_action.clone();
                        action_vec.push(candidate);
                        let action = Action::from_vector(action_vec.iter().map(|x| (x + 1) as u8).collect()).unwrap();

                        let direction = Direction::between(jumper, candidate).unwrap();

                        let skipped_over = direction.relative_to(jumper).unwrap();

                        let starts_as_king = board.is_king(jumper);

                        let ends_as_king = {
                            let dest_row = candidate / 4;
                            // will be a king if it was a king or will be in end row last
                            starts_as_king || dest_row == 0 || dest_row == 7
                        };

                        // apply jump on piece
                        let mut board_p = board.clone();
                        board_p.add_piece(candidate, &board.turn, ends_as_king);
                        board_p.remove_piece(jumper);
                        board_p.remove_piece(skipped_over);
                        board_p.turn = unsafe { mem::transmute((self.turn as u8 + 1) % 2) };

                        // make the zobrist hash
                        zobrist_hash ^= zobrist::get_position_hash(jumper as u32, board.turn, starts_as_king);
                        zobrist_hash ^= zobrist::get_position_hash(candidate as u32, board.turn, ends_as_king);
                        // use board_p.turn below it is set to next turn
                        zobrist_hash ^= zobrist::get_position_hash(skipped_over as u32, board_p.turn, board.is_king(skipped_over));


                        // check if we cannot jump anymore
                        if (board_p.get_jumpers(&board.turn) & (1 << candidate) == 0) | (!starts_as_king & ends_as_king) {
                            // finally add the turn hash when it is over
                            zobrist_hash ^= zobrist::get_turn_hash();

                            actions.push(ActionStatePair::new(action, board_p, zobrist_hash));
                            continue;
                        }
                        // other wise put it in the deque
                        boards_in_progress.push_back((board_p, action_vec, zobrist_hash));
                    }
                }
            },
        }

        actions
    }

    /// Returns the ensuing bitboard after making a particular action by a player.
    ///
    /// This information is also encoded in a rust `Result`. If the action is valid, `Ok`
    /// wrapping the resultant bitboard is returned; if not, then the `Err` option is returned
    /// wrapping [ActionError](crate.error.enum.ActionError.html) with the reason for error.
    ///
    /// # Arguments
    ///
    /// * `action` - An [action](muskox/action/struct.Action.html) representing the particular
    /// move to take
    ///
    /// # Examples
    ///
    /// ```
    /// use muskox::board::{Bitboard, Action};
    /// use muskox::search::Searchable;
    /// use muskox::error::ActionError;
    ///
    /// let board = Bitboard::from_fen("B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29").unwrap();
    ///
    /// let action = Action::from_movetext("22-17").unwrap();
    /// assert_eq!(board.take_action(&action).unwrap().fen(), "W:WK10,K15,18,24,27,28:B12,16,K17,20,K25,K29");
    ///
    /// let action = Action::from_movetext("12-8").unwrap();
    /// assert_eq!(board.validate_action(&action), Err(ActionError::SinglePieceBackwardsError));
    /// ```
    fn take_action(&self, action: &Action) -> Result<Bitboard, ActionError> {
        let mut board_p = self.clone();

        let source = action.source();
        let destination = action.destination();

        let starts_as_king = self.is_king(source);

        let ends_as_king = {
            let dest_row = destination / 4;
            // will be a king if it was a king or will be in end row last
            starts_as_king || dest_row == 0 || dest_row == 7
        };

        // sketchy way of flipping the turn color enum
        // maybe just match with the opposite color instead
        let opponent_color: Color = unsafe { mem::transmute((self.turn as u8 + 1) % 2) };

        // erase color from source
        board_p.remove_piece(source);

        // add color to destination
        board_p.add_piece(destination, &self.turn, ends_as_king);

        // ensure that the source has turn color
        if !self.coloring_eq(source, &self.turn) {
            let color = self.turn;
            return Err(ActionError::SourceColorError { position: source, color: color });
        }

        // ensure that destination is empty.
        if !self.is_empty(destination) {
            return Err(ActionError::DestinationEmptyError { destination });
        }

        match action.action_type() {
            ActionType::Move => {
                // ensure that no jumpers are available
                if self.get_jumpers(&self.turn) != 0 {
                    return Err(ActionError::HaveToJumpError);
                }

                // ensure that it only moves backwards if source is a king
                // reformat the next dozen or so lines
                let move_direction = action.move_direction().unwrap();

                if (move_direction == Direction::UpLeft || move_direction == Direction::UpRight) &&
                        self.turn == Black && !self.is_king(source) {
                    return Err(ActionError::SinglePieceBackwardsError);
                }

                if (move_direction == Direction::DownLeft || move_direction == Direction::DownRight) &&
                        self.turn == White && !self.is_king(source) {
                    return Err(ActionError::SinglePieceBackwardsError);
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
                        return Err(ActionError::SinglePieceBackwardsError);
                    }

                    if (jump_direction == Direction::DownLeft || jump_direction == Direction::DownRight) &&
                            self.turn == White && !starts_as_king {
                        return Err(ActionError::SinglePieceBackwardsError);
                    }

                    // see if we can try to use bitmasking to get this next time
                    let skipped_over = jump_direction.relative_to(curr).unwrap();

                    // ensure that it actually jumps over another piece that is not its own color
                    if !self.coloring_eq(skipped_over, &opponent_color) {
                        return Err(ActionError::SkippedPositionError {
                            skipped: skipped_over,
                            color: opponent_color,
                        });
                    }

                    board_p.remove_piece(skipped_over);

                    curr = jump_direction.relative_jump_from(curr).unwrap();
                }
                // ensure that it there isnt another jump for it to do at destination
                if (board_p.get_jumpers(&self.turn) & 1 << destination != 0) & !(!starts_as_king & ends_as_king) {
                    return Err(ActionError::NeedMoreJumpingError);
                }
            }
        }

        board_p.turn = opponent_color;

        Ok(board_p)
    }

    #[inline]
    fn turn(&self) -> Color {
        self.turn
    }

    fn zobrist_hash(&self) -> u64 {
        // returns the next piece to check moves for
        let pop_piece = |mask: &mut Mask, color: &Color| {
            let position = match *color {
                White => mask.trailing_zeros(),
                Black => (0x80000000 as u32 >> mask.leading_zeros()).trailing_zeros(),
            };
            *mask ^= 1 << position;
            position
        };

        let mut zobrist_hash = 0;

        let mut blacks_iter = self.blacks;
        let mut whites_iter = self.whites;

        while blacks_iter != 0 {
            let position = pop_piece(&mut blacks_iter, &Black);
            zobrist_hash ^= zobrist::get_position_hash(position, Black, self.is_king(position as u8));
        }

        while whites_iter != 0 {
            let position = pop_piece(&mut whites_iter, &White);
            zobrist_hash ^= zobrist::get_position_hash(position, White, self.is_king(position as u8));
        }

        // if it is white then start it off with the turn hash
        if self.turn == Color::White {
            zobrist_hash ^= zobrist::get_turn_hash()
        }

        zobrist_hash
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_BOARD: &'static str = "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12";
    const TEST_BOARD_1: &'static str = "B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29";
    const TEST_BOARD_2: &'static str = "W:W9,K11,19,K26,27,30:B15,22,25,K32";
    const TEST_BOARD_3: &'static str = "B:WK3,11,23,25,26,27:B6,7,8,18,19,21,K31";
    const TEST_BOARD_4: &'static str = "B:WK11,3:B";
    const TEST_BOARD_5: &'static str = "W:B:W";
    const TEST_BOARD_6: &'static str = "W:B11:W6";
    const TEST_BOARD_7: &'static str = "B:W11,18,26,27:B8";

    #[test]
    fn from_fen_test() {
        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.blacks, 0x11288800);
        assert_eq!(board.whites, 0x0c824200);
        assert_eq!(board.kings, 0x11204200);
        assert_eq!(board.turn, Black);

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.blacks, 0x81204000);
        assert_eq!(board.whites, 0x26040500);
        assert_eq!(board.kings, 0x82000400);
        assert_eq!(board.turn, White);

        let board = Bitboard::from_fen(TEST_BOARD_4).unwrap();
        assert_eq!(board.blacks, 0);
        assert_eq!(board.whites, 0x00000404);
        assert_eq!(board.kings, 0x00000400);
        assert_eq!(board.turn, Black);
    }

    #[test]
    fn fen_test() {
        let board = Bitboard::default();
        assert_eq!(board.fen(), DEFAULT_BOARD);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.fen(), "B:WK10,K15,18,24,27,28:B12,16,20,K22,K25,K29");
    }

    #[test]
    fn get_movers_white_test() {
        let board = Bitboard::default();
        assert_eq!(board.get_movers(&White), 0x00f00000);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(&White), 0x04824200);

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(&White), 0x06040500);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(&White), 0x07000000);
    }

    #[test]
    fn get_movers_black_test() {
        let board = Bitboard::default();
        assert_eq!(board.get_movers(&Black), 0x00000f00);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_movers(&Black), 0x01208000);

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_movers(&Black), 0x81004000);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_movers(&Black), 0x000600e0);
    }

    #[test]
    fn get_jumpers_white_test() {
        let board = Bitboard::default();
        assert_eq!(board.get_jumpers(&White), 0);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(&White), 0);

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(&White), 0x22040400);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(&White), 0x00400404);
    }

    #[test]
    fn get_jumpers_black_test() {
        let board = Bitboard::default();
        assert_eq!(board.get_jumpers(&Black), 0);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0);

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0x80204000);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_jumpers(&Black), 0x401000c0);
    }

    #[test]
    fn get_game_state_test() {
        let board = Bitboard::default();
        assert_eq!(board.get_game_state(), GameState::InProgress);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        assert_eq!(board.get_game_state(), GameState::InProgress);

        let board = Bitboard::from_fen(TEST_BOARD_4).unwrap();
        assert_eq!(board.get_game_state(), GameState::Completed(Winner::Player(White)));

        let board = Bitboard::from_fen(TEST_BOARD_5).unwrap();
        assert_eq!(board.get_game_state(), GameState::Completed(Winner::Player(Black)));
    }

    #[test]
    fn validate_action_move_test() {
        let board = Bitboard::default();
        let action = Action::from_movetext("10-14").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::from_movetext("23-18").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::SourceColorError { position: 22, color: Black }));

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        let action = Action::from_movetext("16-19").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::from_movetext("22-17").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::from_movetext("12-8").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::SinglePieceBackwardsError));
        let action = Action::from_movetext("22-18").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::DestinationEmptyError { destination: 17 }));

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        let action = Action::from_movetext("9-6").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::HaveToJumpError));
    }

    #[test]
    fn take_action_move_test() {
        let board = Bitboard::default();
        let action = Action::from_movetext("10-14").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.whites, 0xfff00000);
        assert_eq!(board_p.blacks, 0x00002dff);
        assert_eq!(board_p.kings, 0);
        assert_eq!(board_p.turn, White);

        let board = Bitboard::from_fen(TEST_BOARD_1).unwrap();
        let action = Action::from_movetext("16-19").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x112c0800);
        assert_eq!(board_p.whites, 0x0c824200);
        assert_eq!(board_p.kings, 0x11204200);

        // king moving backwards
        let action = Action::from_movetext("22-17").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x11098800);
        assert_eq!(board_p.whites, 0x0c824200);
        assert_eq!(board_p.kings, 0x11014200);
        
        // kinging of single piece
        let board = Bitboard::from_fen(TEST_BOARD_6).unwrap();
        let action = Action::from_movetext("6-2").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x00000400);
        assert_eq!(board_p.whites, 0x00000002);
        assert_eq!(board_p.kings, 0x00000002);
        assert_eq!(board_p.turn, Black);
    }

    #[test]
    fn validate_action_jump_test() {
        // need to show failure for jumping over own piece and blank space
        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        let action = Action::from_movetext("30-21").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::from_movetext("30-23").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::SkippedPositionError { skipped: 25, color: Black }));
        let action = Action::from_movetext("27-20").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::SkippedPositionError { skipped: 23, color: Black }));

        let board = Bitboard::from_fen(TEST_BOARD_7).unwrap();
        let action = Action::from_movetext("8-15-22").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::NeedMoreJumpingError));  // this is wrong
        let action = Action::from_movetext("8-15-22-31").unwrap();
        assert_eq!(board.validate_action(&action), Ok(()));
        let action = Action::from_movetext("8-15-22-31-24").unwrap();
        assert_eq!(board.validate_action(&action), Err(ActionError::SinglePieceBackwardsError));
    }

    #[test]
    fn take_action_jump_test() {
        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();
        let action = Action::from_movetext("11-18").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x81200000);
        assert_eq!(board_p.whites, 0x26060100);
        assert_eq!(board_p.kings, 0x82020000);

        let action = Action::from_movetext("19-10").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x81200000);
        assert_eq!(board_p.whites, 0x26000700);
        assert_eq!(board_p.kings, 0x82000400);

        let board = Bitboard::from_fen(TEST_BOARD_3).unwrap();
        let action = Action::from_movetext("21-30").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x600600e0);
        assert_eq!(board_p.whites, 0x06400404);
        assert_eq!(board_p.kings, 0x60000004);

        let board = Bitboard::from_fen(TEST_BOARD_7).unwrap();
        let action = Action::from_movetext("8-15-22-31").unwrap();
        let board_p = board.take_action(&action).unwrap();
        assert_eq!(board_p.blacks, 0x40000000);
        assert_eq!(board_p.whites, 0x04000000);
        assert_eq!(board_p.kings, 0x40000000);
    }

    #[test]
    fn zobrist_hashing_test() {
        // checks that the zobrist hashing is consistent with 2 different ways of making it
        let board = Bitboard::from_fen(DEFAULT_BOARD).unwrap();

        for action in board.generate_all_actions() {
            let board_p = action.state();
            let zobrist_diff = *action.zobrist_diff();
            let zobrist_hash = board.zobrist_hash() ^ zobrist_diff;
            assert_eq!(zobrist_hash, board_p.zobrist_hash());
        }

        let board = Bitboard::from_fen(TEST_BOARD_2).unwrap();

        for action in board.generate_all_actions() {
            let board_p = action.state();
            let zobrist_diff = *action.zobrist_diff();
            let zobrist_hash = board.zobrist_hash() ^ zobrist_diff;
            assert_eq!(zobrist_hash, board_p.zobrist_hash());
        }
    }
}