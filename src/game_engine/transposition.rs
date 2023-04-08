use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::game_engine::{board::Board, board_state::BoardState};

/// Represents whether a transposition has had its X axis flipped.
#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub enum IsFlipped {
    #[default]
    Normal,
    Flipped,
}

impl IsFlipped {
    /// Flips the orientation.
    pub fn flip(&self) -> IsFlipped {
        match *self {
            IsFlipped::Normal => IsFlipped::Flipped,
            IsFlipped::Flipped => IsFlipped::Normal,
        }
    }
}

/// A table with weak references to every board state that has been created. Will consider symmetrical board
/// states to be the same.
#[derive(Default, Debug)]
pub struct TranspositionTable {
    table: HashMap<u64, Weak<RefCell<BoardState>>>,
}

impl TranspositionTable {
    /// Using a board, gets a corresponding BoardState transposition.
    ///
    /// The IsFlipped return value represents whether the returned transposition is horizontally flipped.
    pub fn get_board_state(
        &mut self,
        board: Board,
        turn: bool,
    ) -> (Rc<RefCell<BoardState>>, IsFlipped) {
        // First check for the non-flipped board
        let mut hasher = DefaultHasher::new();
        board.iter().collect::<Vec<u8>>().hash(&mut hasher);
        let normal_hash = hasher.finish();
        if let Some(board_state_weak) = self.table.get(&normal_hash) {
            if let Some(board_state) = board_state_weak.upgrade() {
                assert_eq!(
                    board_state.borrow().get_turn(),
                    turn,
                    "board: {:?} turn: {} doesn't match turn of {:?}",
                    board,
                    turn,
                    board_state.borrow()
                );

                return (board_state, IsFlipped::Normal);
            }
        }

        // Next we can check for the flipped board
        let mut hasher = DefaultHasher::new();
        board.flipped_iter().collect::<Vec<u8>>().hash(&mut hasher);
        let flipped_hash = hasher.finish();
        if let Some(board_state_weak) = self.table.get(&flipped_hash) {
            if let Some(board_state) = board_state_weak.upgrade() {
                assert_eq!(
                    board_state.borrow().get_turn(),
                    turn,
                    "board: {:?} turn: {} doesn't match turn of {:?}",
                    board,
                    turn,
                    board_state.borrow()
                );

                return (board_state, IsFlipped::Flipped);
            }
        }

        // The board we're evaluating is not in the Transposition table, so construct a new BoardState
        let board_state = Rc::new(RefCell::new(BoardState::new(board, turn)));
        self.table.insert(normal_hash, Rc::downgrade(&board_state));

        (board_state, IsFlipped::Normal)
    }

    /// Removes unreachable board states from the transposition table.
    pub fn clean(&mut self) {
        for (key, weak_ref) in self
            .table
            .iter()
            .map(|(k, r)| (*k, r.clone()))
            .collect::<Vec<(u64, Weak<RefCell<BoardState>>)>>()
        {
            if weak_ref.strong_count() == 0 {
                self.table.remove(&key);
            }
        }
    }

    /// Gets an iterator to the contents of the transposition table.
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Weak<RefCell<BoardState>>)> + '_ {
        self.table.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::game_engine::{
        board::Board,
        transposition::{IsFlipped, TranspositionTable},
    };

    #[test]
    fn transposes() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 2, 0, 1, 0, 2, 1],
            [0, 1, 2, 1, 0, 1, 2],
            [0, 1, 2, 1, 2, 1, 2],
        ]);

        let flipped_board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 1, 0, 0, 0],
            [1, 2, 0, 1, 0, 2, 0],
            [2, 1, 0, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ]);

        let mut table = TranspositionTable::default();

        let (state, state_is_flipped) = table.get_board_state(board, false);
        let (flipped, flipped_is_flipped) = table.get_board_state(flipped_board, false);

        assert_eq!(state, flipped);
        assert_eq!(state_is_flipped, IsFlipped::Normal);
        assert_eq!(flipped_is_flipped, IsFlipped::Flipped);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 2, 0, 1, 0, 2, 1],
            [0, 1, 2, 1, 0, 1, 2],
            [0, 1, 2, 1, 2, 1, 2],
        ]);

        let (clone, clone_is_flipped) = table.get_board_state(board, false);
        assert_eq!(state, clone);
        assert_eq!(clone_is_flipped, IsFlipped::Normal);
    }

    #[test]
    fn new_reference() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 2, 0, 1, 0, 2, 1],
            [0, 1, 2, 1, 0, 1, 2],
            [0, 1, 2, 1, 2, 1, 2],
        ]);

        let mut table = TranspositionTable::default();

        let (state, _) = table.get_board_state(board, false);
        drop(state);

        let flipped_board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 1, 0, 0, 0],
            [1, 2, 0, 1, 0, 2, 0],
            [2, 1, 0, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ]);

        let (_, is_flipped) = table.get_board_state(flipped_board, false);
        assert_eq!(is_flipped, IsFlipped::Normal);
    }

    #[test]
    fn clean_table() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 2, 0, 1, 0, 2, 1],
            [0, 1, 2, 1, 0, 1, 2],
            [0, 1, 2, 1, 2, 1, 2],
        ]);

        let mut table = TranspositionTable::default();

        let (state, _) = table.get_board_state(board, false);
        drop(state);

        table.clean();

        assert_eq!(table.table.len(), 0);
    }
}
