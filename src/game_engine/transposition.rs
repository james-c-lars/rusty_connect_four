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

/// A table with values that can be accessed by using a Board. Will consider symmetrical
///  boards to be the same.
#[derive(Default, Debug)]
pub struct TranspositionTable<T> {
    table: HashMap<u64, T>,
}

/// Used to get the normal hash of a board.
fn normal_hash(board: &Board) -> u64 {
    let mut hasher = DefaultHasher::new();
    board.iter().collect::<Vec<u8>>().hash(&mut hasher);
    hasher.finish()
}

/// Used to get the hash of a flipped board.
fn flipped_hash(board: &Board) -> u64 {
    let mut hasher = DefaultHasher::new();
    board.flipped_iter().collect::<Vec<u8>>().hash(&mut hasher);
    hasher.finish()
}

impl<T> TranspositionTable<T> {
    /// Gets a value in the table corresponding to a board.
    pub fn get_transposed(&mut self, board: &Board) -> Option<(&T, IsFlipped)> {
        let normal = normal_hash(&board);
        if let Some(value) = self.table.get(&normal) {
            return Some((value, IsFlipped::Normal));
        }

        let flipped = flipped_hash(&board);
        if let Some(value) = self.table.get(&flipped) {
            return Some((value, IsFlipped::Flipped));
        }

        None
    }

    /// Inserts a key value pair into the transposition table.
    pub fn insert(&mut self, board: &Board, value: T) {
        self.table.insert(normal_hash(board), value);
    }

    /// Gets an iterator to the contents of the transposition table.
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &T)> + '_ {
        self.table.iter()
    }

    /// Gets how many entries are in the table.
    pub fn len(&self) -> usize {
        self.table.len()
    }
}

/// A table with weak references to every board state that has been created. Will consider symmetrical board
/// states to be the same.
pub type TranspositionStateTable = TranspositionTable<Weak<RefCell<BoardState>>>;

impl TranspositionStateTable {
    /// Using a board, gets a corresponding BoardState transposition.
    ///
    /// The IsFlipped return value represents whether the returned transposition is horizontally flipped.
    pub fn get_board_state(
        &mut self,
        board: Board,
        turn: bool,
    ) -> (Rc<RefCell<BoardState>>, IsFlipped) {
        if let Some((board_state_weak, is_flipped)) = self.get_transposed(&board) {
            if let Some(board_state) = board_state_weak.upgrade() {
                assert_eq!(
                    board_state.borrow().get_turn(),
                    turn,
                    "board: {:?} turn: {} doesn't match turn of {:?}",
                    board,
                    turn,
                    board_state.borrow()
                );

                return (board_state, is_flipped);
            }
        }

        // The board we're evaluating is not in the Transposition table, so construct a new BoardState
        let board_state = Rc::new(RefCell::new(BoardState::new(board, turn)));
        let normal = normal_hash(&board_state.borrow().board);
        self.table.insert(normal, Rc::downgrade(&board_state));

        (board_state, IsFlipped::Normal)
    }

    /// Removes unreachable board states from the transposition table.
    pub fn clean(&mut self) {
        self.table.retain(|_, r| r.strong_count() != 0);
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
