use nom::error::{VerboseError, VerboseErrorKind::Context};
use snafu::Snafu;

use crate::board::Color;

#[derive(Debug, PartialEq, Snafu)]
pub enum ActionError {
    #[snafu(display(
        "Source position {} must be in possession of mover {:?}",
        position,
        color
    ))]
    SourceColorError { position: u8, color: Color },

    #[snafu(display("Destination position {} must be empty", destination))]
    DestinationEmptyError { destination: u8 },

    #[snafu(display(
        "Skipped position {} must be have opponent of color {:?}",
        skipped,
        color
    ))]
    SkippedPositionError { skipped: u8, color: Color },

    #[snafu(display("One of the jumpers need to move!"))]
    HaveToJumpError,

    #[snafu(display("Only kings can move backwards!"))]
    SinglePieceBackwardsError,

    #[snafu(display("More jumping required!"))]
    NeedMoreJumpingError,
}

#[derive(Debug, Snafu)]
pub enum ParseError {
    // for board below
    #[snafu(display("Invalid color letter (W and B are valid)!"))]
    ColorError,

    #[snafu(display("Expected king designation or numbered position!"))]
    PieceError,

    #[snafu(display("{} is not a valid position 1 - 32", position))]
    PositionError { position: String },

    #[snafu(display("Couldn't parse board!"))]
    InvalidBoard,

    // for actions only...
    #[snafu(display("Can only have up to eight positions in any given movetext!"))]
    MoveQuantityError,

    #[snafu(display("Read invalid position (make sure all positions are between 1 and 32)!"))]
    PositionValueError,

    #[snafu(display("Error parsing delimiter '-' between positions in movetext!"))]
    InvalidDelimiter,

    #[snafu(display("Invalid action!"))]
    InvalidAction,

    // For the commands here....
    #[snafu(display("No command supplied!"))]
    NoCommandError,

    #[snafu(display("Invalid constraint option!"))]
    ConstraintOptionError,

    #[snafu(display("Invalid constraint value!"))]
    ConstraintValueError,

    #[snafu(display("Invalid command!"))]
    InvalidCommand,
}

impl<T> From<nom::Err<VerboseError<T>>> for ParseError {
    fn from(err: nom::Err<VerboseError<T>>) -> Self {
        let errors = match err {
            nom::Err::Error(VerboseError { errors }) => errors,
            _ => vec![],
        };

        for (_, kind) in errors {
            // will need a way of contexting invalid board, action, command, etc
            match kind {
                Context("color") => return ParseError::ColorError,
                Context("king") => return ParseError::PieceError,
                Context("digit") => return ParseError::PieceError,
                Context("position") => return ParseError::PositionValueError,
                Context("delimiter") => return ParseError::InvalidDelimiter,
                Context("no command") => return ParseError::NoCommandError,
                Context("constraint option") => return ParseError::ConstraintOptionError,
                Context("constraint value") => return ParseError::ConstraintValueError,
                _ => (),
            }
        }

        ParseError::ColorError // this will just need to be generalll..
    }
}
