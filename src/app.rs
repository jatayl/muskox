use std::io::{self, Write};
use std::error;
use std::process;
use std::default;

use crate::Bitboard;
use crate::Action;
use crate::movepick::MovePicker;
use crate::movepick::PickContraint;

// convert this to lifetimes later...
enum Command {
    SetFen(Bitboard),
    GetGameState,
    ValidateAction(Action),
    TakeAction(Action),
    GenerateAllActions,
    PickAction,
    GetTurn,
    Print,
    GetMoveHistory,
    Clear,
    Exit,
}
use Command::*;

impl Command {
    fn parse(command: &str) -> Result<Command, Box<dyn error::Error>> {
        // so unsafe hahaha

        let command_split: Vec<_> = command.split(" ").collect();

        if command_split.len() != 1 && command_split.len() != 2 {
            return Err(format!("Invalid command: {}!", command))?;
        }

        // might want different assert size based on the command.
        // there could be really useful macros here.

        // match a command string to the abstract command object
        // this parser deal does the error handling
        match command_split[0] {
            "fen" => {
                let fen_sring = command_split.get(1).unwrap();
                let board = Bitboard::new_from_fen(&fen_sring)?;
                Ok(SetFen(board))
            },
            "gamestate" => Ok(GetGameState),
            "validate" => {
                let action = Action::new_from_movetext(command_split.get(1).unwrap())?;
                Ok(ValidateAction(action))
            },
            "take" => {
                let action = Action::new_from_movetext(command_split.get(1).unwrap())?;
                Ok(TakeAction(action))
            }
            "generate" => Ok(GenerateAllActions),
            "pick" => Ok(PickAction),
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
    move_picker: MovePicker,
    action_history: Vec<Action>,
}

impl default::Default for State {
    fn default() -> State {
        let board = Bitboard::new();
        let move_picker = MovePicker::default();
        let action_history = Vec::new();
        State { board, move_picker, action_history }
    }
}

impl State {
    fn execute(&mut self, command: &Command) {
        // match an abstract command to the function
        match command {
            SetFen(board) => self.set_board(board),
            GetGameState => self.get_game_state(),
            ValidateAction(action) => self.validate_action(action),
            TakeAction(action) => self.take_action(action),
            GenerateAllActions => self.generate_all_actions(),
            GetTurn => self.get_turn(),
            PickAction => self.pick_action(),
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
    fn get_game_state(&self) {
        let game_state = self.board.get_game_state();
        println!("\n{}", game_state);
    }

    #[inline]
    fn validate_action(&self, action: &Action) {
        let validate = self.board.validate_action(&action);
        match validate {
            Ok(()) => println!("\ntrue"),
            Err(err) => println!("\nfalse: {}", err),
        }
    }

    #[inline]
    fn generate_all_actions(&self) {
        let mut out = String::new();
        let all_action_pairs = self.board.generate_all_actions();

        if all_action_pairs.len() == 0 {
            println!("\nno valid actions");
        }

        all_action_pairs.iter()
            .map(|p| p.0)
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

    #[inline]
    fn pick_action(&mut self) {
        let action = self.move_picker.pick(&self.board, &PickContraint::None);

        match action {
            Some(a) => println!("\n{}", a),
            None => println!("no action to take!"),
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
            println!("no moves taken yet");
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
        self.board = Bitboard::new();
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
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let command = Command::parse(&input);

        match command {
            Ok(cmd) => state.execute(&cmd),
            Err(err) => println!("Error: {}", err),
        }

        counter += 1;
    }
}