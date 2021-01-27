use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::digit1,
    combinator::{map, map_res},
    error::{context, VerboseError},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};
use num_traits::PrimInt;

use crate::app::Command;
use crate::board::{Action, Bitboard, Color};
use crate::error::ParseError;
use crate::search::SearchConstraint;

// try to condense these functions except for stuff taht is too large or reused..
// convert the match statements to nom's switch macro

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn from_decimal<T: PrimInt>(input: &str) -> Result<T, T::FromStrRadixErr> {
    T::from_str_radix(input, 10)
}

// everything below is for parsing the action

fn position_primary(input: &str) -> Res<&str, u8> {
    let (input, position) = context("position", map_res(digit1, from_decimal::<u8>))(input)?;

    if !(1..=32).contains(&position) {
        panic!("another unhandled error!");
    }

    Ok((input, position))
}

pub(crate) fn action_primary(input: &str) -> Res<&str, Action> {
    context(
        "delimiter",
        map_res(
            separated_list1(tag("-"), position_primary),
            Action::from_vec,
        ),
    )(input)
}

// everything below is for parsing the pick constraint

fn is_space(c: char) -> bool {
    c == ' '
}

fn search_constraint_primary(input: &str) -> Res<&str, SearchConstraint> {
    let (input, constraint_name) = take_while(|c: char| c.is_ascii_alphabetic())(input)?;
    let (input, _) = take_while(is_space)(input)?;

    // would be better to use the switch macro
    let constraint = match constraint_name {
        "" => SearchConstraint::none(),
        "timed" => map_res(map_res(digit1, from_decimal), SearchConstraint::time)(input)?.1,
        "depth" => map_res(map_res(digit1, from_decimal), SearchConstraint::depth)(input)?.1,
        _ => panic!("bad!"),
    };

    Ok((input, constraint))
}

// everything below is for parsing commands in app

pub(crate) fn command_primary(input: &str) -> Res<&str, Command> {
    use Command::*;

    let (input, cmd_name) = take_while(|c: char| c.is_ascii_alphabetic())(input)?;
    let (input, _) = take_while(is_space)(input)?;

    let wrap_fn = |cmd| Ok(("", cmd));

    match cmd_name {
        "fen" => match input {
            "" => wrap_fn(PrintFen),
            _ => map(board_fen_primary, SetFen)(input),
        },
        "validate" => map(action_primary, ValidateAction)(input),
        "take" => map(action_primary, TakeAction)(input),
        "search" => map(search_constraint_primary, Search)(input),
        "best" => map(search_constraint_primary, PickAction)(input),
        "evaluate" => map(search_constraint_primary, EvaluateBoard)(input),
        "gamestate" => wrap_fn(GetGameState),
        "generate" => wrap_fn(GenerateAllActions),
        "turn" => wrap_fn(GetTurn),
        "print" => wrap_fn(Print),
        "history" => wrap_fn(GetMoveHistory),
        "clear" => wrap_fn(Clear),
        "exit" => wrap_fn(Exit),
        _ => panic!("return error here when it implements properly!!"),
    }
}

// everything below is for parsing bitboards' fens

fn match_color(input: &str) -> Result<Color, ParseError> {
    // not sure if this is 'best practice'. switch to switch in nom
    match input {
        "W" => Ok(Color::White),
        "B" => Ok(Color::Black),
        _ => Err(ParseError::ColorError),
    }
}

fn color_primary(input: &str) -> Res<&str, Color> {
    context("color", map_res(take(1_usize), match_color))(input)
}

fn king_primary(input: &str) -> Result<bool, ParseError> {
    // not a huge fan of this
    match input {
        "K" => Ok(true),
        "" => Ok(false),
        _ => Err(ParseError::PieceError),
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

    let (input, position) = context("digit", map_res(digit1, from_decimal::<u32>))(input)?;

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
