#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rusty_connect_four::{
    game_manager,
    consts::{BOARD_HEIGHT, BOARD_WIDTH}
};

#[tauri::command]
fn new_game() {
    game_manager::new_game()
}

#[tauri::command]
fn start_from_position(
    position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    turn: bool,
    last_move: u8,
) {
    game_manager::start_from_position(position, turn, last_move)
}

#[tauri::command]
fn get_position() -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
    game_manager::get_position()
}

#[tauri::command]
fn generate_x_states(x: isize) {
    game_manager::generate_x_states(x)
}

#[tauri::command]
fn make_move(col: u8) -> (bool, bool, u8) {
    game_manager::make_move(col)
}

#[tauri::command]
fn get_move_scores() -> Vec<(u8, isize)> {
    game_manager::get_move_scores()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![new_game])
        .invoke_handler(tauri::generate_handler![start_from_position])
        .invoke_handler(tauri::generate_handler![get_position])
        .invoke_handler(tauri::generate_handler![generate_x_states])
        .invoke_handler(tauri::generate_handler![make_move])
        .invoke_handler(tauri::generate_handler![get_move_scores])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
