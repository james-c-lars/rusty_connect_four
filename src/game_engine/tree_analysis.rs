use std::{
    cmp::{max, min},
    isize::{MAX, MIN},
};

use crate::game_engine::{
    board_state::BoardState, heuristics::how_good_is_board, win_check::GameOver,
};

/// Analyses a BoardState to determine how good it is based off of its
///  entire decision tree.
pub fn how_good_is(board_state: &BoardState) -> isize {
    board_state.alpha_beta_pruning(MIN, MAX)
}

impl BoardState {
    /// An implementation of alpha-beta pruning, a faster version of the
    ///  mini-max algorithm.
    fn alpha_beta_pruning(&self, mut alpha: isize, mut beta: isize) -> isize {
        // If the game is over, we can return a score based on who won
        match self.is_game_over() {
            GameOver::Tie => return 0,
            GameOver::OneWins => return MIN,
            GameOver::TwoWins => return MAX,
            _ => (),
        }

        // If the BoardState is a terminal node we can use our heuristic
        if self.children.len() == 0 {
            return how_good_is_board(&self.board);
        }

        // Otherwise we can proceed with alpha-beta pruning the child nodes
        if self.get_turn() {
            // We are the maximizing player
            let mut value = MIN;
            for child in self.children.iter() {
                value = max(value, child.state.borrow().alpha_beta_pruning(alpha, beta));

                if value >= beta {
                    break;
                }

                alpha = max(alpha, value);
            }

            return value;
        } else {
            // We are the minimizing player
            let mut value = MAX;
            for child in self.children.iter() {
                value = min(value, child.state.borrow().alpha_beta_pruning(alpha, beta));

                if value <= alpha {
                    break;
                }

                beta = min(beta, value);
            }

            return value;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::isize::{MAX, MIN};

    use crate::game_engine::{
        board::Board, layer_generator::LayerGenerator, transposition::TranspositionTable,
    };

    use super::how_good_is;

    #[test]
    fn alpha_beta_pruning() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 2, 2, 2, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 0],
        ]);

        let mut table = TranspositionTable::default();
        let (board_state, _) = table.get_board_state(board, false);
        let mut generator = LayerGenerator::new(table);

        for _ in 0..1000 {
            generator.next();
        }

        assert_eq!(how_good_is(&board_state.borrow()), MIN);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 1, 2],
            [0, 2, 2, 0, 1, 2, 2],
            [0, 1, 1, 0, 2, 2, 1],
        ]);

        let mut table = TranspositionTable::default();
        let (board_state, _) = table.get_board_state(board, true);
        let mut generator = LayerGenerator::new(table);

        for _ in 0..1000 {
            generator.next();
        }

        assert_ne!(how_good_is(&board_state.borrow()), MIN);
        assert_ne!(how_good_is(&board_state.borrow()), MAX);

        let board = Board::from_arrays([
            [1, 2, 2, 1, 1, 0, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ]);

        let mut table = TranspositionTable::default();
        let (board_state, _) = table.get_board_state(board, false);
        let mut generator = LayerGenerator::new(table);

        for _ in 0..1000 {
            generator.next();
        }

        assert_eq!(how_good_is(&board_state.borrow()), MIN);

        let board = Board::from_arrays([
            [1, 2, 2, 1, 1, 0, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ]);

        let mut table = TranspositionTable::default();
        let (board_state, _) = table.get_board_state(board, true);
        let mut generator = LayerGenerator::new(table);

        for _ in 0..1000 {
            generator.next();
        }

        assert_eq!(how_good_is(&board_state.borrow()), 0);
    }
}
