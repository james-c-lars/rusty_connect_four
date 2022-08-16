use std::iter::zip;

use crate::board::{Board, Error};
use crate::consts::{BOARD_WIDTH, NUMBER_TO_WIN};

/// Returns whether the given color has won in the given board state
pub fn does_color_win(board: &Board, color: bool) -> bool {
    // Figuring out what row the highest piece is in
    // Can prevent iterating through empty rows
    let highest_row = board.get_max_height();

    // First checking for horizontal connect fours
    if has_horizontal_win(board, highest_row, color) {
        return true;
    }

    // We can skip the other checks if there's not yet pieces stacked four high
    if highest_row >= NUMBER_TO_WIN {
        // Checking for vertical connect fours
        if has_vertical_win(board, highest_row, color) {
            return true;
        }

        // Checking for upward diagonal connect fours
        if has_upward_diagonal_win(board, highest_row, color) {
            return true;
        }

        // Checking for downward diagonal connect fours
        if has_downward_diagonal_win(board, highest_row, color) {
            return true;
        }
    }

    false
}

/// Helper function to check for horizontal connect fours
fn has_horizontal_win(board: &Board, highest_row: u8, color: bool) -> bool {
    let mut in_a_row: u8;
    for row in 0..highest_row {
        in_a_row = 0;
        for col in 0..BOARD_WIDTH {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }
        }
    }

    // We didn't find any connect fours and can return false
    false
}

/// Helper function to check for vertical connect fours
fn has_vertical_win(board: &Board, highest_row: u8, color: bool) -> bool {
    let mut in_a_row: u8;
    for col in 0..BOARD_WIDTH {
        in_a_row = 0;
        for row in 0..highest_row {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // We can do an early stop if there aren't enough rows left to possibly make a connect four
            let remaining_rows = highest_row - (row + 1);
            if (in_a_row + remaining_rows) < NUMBER_TO_WIN {
                break;
            }
        }
    }

    // We didn't find any connect fours and can return false
    false
}

/// Helper function to check for upward diagonal connect fours
fn has_upward_diagonal_win(board: &Board, highest_row: u8, color: bool) -> bool {
    // There are two types of upward diagonals, ones that touch the bottom of the board
    //  and those that don't
    let mut in_a_row: u8;

    // We can first check the ones that don't touch the bottom of the board
    // i is iterating through the rows that could have a connect four going up from them
    for i in 1..=(highest_row - NUMBER_TO_WIN) {
        in_a_row = 0;
        for (col, row) in zip(0..BOARD_WIDTH, i..highest_row) {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // TODO: early stop if there aren't enough rows or cols left to get in_a_row to 4
        }
    }

    // We can then check the ones that do touch the bottom of the board
    // i is iterating through the columns that could have a connect four going from the right of them
    for i in 0..=(BOARD_WIDTH - NUMBER_TO_WIN) {
        in_a_row = 0;
        for (col, row) in zip(i..BOARD_WIDTH, 0..highest_row) {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // TODO: early stop if there aren't enough rows or cols left to get in_a_row to 4
        }
    }

    false
}

/// Helper function to check for downward diagonal connect fours
fn has_downward_diagonal_win(board: &Board, highest_row: u8, color: bool) -> bool {
    // There are two types of upward diagonals, ones that touch the bottom of the board
    //  and those that don't
    let mut in_a_row: u8;

    // We can first check the ones that don't touch the bottom of the board
    // i is iterating through the rows that could have a connect four going up from them
    for i in 1..=(highest_row - NUMBER_TO_WIN) {
        in_a_row = 0;
        for (col, row) in zip((0..BOARD_WIDTH).rev(), i..highest_row) {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // TODO: early stop if there aren't enough rows or cols left to get in_a_row to 4
        }
    }

    // We can then check the ones that do touch the bottom of the board
    // i is iterating through the columns that could have a connect four going from the left of them
    for i in 0..=(BOARD_WIDTH - NUMBER_TO_WIN) {
        in_a_row = 0;
        for (col, row) in zip((0..(BOARD_WIDTH - i)).rev(), 0..highest_row) {
            // We look at the piece and determine if it's the same as the last
            in_a_row = increment_if_matching(in_a_row, board.get_piece(col, row), color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // TODO: early stop if there aren't enough rows or cols left to get in_a_row to 4
        }
    }

    false
}

/// Helper function
/// Increments in_a_row based on if the new piece matches the given color
/// If it doesn't match, resets in_a_row to 0
fn increment_if_matching(in_a_row: u8, result: Result<bool, Error>, color: bool) -> u8 {
    match result {
        Ok(piece) => {
            if piece == color {
                in_a_row + 1
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        win_check::{
            has_downward_diagonal_win, has_horizontal_win, has_upward_diagonal_win,
            has_vertical_win,
        },
    };

    #[test]
    fn horizontal_wins() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [1, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 1, 2, 1, 1],
            [2, 1, 1, 2, 1, 1, 2],
            [1, 1, 2, 2, 2, 1, 1],
        ]);

        assert!(has_horizontal_win(&board, board.get_max_height(), false) == false);
        assert!(has_horizontal_win(&board, board.get_max_height(), true) == false);

        let board = Board::from_arrays([
            [2, 2, 2, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 1, 0, 0, 0],
        ]);

        assert!(has_horizontal_win(&board, board.get_max_height(), false));
        assert!(has_horizontal_win(&board, board.get_max_height(), true));

        let board = Board::from_arrays([
            [0, 0, 0, 2, 2, 2, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 1],
        ]);

        assert!(has_horizontal_win(&board, board.get_max_height(), false));
        assert!(has_horizontal_win(&board, board.get_max_height(), true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
        ]);

        assert!(has_horizontal_win(&board, board.get_max_height(), false));
        assert!(has_horizontal_win(&board, board.get_max_height(), true) == false);
    }

    #[test]
    fn vertical_wins() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [1, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 1, 2, 1, 1],
            [2, 1, 1, 2, 1, 1, 2],
            [1, 1, 2, 2, 2, 1, 1],
        ]);

        assert!(has_vertical_win(&board, board.get_max_height(), false) == false);
        assert!(has_vertical_win(&board, board.get_max_height(), true) == false);

        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1],
            [2, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
        ]);

        assert!(has_vertical_win(&board, board.get_max_height(), false));
        assert!(has_vertical_win(&board, board.get_max_height(), true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
        ]);

        assert!(has_vertical_win(&board, board.get_max_height(), false));
        assert!(has_vertical_win(&board, board.get_max_height(), true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ]);

        assert!(has_vertical_win(&board, board.get_max_height(), false) == false);
        assert!(has_vertical_win(&board, board.get_max_height(), true));
    }

    #[test]
    fn upward_diagonal_wins() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [1, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 1, 2, 1, 1],
            [2, 1, 2, 2, 1, 1, 2],
            [1, 1, 1, 2, 2, 1, 1],
        ]);

        assert!(has_upward_diagonal_win(&board, 6, false) == false);
        assert!(has_upward_diagonal_win(&board, 6, true) == false);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 2, 1],
            [0, 0, 0, 1, 2, 1, 2],
            [0, 0, 1, 2, 1, 1, 1],
            [0, 1, 1, 2, 1, 1, 1],
            [1, 1, 2, 1, 1, 1, 1],
        ]);

        assert!(has_upward_diagonal_win(&board, 6, false));
        assert!(has_upward_diagonal_win(&board, 6, true));

        let board = Board::from_arrays([
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 2],
            [1, 1, 1, 1, 0, 2, 1],
            [2, 1, 1, 1, 2, 1, 1],
            [2, 1, 1, 2, 1, 1, 1],
        ]);

        assert!(has_upward_diagonal_win(&board, 6, false));
        assert!(has_upward_diagonal_win(&board, 6, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 1, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
            [0, 2, 1, 1, 1, 0, 0],
        ]);

        assert!(has_upward_diagonal_win(&board, 5, false));
        assert!(has_upward_diagonal_win(&board, 5, true) == false);
    }

    #[test]
    fn downward_diagonal_wins() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [1, 1, 1, 1, 1, 2, 1],
            [1, 1, 2, 1, 2, 1, 1],
            [2, 1, 2, 2, 2, 1, 2],
            [1, 1, 1, 2, 2, 1, 1],
        ]);

        assert!(has_downward_diagonal_win(&board, 6, false) == false);
        assert!(has_downward_diagonal_win(&board, 6, true) == false);

        let board = Board::from_arrays([
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 2, 0, 0],
            [1, 0, 0, 2, 1, 2, 0],
            [2, 1, 0, 2, 1, 1, 2],
            [2, 2, 1, 2, 1, 1, 1],
            [2, 2, 2, 1, 1, 1, 1],
        ]);

        assert!(has_downward_diagonal_win(&board, 6, false));
        assert!(has_downward_diagonal_win(&board, 6, true));

        let board = Board::from_arrays([
            [1, 0, 0, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 1, 2, 0, 0],
            [1, 1, 1, 2, 2, 2, 0],
            [1, 1, 1, 2, 2, 2, 2],
        ]);

        assert!(has_downward_diagonal_win(&board, 6, false));
        assert!(has_downward_diagonal_win(&board, 6, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 0, 1, 2, 0, 0, 0],
            [0, 0, 1, 1, 2, 0, 0],
            [0, 0, 1, 1, 1, 2, 0],
            [0, 0, 1, 1, 1, 2, 0],
        ]);

        assert!(has_downward_diagonal_win(&board, 5, false) == false);
        assert!(has_downward_diagonal_win(&board, 5, true));
    }
}
