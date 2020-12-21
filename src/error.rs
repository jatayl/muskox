use snafu::Snafu;

use crate::board::Color;

#[derive(Debug, PartialEq, Snafu)]
pub enum ActionError {
    #[snafu(display("Source position {} must be in possession of mover {:?}", position, color))]
    SourceColorError { position: u8, color: Color },

    #[snafu(display("Destination position {} must be empty", destination))]
    DestinationEmptyError { destination: u8 },

    #[snafu(display("Skipped position {} must be have opponent of color {:?}", skipped, color))]
    SkippedPositionError { skipped: u8, color: Color },

    #[snafu(display("One of the jumpers need to move!"))]
    HaveToJumpError,

    #[snafu(display("Only kings can move backwards!"))]
    SinglePieceBackwardsError,

    #[snafu(display("More jumping required!"))]
    NeedMoreJumpingError,
}

#[derive(Debug, Snafu)]
pub enum ParseBoardError {
    #[snafu(display("There should be two colons ':' in the FEN string"))]
    ColonQuantityError,

    #[snafu(display("{} is not a valid board color (Black 'B' or White 'W'", letter))]
    ColorError { letter: String },

    #[snafu(display("{} is not a valid position 1 - 32", position))]
    PositionError { position: String },
}

#[derive(Debug, Snafu)]
pub enum ParseActionError {
    #[snafu(display("Number of moves must be 1 and 8 and not {}", quantity))]
    MoveQuantityError { quantity: usize },

    #[snafu(display("Position {} is invalid", position))]
    PositionValueError { position: String },
}