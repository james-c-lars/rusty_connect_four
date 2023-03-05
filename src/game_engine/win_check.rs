use crate::{
    consts::NUMBER_TO_WIN,
    game_engine::board::{Board, OutOfBounds},
};

/// Returns whether the given color has won in the given board state.
pub fn has_color_won(board: &Board, color: bool) -> bool {
    // Figuring out what row the highest piece is in
    // Can prevent iterating through empty rows
    let highest_row = board.get_max_height();

    // First checking for horizontal connect fours
    if has_color_won_horizontally(board, color) {
        return true;
    }

    // We can skip the other checks if there's not yet pieces stacked four high
    if highest_row >= NUMBER_TO_WIN {
        // Checking for the other possible connect fours
        if has_color_won_vertically(board, color)
            || has_color_won_upward_diagonally(board, color)
            || has_color_won_downward_diagonally(board, color)
        {
            return true;
        }
    }

    false
}

/// Helper function to check for horizontal connect fours.
fn has_color_won_horizontally(board: &Board, color: bool) -> bool {
    check_strips(board.horizontal_strip_iter(), color)
}

/// Helper function to check for vertical connect fours.
fn has_color_won_vertically(board: &Board, color: bool) -> bool {
    check_strips(board.vertical_strip_iter(false), color)
}

/// Helper function to check for upward diagonal connect fours.
fn has_color_won_upward_diagonally(board: &Board, color: bool) -> bool {
    check_strips(board.upward_diagonal_strip_iter(false), color)
}

/// Helper function to check for downward diagonal connect fours.
fn has_color_won_downward_diagonally(board: &Board, color: bool) -> bool {
    check_strips(board.downward_diagonal_strip_iter(false), color)
}

/// Helper function to check a strip iterator for connect fours.
fn check_strips<T, U>(mut strip_iter: T, color: bool) -> bool
where
    T: Iterator<Item = U>,
    U: ExactSizeIterator + Iterator<Item = Result<bool, OutOfBounds>>,
{
    // We iterate through each strip of spaces in the board
    while let Some(mut strip) = strip_iter.next() {
        // As we come across each piece we track how many in a row we've seen
        let mut in_a_row = 0;

        while let Some(piece) = strip.next() {
            in_a_row = increment_if_matching(in_a_row, piece, color);

            // If there are four in a row, then we can return true
            if in_a_row == NUMBER_TO_WIN {
                return true;
            }

            // And if there aren't enough pieces left to make a connect four, we can break early
            if in_a_row + (strip.len() as u8) < NUMBER_TO_WIN {
                break;
            }
        }
    }

    // We didn't find any connect fours and can return false
    false
}

/// Helper function for check_strips.
///
/// Increments in_a_row based on if the new piece matches the given color.
/// If it doesn't match, resets in_a_row to 0.
fn increment_if_matching(in_a_row: u8, piece: Result<bool, OutOfBounds>, color: bool) -> u8 {
    match piece {
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
    use crate::game_engine::{
        board::Board,
        win_check::{
            has_color_won, has_color_won_downward_diagonally, has_color_won_horizontally,
            has_color_won_upward_diagonally, has_color_won_vertically,
        },
    };

    #[test]
    fn horizontal_wins() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        assert!(has_color_won_horizontally(&board, false) == false);
        assert!(has_color_won(&board, false) == false);
        assert!(has_color_won_horizontally(&board, true) == false);
        assert!(has_color_won(&board, true) == false);

        let board = Board::from_arrays([
            [2, 2, 2, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 1, 0, 0, 0],
        ]);

        assert!(has_color_won_horizontally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_horizontally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 2, 2, 2, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 2],
            [0, 0, 0, 1, 1, 1, 1],
        ]);

        assert!(has_color_won_horizontally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_horizontally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
            [0, 2, 2, 2, 1, 0, 0],
        ]);

        assert!(has_color_won_horizontally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_horizontally(&board, true) == false);
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

        assert!(has_color_won_vertically(&board, false) == false);
        assert!(has_color_won_vertically(&board, true) == false);

        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1],
            [2, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
        ]);

        assert!(has_color_won_vertically(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_vertically(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 1],
        ]);

        assert!(has_color_won_vertically(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_vertically(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ]);

        assert!(has_color_won_vertically(&board, false) == false);
        assert!(has_color_won_vertically(&board, true));
        assert!(has_color_won(&board, true));
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

        assert!(has_color_won_upward_diagonally(&board, false) == false);
        assert!(has_color_won_upward_diagonally(&board, true) == false);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 2, 1],
            [0, 0, 0, 1, 2, 1, 2],
            [0, 0, 1, 2, 1, 1, 1],
            [0, 1, 1, 2, 1, 1, 1],
            [1, 1, 2, 1, 1, 1, 1],
        ]);

        assert!(has_color_won_upward_diagonally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_upward_diagonally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 2],
            [1, 1, 1, 1, 0, 2, 1],
            [2, 1, 1, 1, 2, 1, 1],
            [2, 1, 1, 2, 1, 1, 1],
        ]);

        assert!(has_color_won_upward_diagonally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_upward_diagonally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 1, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
            [0, 2, 1, 1, 1, 0, 0],
        ]);

        assert!(has_color_won_upward_diagonally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_upward_diagonally(&board, true) == false);
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

        assert!(has_color_won_downward_diagonally(&board, false) == false);
        assert!(has_color_won_downward_diagonally(&board, true) == false);

        let board = Board::from_arrays([
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 2, 0, 0],
            [1, 0, 0, 2, 1, 2, 0],
            [2, 1, 0, 2, 1, 1, 2],
            [2, 2, 1, 2, 1, 1, 1],
            [2, 2, 2, 1, 1, 1, 1],
        ]);

        assert!(has_color_won_downward_diagonally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_downward_diagonally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [1, 0, 0, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 1, 2, 0, 0, 0],
            [1, 1, 1, 1, 2, 0, 0],
            [1, 1, 1, 2, 2, 2, 0],
            [1, 1, 1, 2, 2, 2, 2],
        ]);

        assert!(has_color_won_downward_diagonally(&board, false));
        assert!(has_color_won(&board, false));
        assert!(has_color_won_downward_diagonally(&board, true));
        assert!(has_color_won(&board, true));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 0, 1, 2, 0, 0, 0],
            [0, 0, 1, 1, 2, 0, 0],
            [0, 0, 1, 1, 1, 2, 0],
            [0, 0, 1, 1, 1, 2, 0],
        ]);

        assert!(has_color_won_downward_diagonally(&board, false) == false);
        assert!(has_color_won_downward_diagonally(&board, true));
        assert!(has_color_won(&board, true));
    }
}
