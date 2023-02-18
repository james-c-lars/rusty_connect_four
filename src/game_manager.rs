use std::{collections::HashMap, rc::Rc};

use crate::{
    board::Board,
    board_state::{BoardState, GameOver},
    consts::{BOARD_HEIGHT, BOARD_WIDTH},
    layer_generator::LayerGenerator,
    tree_analysis::how_good_is,
};

#[derive(Debug)]
pub struct GameManager {
    board_state: Rc<BoardState>,
    layer_generator: LayerGenerator,
}

impl GameManager {
    /// Starts a new game with an empty board
    pub fn new_game() -> GameManager {
        let state: Rc<BoardState> = BoardState::default_const().into();

        GameManager {
            board_state: state.clone(),
            layer_generator: LayerGenerator::new(state),
        }
    }

    /// Starts a new game from a position
    ///
    /// The position is given as array[row][col]
    pub fn start_from_position(
        position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        turn: bool,
        last_move: u8,
    ) -> GameManager {
        let state: Rc<BoardState> = BoardState::new(
            Board::from_arrays(position),
            turn,
            last_move,
        ).into();

        GameManager {
            board_state: state.clone(),
            layer_generator: LayerGenerator::new(state),
        }
    }
    
    /// Returns the current position of the game as array[row][col]
    pub fn get_position(&mut self) -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
        let mut position = [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                position[(BOARD_HEIGHT - 1 - row) as usize][col as usize] =
                    match self.board_state.board.get_piece(col, row) {
                        Ok(piece) => piece as u8 + 1,
                        Err(_) => 0,
                    };
            }
        }

        position
    }
    
    /// Generates up to x board states in the decision tree
    pub fn generate_x_states(&mut self, x: isize) {
        for _ in 0..x {
            if let None = self.layer_generator.next() {
                break;
            }
        }
    }
    
    /// Drop a piece down the corresponding column
    pub fn make_move(&mut self, col: u8) -> Result<(), String> {
        // If the game is already won, no move is valid
        if GameOver::NoWin != self.board_state.is_game_over() {
            return Err(format!("Game is already over. Can't make move: {}", col));
        }

        // We haven't yet generated the children of this board state
        if self.board_state.children.len() == 0 {
            return Err(format!("No children have been generated. Can't make move: {}", col));
        }

        for child in self.board_state.children.iter() {
            if child.get_last_move() == col {
                self.board_state = self.board_state.narrow_possibilities(col);
                
                return Ok(());
            }
        }

        // The chosen column isn't valid
        Err(format!("The chosen column wasn't valid. Can't make move: {}", col))
    }

    /// Returns a vector of moves and their corresponding scores
    pub fn get_move_scores(&self) -> HashMap<u8, isize> {
        let mut move_scores = HashMap::new();

        let child_iter = self.board_state.children.iter();
        let whose_turn = self.board_state.get_turn();

        for child in child_iter {
            let child_score = if whose_turn {
                how_good_is(child)
            } else {
                // Some funky handling to avoid int overflow on negating isize::MIN
                match how_good_is(child) {
                    isize::MIN => isize::MAX,
                    isize::MAX => isize::MIN,
                    score => -score,
                }
            };

            move_scores.insert(child.get_last_move(), child_score);
        }

        move_scores
    }

    /// Returns whether the game is over, and if so who won
    pub fn is_game_over(&self) -> GameOver {
        self.board_state.is_game_over()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{game_manager::GameManager, tree_analysis::how_good_is, board_state::GameOver};

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

        let manager = GameManager::start_from_position(board_array, false, 0);

        manager.generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(how_good_is(&state), isize::MIN);

        let manager = GameManager::start_from_position(board_array, true, 0);

        manager.generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(how_good_is(&state), 0);
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

        let manager = GameManager::start_from_position(board_array, false, 0);

        let manager = manager.make_move(5).unwrap();
        let manager = manager.make_move(5).unwrap_err();
        let manager = manager.make_move(4).unwrap_err();
        let manager = manager.make_move(0).unwrap_err();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap_err();
        assert_eq!(manager.is_game_over(), GameOver::OneWins);

        let manager = GameManager::start_from_position(board_array, true, 0);

        let manager = manager.make_move(5).unwrap();
        let manager = manager.make_move(5).unwrap_err();
        let manager = manager.make_move(4).unwrap_err();
        let manager = manager.make_move(0).unwrap_err();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap();
        let manager = manager.make_move(6).unwrap_err();
        assert_eq!(manager.is_game_over(), GameOver::NoWin);
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
        let real_move_scores = HashMap::new();
        real_move_scores.insert(5, isize::MAX);
        real_move_scores.insert(6, 0);
        assert_eq!(move_scores, real_move_scores);

        let manager = GameManager::start_from_position(board_array, true, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        let real_move_scores = HashMap::new();
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

        let manager = GameManager::start_from_position(board_array, false, 0);
        manager.generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_ne!(score, isize::MIN);
            } else {
                assert_eq!(score, isize::MIN);
            }
        }

        let manager = GameManager::start_from_position(board_array, true, 0);
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
