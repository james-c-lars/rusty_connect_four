use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    consts::{BOARD_HEIGHT, BOARD_WIDTH},
    game_engine::{
        board::Board, board_state::BoardState, layer_generator::LayerGenerator,
        transposition::TranspositionTable, tree_analysis::how_good_is, tree_size::calculate_size,
    },
    log::PerfTimer,
};

// Reexport GameOver
pub use crate::game_engine::{tree_size::TreeSize, win_check::GameOver};

#[derive(Debug)]
pub struct GameManager {
    board_state: Rc<RefCell<BoardState>>,
    layer_generator: LayerGenerator,
}

impl GameManager {
    /// Starts a new game with an empty board.
    pub fn new_game() -> GameManager {
        let mut table = TranspositionTable::default();
        let (state, _) = table.get_board_state(Board::default(), false);

        GameManager {
            board_state: state,
            layer_generator: LayerGenerator::new(table),
        }
    }

    /// Starts a new game from a position.
    ///
    /// The position is given as array[row][col].
    pub fn start_from_position(
        position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        turn: bool,
    ) -> GameManager {
        let mut table = TranspositionTable::default();
        let (state, _) = table.get_board_state(Board::from_arrays(position), turn);

        GameManager {
            board_state: state,
            layer_generator: LayerGenerator::new(table),
        }
    }

    /// Returns the current position of the game as array[row][col].
    pub fn get_position(&self) -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
        self.board_state.borrow().board.to_arrays()
    }

    /// Generates approximately x board states in the decision tree. Will generate less than
    /// x board states if the decision tree is completely explored.
    ///
    /// Returns the number of board states generated.
    pub fn try_generate_x_states(&mut self, x: usize) -> usize {
        let timer = PerfTimer::start(&format!("Generate {} states", x));
        let mut num_generated = 0;

        while num_generated < x {
            if let Some(num) = self.layer_generator.next() {
                num_generated += num;
            } else {
                break;
            }
        }

        timer.stop();
        num_generated
    }

    /// Drop a piece down the corresponding column.
    pub fn make_move(&mut self, col: u8) -> Result<(), String> {
        let timer = PerfTimer::start("Make Move");

        // If the game is already won, no move is valid
        if GameOver::NoWin != self.board_state.borrow().is_game_over() {
            return Err(format!("Game is already over. Can't make move: {}", col));
        }

        // We haven't yet generated the children of this board state
        if self.board_state.borrow().children.len() == 0 {
            self.try_generate_x_states(1);

            if self.board_state.borrow().children.len() == 0 {
                return Err(format!(
                    "Was unable to generate children for the root. Can't make move: {}",
                    col
                ));
            }
        }

        let mut is_valid_col = false;
        for child in self.board_state.borrow().children.iter() {
            if child.get_last_move() == col {
                is_valid_col = true;
            }
        }

        if !is_valid_col {
            return Err(format!(
                "The chosen column wasn't valid. Can't make move: {}",
                col
            ));
        }

        let sub_timer = PerfTimer::start("Make Move [Trim Tree]");
        self.board_state
            .replace(self.board_state.take().narrow_possibilities(col).take());
        sub_timer.stop();

        let sub_timer = PerfTimer::start("Make Move [Restart Layer Generator]");
        self.layer_generator.restart();
        sub_timer.stop();

        timer.stop();
        Ok(())
    }

    /// Returns a map of moves to their corresponding scores.
    ///
    /// Higher scores are better for the player about to make a move,
    ///  lower scores are better for their opponent.
    pub fn get_move_scores(&self) -> HashMap<u8, isize> {
        let timer = PerfTimer::start("Get Move Scores");

        let mut move_scores = HashMap::new();
        let mut score_table = TranspositionTable::<isize>::default();

        let borrowed_board_state = self.board_state.borrow();
        let child_iter = borrowed_board_state.children.iter();
        let whose_turn = borrowed_board_state.get_turn();

        for child in child_iter {
            let child_score = if whose_turn {
                how_good_is(&child.state.borrow(), &mut score_table)
            } else {
                // Some funky handling to avoid int overflow on negating isize::MIN
                match how_good_is(&child.state.borrow(), &mut score_table) {
                    isize::MIN => isize::MAX,
                    isize::MAX => isize::MIN,
                    score => -score,
                }
            };

            move_scores.insert(child.get_last_move(), child_score);
        }

        timer.stop();
        move_scores
    }

    /// Returns whether the game is over, and if so who won.
    pub fn is_game_over(&self) -> GameOver {
        self.board_state.borrow().is_game_over()
    }

    /// Returns the size and depth of the board.
    pub fn size(&self) -> TreeSize {
        let timer = PerfTimer::start("Get Size");

        let to_return = calculate_size(self.board_state.clone(), &self.layer_generator);

        timer.stop();
        to_return
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::game_engine::{
        game_manager::GameManager, transposition::TranspositionTable, tree_analysis::how_good_is,
        win_check::GameOver,
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

        let manager = GameManager::start_from_position(board_array, true);

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

        let mut manager = GameManager::start_from_position(board_array, false);

        manager.try_generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(
            how_good_is(&state.borrow(), &mut TranspositionTable::<isize>::default()),
            isize::MIN
        );

        let mut manager = GameManager::start_from_position(board_array, true);

        manager.try_generate_x_states(10000);

        let state = manager.board_state;

        assert_eq!(
            how_good_is(&state.borrow(), &mut TranspositionTable::<isize>::default()),
            0
        );
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

        let mut manager = GameManager::start_from_position(board_array, false);

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

        let mut manager = GameManager::start_from_position(board_array, true);

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

        let mut manager = GameManager::start_from_position(board_array, false);
        manager.try_generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        let mut real_move_scores = HashMap::new();
        real_move_scores.insert(5, isize::MAX);
        real_move_scores.insert(6, 0);
        assert_eq!(move_scores, real_move_scores);

        let mut manager = GameManager::start_from_position(board_array, true);
        manager.try_generate_x_states(10000);

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

        let mut manager = GameManager::start_from_position(board_array, false);
        manager.try_generate_x_states(10000);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_ne!(score, isize::MIN);
            } else {
                assert_eq!(score, isize::MIN);
            }
        }

        let mut manager = GameManager::start_from_position(board_array, true);
        manager.try_generate_x_states(10000);

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
