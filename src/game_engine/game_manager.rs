use std::{cell::RefCell, collections::HashMap, mem::size_of, rc::Rc, time::{Instant, Duration}};

use rand::thread_rng;

use crate::{
    consts::{BOARD_HEIGHT, BOARD_WIDTH},
    game_engine::{
        board::Board,
        board_state::{BoardState, ChildState},
        monte_carlo::RolloutResults,
        transposition::TranspositionStateTable,
        tree_size::calculate_size,
    },
    log::{log_message, LogType, PerfTimer},
};

// Reexport GameOver
pub use crate::game_engine::{tree_size::TreeSize, win_check::GameOver};

#[derive(Debug)]
pub struct GameManager {
    board_state: Rc<RefCell<BoardState>>,
    table: TranspositionStateTable,
}

impl GameManager {
    /// Starts a new game with an empty board.
    pub fn new_game() -> GameManager {
        let mut table = TranspositionStateTable::default();
        let (state, _) = table.get_board_state(Board::default(), false);

        log_message(
            LogType::SizeOfData,
            format!(
                "Sizes - Board: {}, Board State: {}, ChildState: {}, Rollout Results: {}",
                size_of::<Board>(),
                size_of::<BoardState>(),
                size_of::<ChildState>(),
                size_of::<RolloutResults>(),
            ),
        );

        GameManager {
            board_state: state,
            table,
        }
    }

    /// Starts a new game from a position.
    ///
    /// The position is given as array[row][col].
    pub fn start_from_position(
        position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        turn: bool,
    ) -> GameManager {
        let mut table = TranspositionStateTable::default();
        let (state, _) = table.get_board_state(Board::from_arrays(position), turn);

        GameManager {
            board_state: state,
            table,
        }
    }

    /// Returns the current position of the game as array[row][col].
    pub fn get_position(&self) -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
        self.board_state.borrow().board.to_arrays()
    }

    /// Explores the decision tree for x seconds. It does this by generating rollouts from
    ///  the root of the tree.
    /// 
    /// x is the number of seconds to spend generating states.
    pub fn explore_for_x_secs(&mut self, x: f32) {
        let timer = PerfTimer::start(&format!("Generate {} states", x));
        let mut thread_rng = thread_rng();
        let mut root = self.board_state.borrow_mut();

        let start = Instant::now();
        loop {
            for _ in 0..128 {
                root.generate_rollouts(&mut self.table, &mut thread_rng);
            }

            if start.elapsed() < Duration::from_secs_f32(x) {
                break;
            }
        }

        timer.stop();
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
            self.board_state.borrow_mut().generate_children(&mut self.table);

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

        let sub_timer = PerfTimer::start("Make Move [Clean Transposition Table]");
        self.table.clean();
        sub_timer.stop();

        timer.stop();
        Ok(())
    }

    /// Returns a map of moves to their corresponding scores.
    ///
    /// Higher scores are better for the player about to make a move,
    ///  lower scores are better for their opponent.
    pub fn get_move_scores(&self) -> HashMap<u8, f32> {
        let timer = PerfTimer::start("Get Move Scores");

        let move_scores = self.board_state.borrow().move_scores();

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

        let to_return = calculate_size(self.board_state.clone(), &self.table);

        timer.stop();
        to_return
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::game_engine::{
        game_manager::GameManager,
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

        manager.explore_for_x_secs(0.5);

        assert_eq!(
            *manager.get_move_scores().get(&5).unwrap(),
            1.0
        );

        let mut manager = GameManager::start_from_position(board_array, true);

        manager.explore_for_x_secs(0.5);

        assert_eq!(
            *manager.get_move_scores().get(&5).unwrap(),
            0.0
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
        manager.explore_for_x_secs(0.5);

        let move_scores = manager.get_move_scores();
        let mut real_move_scores = HashMap::new();
        real_move_scores.insert(5, 1.0);
        real_move_scores.insert(6, 0.0);
        assert_eq!(move_scores, real_move_scores);

        let mut manager = GameManager::start_from_position(board_array, true);
        manager.explore_for_x_secs(0.5);

        let move_scores = manager.get_move_scores();
        let mut real_move_scores = HashMap::new();
        real_move_scores.insert(5, 0.0);
        real_move_scores.insert(6, 0.0);
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
        manager.explore_for_x_secs(0.5);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_ne!(score, 0.0);
            } else {
                assert!(score < 0.1);
            }
        }

        let mut manager = GameManager::start_from_position(board_array, true);
        manager.explore_for_x_secs(0.5);

        let move_scores = manager.get_move_scores();
        for (col, score) in move_scores {
            if col == 3 {
                assert_eq!(score, 1.0);
            } else {
                assert_ne!(score, 1.0);
            }
        }
    }
}
