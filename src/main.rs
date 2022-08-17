use std::mem::size_of;

use rusty_connect_four::{board::Board, board_state::BoardState};

fn main() {
    print!("Board size = {}\n", size_of::<Board>());
    print!("Vec size = {}\n", size_of::<Vec<BoardState>>());
    print!("u8 size = {}\n", size_of::<u8>());
    print!("BoardState size = {}\n", size_of::<BoardState>());
}
