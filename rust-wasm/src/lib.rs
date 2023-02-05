mod board;
mod board_iters;
mod board_state;
mod consts;
mod game_manager;
mod heuristics;
mod layer_generator;
mod tree_analysis;
mod win_check;

use consts::{BOARD_HEIGHT, BOARD_WIDTH};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// Starts a new game with an empty board
pub fn new_game() {
    game_manager::new_game()
}

#[wasm_bindgen]
/// Starts a new game from a position
///
/// The position is given as array[row * 7 + col]
pub fn start_from_position(position: Box<[u8]>, turn: bool, last_move: u8) {
    if position.len() != (BOARD_HEIGHT * BOARD_WIDTH) as usize {
        game_manager::new_game();
    }

    let mut position_2d = [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            position_2d[row as usize][col as usize] = position[(row * BOARD_WIDTH + col) as usize];
        }
    }

    game_manager::start_from_position(position_2d, turn, last_move)
}

#[wasm_bindgen]
/// Returns the current position of the game as array[row * 6 + col]
pub fn get_position() -> Box<[u8]> {
    let position_2d = game_manager::get_position();

    let mut position = [0; (BOARD_HEIGHT * BOARD_WIDTH) as usize];
    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            position[(row * BOARD_WIDTH + col) as usize] = position_2d[row as usize][col as usize];
        }
    }

    Box::new(position)
}

#[wasm_bindgen]
/// Generates up to x board states in the decision tree
pub fn generate_x_states(x: isize) {
    game_manager::generate_x_states(x)
}

#[wasm_bindgen]
/// Drop a piece down the corresponding column
///
/// Returns a tuple containing if the move was made successfully,
/// if the game is over, and who won the game (0 is tie)
///
/// These values are represented as numbers
pub fn make_move(col: u8) -> Box<[u8]> {
    let (success, done, winner) = game_manager::make_move(col);
    let arr = [success as u8, done as u8, winner];
    Box::new(arr)
}

#[wasm_bindgen]
/// Returns a JSON dictionary of moves and scores
pub fn get_move_scores() -> String {
    let move_scores = game_manager::get_move_scores();

    let mut to_return = "{".to_owned();
    for (col, score) in move_scores {
        to_return.push_str(&format!("{}:{},", col, score));
    }
    to_return.push_str("}");

    to_return
}
