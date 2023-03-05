use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    consts::{BOARD_HEIGHT, BOARD_WIDTH},
    game_engine::{
        board::Board, board_state::BoardState, layer_generator::LayerGenerator,
        tree_analysis::how_good_is,
    },
};

// Reexport GameOver
pub use crate::game_engine::board_state::GameOver;

#[derive(Debug)]
pub struct GameManager {
    board_state: Rc<RefCell<BoardState>>,
    layer_generator: LayerGenerator,
}

impl GameManager {
    /// Starts a new game with an empty board.
    pub fn new_game() -> GameManager {
        let state: Rc<RefCell<BoardState>> = RefCell::new(BoardState::default_const()).into();

        GameManager {
            board_state: state.clone(),
            layer_generator: LayerGenerator::new(state),
        }
    }

    /// Starts a new game from a position.
    ///
    /// The position is given as array[row][col].
    pub fn start_from_position(
        position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        turn: bool,
        last_move: u8,
    ) -> GameManager {
        let state: Rc<RefCell<BoardState>> = RefCell::new(BoardState::new(
            Board::from_arrays(position),
            turn,
            last_move,
        ))
        .into();

        GameManager {
            board_state: state.clone(),
            layer_generator: LayerGenerator::new(state),
        }
    }

    /// Returns the current position of the game as array[row][col].
    pub fn get_position(&self) -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
        self.board_state.borrow().board.to_arrays()
    }

    /// Generates up to x board states in the decision tree.
    pub fn generate_x_states(&mut self, x: usize) {
        for _ in 0..x {
            if let None = self.layer_generator.next() {
                break;
            }
        }
    }

    /// Drop a piece down the corresponding column.
    pub fn make_move(&mut self, col: u8) -> Result<(), String> {
        // If the game is already won, no move is valid
        if GameOver::NoWin != self.board_state.borrow().is_game_over() {
            return Err(format!("Game is already over. Can't make move: {}", col));
        }

        // We haven't yet generated the children of this board state
        if self.board_state.borrow().children.len() == 0 {
            self.generate_x_states(1);

            if self.board_state.borrow().children.len() == 0 {
                return Err(format!(
                    "Was unable to generate children for the root. Can't make move: {}",
                    col
                ));
            }
        }

        let mut is_valid_col = false;
        for child in self.board_state.borrow().children.iter() {
            if child.borrow().get_last_move() == col {
                is_valid_col = true;
            }
        }

        if !is_valid_col {
            return Err(format!(
                "The chosen column wasn't valid. Can't make move: {}",
                col
            ));
        }

        self.board_state
            .replace(self.board_state.take().narrow_possibilities(col).take());
        self.layer_generator = LayerGenerator::new(self.board_state.clone());
        Ok(())
    }

    /// Returns a map of moves to their corresponding scores.
    ///
    /// Higher scores are better for the player about to make a move,
    ///  lower scores are better for their opponent.
    pub fn get_move_scores(&self) -> HashMap<u8, isize> {
        let mut move_scores = HashMap::new();

        let borrowed_board_state = self.board_state.borrow();
        let child_iter = borrowed_board_state.children.iter();
        let whose_turn = borrowed_board_state.get_turn();

        for child in child_iter {
            let child_score = if whose_turn {
                how_good_is(&child.borrow())
            } else {
                // Some funky handling to avoid int overflow on negating isize::MIN
                match how_good_is(&child.borrow()) {
                    isize::MIN => isize::MAX,
                    isize::MAX => isize::MIN,
                    score => -score,
                }
            };

            move_scores.insert(child.borrow().get_last_move(), child_score);
        }

        move_scores
    }

    /// Returns whether the game is over, and if so who won.
    pub fn is_game_over(&self) -> GameOver {
        self.board_state.borrow().is_game_over()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::game_engine::{
        board_state::GameOver, game_manager::GameManager, tree_analysis::how_good_is,
    };

    #[test]
    fn board_translation() {
        let board_array = [
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2, 1],
            [0, 1, 2, 0, 0, 1, 2],
            [0, 1, 2, 0, 2, 1, 2],
        ];

        let manager = GameManager::start_from_position(board_array, true, 6);

        assert_eq!(manager.get_position(), board_array);
    }

    #[test]
    fn generates_to_win() {
        let board_array = [
            [1, 2, 2, 1, 1, 0, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ];

        let mut manager = GameManager::start_from_position(board_array, false, 0);

        manager.generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(how_good_is(&state.borrow()), isize::MIN);

        let mut manager = GameManager::start_from_position(board_array, true, 0);

        manager.generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(how_good_is(&state.borrow()), 0);
    }

    #[test]
    fn drops_successful() {
        let board_array = [
            [1, 2, 2, 1, 1, 0, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ];

        let mut manager = GameManager::start_from_position(board_array, false, 0);

        manager.make_move(5).unwrap();
        manager.make_move(5).unwrap_err();
        manager.make_move(4).unwrap_err();
        manager.make_move(0).unwrap_err();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap_err();
        assert_eq!(manager.is_game_over(), GameOver::OneWins);

        let mut manager = GameManager::start_from_position(board_array, true, 0);

        manager.make_move(5).unwrap();
        manager.make_move(5).unwrap_err();
        manager.make_move(4).unwrap_err();
        manager.make_move(0).unwrap_err();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap();
        manager.make_move(6).unwrap_err();
        assert_eq!(manager.is_game_over(), GameOver::Tie);
    }

    #[test]
    fn correct_predictions() {
        let board_array = [
            [1, 2, 2, 1, 1, 0, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [1, 2, 1, 2, 1, 2, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ];

        let mut manager = GameManager::start_from_position(board_array, false, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        let mut real_move_scores = HashMap::new();
        real_move_scores.insert(5, isize::MAX);
        real_move_scores.insert(6, 0);
        assert_eq!(move_scores, real_move_scores);

        let mut manager = GameManager::start_from_position(board_array, true, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        let mut real_move_scores = HashMap::new();
        real_move_scores.insert(5, 0);
        real_move_scores.insert(6, 0);
        assert_eq!(move_scores, real_move_scores);

        let board_array = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ];

        let mut manager = GameManager::start_from_position(board_array, false, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_ne!(score, isize::MIN);
            } else {
                assert_eq!(score, isize::MIN);
            }
        }

        let mut manager = GameManager::start_from_position(board_array, true, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_eq!(score, isize::MAX);
            } else {
                assert_ne!(score, isize::MAX);
            }
        }
    }
}
