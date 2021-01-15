use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::digit1,
    combinator::map_res,
    error::{context, VerboseError},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

use crate::board::{Bitboard, Color};
use crate::error::ParseBoardError;

// try to condense these functions except for stuff taht is too large or reused..

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn match_color(input: &str) -> Result<Color, ParseBoardError> {
    // not sure if this is 'best practice'
    match input {
        "W" => Ok(Color::White),
        "B" => Ok(Color::Black),
        _ => Err(ParseBoardError::ColorError),
    }
}

fn color_primary(input: &str) -> Res<&str, Color> {
    context("color", map_res(take(1_usize), match_color))(input)
}

fn get_position(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 10)
}

fn king_primary(input: &str) -> Result<bool, ParseBoardError> {
    // not a huge fan of this
    match input {
        "K" => Ok(true),
        "" => Ok(false),
        _ => Err(ParseBoardError::PieceError),
    }
}

fn piece_primary(input: &str) -> Res<&str, (u32, bool)> {
    let (input, is_king) = context(
        "king",
        map_res(take_while(|c: char| c.is_ascii_alphabetic()), king_primary),
    )(input)?;

    if input.is_empty() || input.as_bytes()[0] == 58 {
        return Ok((input, (0, false)));
    }

    let (input, position) = context("digit", map_res(digit1, get_position))(input)?;

    // need to make sure the position is less than 32
    let mask = 1 << (position - 1);

    Ok((input, (mask, is_king)))
}

fn side_primary(input: &str) -> Res<&str, (Color, u32, u32)> {
    let (input, _) = tag(":")(input)?;

    let (input, side) = color_primary(input)?;

    let (input, items) = separated_list1(tag(","), piece_primary)(input)?;

    if items.is_empty() {
        return Ok((input, (side, 0, 0)));
    }

    // not a huge fan of this here
    let (pieces, kings) = items
        .iter()
        .fold((0, 0), |(p_acc, k_acc), (mask, is_king)| {
            let k_x = match is_king {
                true => mask,
                false => &0,
            };
            (p_acc | mask, k_acc | k_x)
        });

    Ok((input, (side, pieces, kings)))
}

pub(crate) fn board_fen_primary(input: &str) -> Res<&str, Bitboard> {
    // read the color of the turn
    let (input, turn) = color_primary(input)?;

    #[allow(unused_variables)]
    let (input, ((s1_clr, s1_pieces, s1_kings), (_, s2_pieces, s2_kings))) =
        tuple((side_primary, side_primary))(input)?;

    let kings = s1_kings | s2_kings;

    let (blacks, whites) = match s1_clr {
        Color::Black => (s1_pieces, s2_pieces),
        Color::White => (s2_pieces, s1_pieces),
    };

    let board = Bitboard::new(blacks, whites, kings, turn);

    Ok((input, board))
}
