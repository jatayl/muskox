use std::io::{self, Write};
use std::error;
use std::process;
use std::default;
use std::sync::Arc;

use crate::board::{Bitboard, Action};
use crate::evaluation::BoardEvaluator;
use crate::search::{Engine, SearchConstraint, Searchable};
use crate::error::ParseCommandError;

// convert this to lifetimes later...
enum Command {
    SetFen(Bitboard),
    PrintFen,
    GetGameState,
    ValidateAction(Action),
    TakeAction(Action),
    GenerateAllActions,
    Search(SearchConstraint),
    PickAction(SearchConstraint),
    EvaluateBoard(SearchConstraint),
    GetTurn,
    Print,
    GetMoveHistory,
    Clear,
    Exit,
}
use Command::*;

// this module is so poorly written its not even funny! check other files for better, more interesting code :)

impl Command {
    fn parse(command: &str) -> Result<Command, Box<dyn error::Error>> {
        // this parser is supposed to do the error handling of commands strings
        // so that the other part of this module does not have to

        let command_split: Vec<_> = command.split(" ").collect();

        // might want different assert size based on the command.

        // match a command string to the abstract command object
        match *command_split.get(0).ok_or(ParseCommandError::NoCommandError)? {
            "fen" => {
                match command_split.get(1) {
                    Some(fen_string) => {
                        let board = Bitboard::from_fen(&fen_string)?;
                        Ok(SetFen(board))
                    },
                    None => Ok(PrintFen),
                }
            },
            "gamestate" => Ok(GetGameState),
            "validate" => {
                let action = Action::from_movetext(command_split.get(1)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "action".to_string() })?)?;
                Ok(ValidateAction(action))
            },
            "take" => {
                let action = Action::from_movetext(command_split.get(1)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "action".to_string() })?)?;
                Ok(TakeAction(action))
            }
            "generate" => Ok(GenerateAllActions),
            "search" => Ok(Search(match command_split.get(1) {
                Some(&"depth") => SearchConstraint::depth(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(&"timed") => SearchConstraint::time(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(bad_option) =>
                    return Err(Box::new(ParseCommandError::ConstraintOptionError { option: bad_option.to_string() })),
                None => SearchConstraint::None,
            })),
            "best" => Ok(PickAction(match command_split.get(1) {
                Some(&"depth") => SearchConstraint::depth(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(&"timed") => SearchConstraint::time(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(bad_option) =>
                    return Err(Box::new(ParseCommandError::ConstraintOptionError { option: bad_option.to_string() })),
                None => SearchConstraint::None,
            })),
            "evaluate" => Ok(EvaluateBoard(match command_split.get(1) {
                 Some(&"depth") => SearchConstraint::depth(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(&"timed") => SearchConstraint::time(command_split.get(2)
                    .ok_or(ParseCommandError::ExpectedParameterError { parameter: "depth".to_string() })?
                    .parse::<u32>()?)?,
                Some(bad_option) =>
                    return Err(Box::new(ParseCommandError::ConstraintOptionError { option: bad_option.to_string() })),
                None => SearchConstraint::None,
            })),
            "turn" => Ok(GetTurn),
            "print" => Ok(Print),
            "history" => Ok(GetMoveHistory),
            "clear" => Ok(Clear),
            "exit" => Ok(Exit),
            _ => Err(format!("Invalid command: {}!", command))?,
        }
    }
}

// will need paramters here for the engine
// have command history as well maybe
struct State {
    board: Bitboard,
    engine: Engine<Bitboard>,
    action_history: Vec<Action>,
}

impl default::Default for State {
    fn default() -> State {
        let board = Bitboard::default();
        let evaluator = Arc::new(BoardEvaluator::default());
        let engine = Engine::new(evaluator);
        let action_history = Vec::new();
        State { board, engine, action_history }
    }
}

impl State {
    fn execute(&mut self, command: &Command) {
        // match an abstract command to the function
        match command {
            SetFen(board) => self.set_board(board),
            PrintFen => self.print_fen(),
            GetGameState => self.get_game_state(),
            ValidateAction(action) => self.validate_action(action),
            TakeAction(action) => self.take_action(action),
            GenerateAllActions => self.generate_all_actions(),
            GetTurn => self.get_turn(),
            Search(constraint) => self.search(constraint),
            PickAction(constraint) => self.pick_action(constraint),
            EvaluateBoard(constraint) => self.evaluate_board(constraint),
            Print => self.print(),
            GetMoveHistory => self.get_move_history(),
            Clear => self.clear(),
            Exit => process::exit(1),
        }
    }

    #[inline]
    fn set_board(&mut self, board: &Bitboard) {
        self.board = *board;
        self.action_history = Vec::new();
    }

    #[inline]
    fn print_fen(&self) {
        println!("\n{}", self.board.fen());
    }

    #[inline]
    fn get_game_state(&self) {
        let game_state = self.board.get_game_state();
        println!("\n{}", game_state);
    }

    #[inline]
    fn validate_action(&self, action: &Action) {
        let validate = self.board.validate_action(&action);
        match validate {
            Ok(()) => println!("\nOk"),
            Err(err) => println!("\nError: {}", err),
        }
    }

    fn generate_all_actions(&self) {
        let mut out = String::new();
        let all_action_pairs = self.board.generate_all_actions();

        if all_action_pairs.len() == 0 {
            println!("\nno valid actions");
            return;
        }

        all_action_pairs.iter()
            .map(|p| p.action())
            .map(|a| a.to_string())
            .for_each(|t| {
                out.push_str(&t);
                out.push_str(", ")
            });

        // unneeded extra ', '
        out.pop();
        out.pop();

        println!("\n{}", out);
    }

    #[inline]
    fn get_turn(&self) {
        println!("\n{:?}", self.board.turn());
    }

    fn search(&mut self, constraint: &SearchConstraint) {
        let mut out = String::new();

        let search = self.engine.search(&self.board, constraint);

        if search.len() == 0 {
            println!("\nno valid actions");
            return;
        }

        for pair in search {
            out.push_str(&format!("{} ({}), ", pair.action(), pair.score()));
        }

        // unneeded extra ', '
        out.pop();
        out.pop();

        println!("\n{}", out);
    }

    #[inline]
    fn pick_action(&mut self, constraint: &SearchConstraint) {
        match self.engine.search(&self.board, constraint).get(0) {
            Some(p) => println!("\n{}", p.action()),
            None => println!("no action to take!"),
        };
    }

    #[inline]
    fn evaluate_board(&mut self, constraint: &SearchConstraint) {
        match self.engine.search(&self.board, constraint).get(0) {
            Some(p) => println!("\n{}", p.score()),
            None => self.get_game_state(), // the game is over
        }
    }

    #[inline]
    fn take_action(&mut self, action: &Action) {
        let validate = self.board.take_action(action);
        self.action_history.push(*action);
        match validate {
            Ok(board_p) => self.board = board_p,
            Err(err) => println!("\nError: {}", err),
        }
    }

    #[inline]
    fn print(&self) {
        println!("\n{}", self.board.pretty())
    }

    #[inline]
    fn get_move_history(&self) {
        // going to have to make sure we comply with PDN later
        let mut out = String::new();

        if self.action_history.len() == 0 {
            println!("\nno moves taken yet");
            return
        }

        self.action_history.iter()
            .map(|a| a.movetext())
            .for_each(|t| {
                out.push_str(&t);
                out.push_str(", ")
            });
        // unneeded extra ', '
        out.pop();
        out.pop();

        println!("\n{}", out);
    }

    #[inline]
    fn clear(&mut self) {
        self.board = Bitboard::default();
        self.action_history = Vec::new();
    }
}

pub fn run() -> ! {
    println!("Developed by James in Cary");

    let mut state = State::default();

    let mut counter = 0;

    loop {
        print!("\n[{}]: ", counter);
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error with your standard input!");
        let input = input.trim();

        let command = Command::parse(&input);

        match command {
            Ok(cmd) => state.execute(&cmd),
            Err(err) => println!("\nError: {}", err),
        }

        counter += 1;
    }
}