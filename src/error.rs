// could be easiest to write this out myself and not use fail
// do this for now though
use failure::Fail;

use crate::bitboard::Color;

#[derive(Debug, Fail, PartialEq)]
pub enum ActionError {
    #[fail(display = "Source position {} must be in possession of mover {:?}", source, color)]
    SourceColorError { source: u8, color: Color },
    #[fail(display = "Destination position {} must be empty", destination)]
    DestinationEmptyError { destination: u8 },
    #[fail(display = "Skipped position {} must be have opponent of color {:?}", skipped, color)]
    SkippedPositionError { skipped: u8, color: Color },
    #[fail(display = "One of the jumpers needs to be moved")]
    HaveToJumpError,
    #[fail(display = "Only kings can move backwards")]
    SinglePieceBackwardsError,
    #[fail(display = "More jumping required")]
    NeedMoreJumpingError,
}

#[derive(Debug, Fail)]
pub enum ParseBoardError {
    #[fail(display = "There should be two colons ':' in the FEN string")]
    ColonQuantityError,
    #[fail(display = "{} is not a valid board color (Black 'B' or White 'W'", letter)]
    ColorError { letter: String },
    #[fail(display = "{} is not a valid position 1 - 32", position)]
    PositionError { position: String },
}

#[derive(Debug, Fail)]
pub enum ParseActionError {
    #[fail(display = "Number of moves must be 1 and 8 and not {}", quantity)]
    MoveQuantityError { quantity: usize },
    #[fail(display = "Position {} is invalid", position)]
    PositionValueError { position: String },
}