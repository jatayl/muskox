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
pub enum ParseBoardError {
    #[snafu(display("Invalid color letter (W and B are valid)!"))]
    ColorError,

    #[snafu(display("Expected king designation or numbered position!"))]
    PieceError,

    #[snafu(display("{} is not a valid position 1 - 32", position))]
    PositionError { position: String },

    #[snafu(display("Couldn't parse board!"))]
    GeneralError,
}

impl<T> From<nom::Err<VerboseError<T>>> for ParseBoardError {
    fn from(err: nom::Err<VerboseError<T>>) -> Self {
        let errors = match err {
            nom::Err::Error(VerboseError { errors }) => errors,
            _ => vec![],
        };

        for (_, kind) in errors {
            if let Context(context) = kind {
                match context {
                    "color" => return ParseBoardError::ColorError,
                    "king" => return ParseBoardError::PieceError,
                    "digit" => return ParseBoardError::PieceError,
                    _ => (),
                }
            }
        }

        ParseBoardError::GeneralError
    }
}

#[derive(Debug, Snafu)]
pub enum ParseActionError {
    #[snafu(display("Number of moves must be 1 and 8 and not {}", quantity))]
    MoveQuantityError { quantity: usize },

    #[snafu(display("Position {} is invalid", position))]
    PositionValueError { position: String },
}

#[derive(Debug, Snafu)]
pub enum ParseCommandError {
    #[snafu(display("No command supplied!"))]
    NoCommandError,

    #[snafu(display("Invalid constraint option: {}!", option))]
    ConstraintOptionError { option: String },

    #[snafu(display("Expected parameter for {}!", parameter))]
    ExpectedParameterError { parameter: String },
}
