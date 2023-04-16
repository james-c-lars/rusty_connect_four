use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    consts::BOARD_WIDTH,
    game_engine::{
        board::{Board, FullColumn},
        monte_carlo::RolloutResults,
        transposition::{IsFlipped, TranspositionStateTable},
        win_check::{is_game_over, GameOver},
    },
};

/// Used to optimize alpha-beta pruning by generating moves that are most likely to be good first
const IDEAL_COLUMNS_FIRST: [u8; 7] = [3, 4, 2, 5, 1, 6, 0];

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ChildState {
    pub state: Rc<RefCell<BoardState>>,
    last_move: u8,
    is_flipped: IsFlipped,
}

impl ChildState {
    /// Gets the move that was played to reach this child.
    pub fn get_last_move(&self) -> u8 {
        self.last_move
    }

    /// Corrects this child's last move and flipped state based on the fact that its parent has
    /// flipped its orientation.
    ///
    /// Should only be used when the parent of this ChildState is the root of the decision tree and
    /// has just flipped its orientation.
    pub fn parent_flipped(&mut self) {
        self.last_move = 6 - self.last_move;
        self.is_flipped = self.is_flipped.flip();
    }
}

/// A BoardState represents a single state of a possible game.
///
/// It has a board.
/// It has a number of other states:
///  is the game over, who has won, whose turn is it, etc.
/// It also has a number of possible BoardStates which could result from
///  this one, its children.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct BoardState {
    pub board: Board,
    pub children: Vec<ChildState>,
    pub rollout_results: RolloutResults,
    turn: bool,
    game_over: GameOver,
}

impl BoardState {
    /// Constructs a new BoardState.
    pub fn new(board: Board, turn: bool) -> BoardState {
        let game_over = is_game_over(&board, turn);

        BoardState {
            board,
            children: Vec::new(),
            turn,
            game_over,
            rollout_results: RolloutResults::default(),
        }
    }

    /// Populates the children vector with new BoardStates.
    pub fn generate_children(
        &mut self,
        table: &mut TranspositionStateTable,
    ) -> Vec<Rc<RefCell<BoardState>>> {
        // If this BoardState has an already won game, no children are generated
        match self.is_game_over() {
            GameOver::NoWin => (),
            _ => return Vec::new(),
        }

        // Children can be already generated if a different transposition calulated them
        if self.children.len() > 0 {
            return Vec::new();
        }

        let turn = self.get_turn();
        let mut new_board = self.board.clone();

        // We attempt to generate a new BoardState for each column a piece
        //  can successfully be dropped down
        for col in IDEAL_COLUMNS_FIRST.iter() {
            if Err(FullColumn) == new_board.drop_piece(*col, turn) {
                // If the column is full, we proceed to the next
                continue;
            } else {
                // We then add a new BoardState corresponding to the move just played
                let (child_state, is_flipped) = table.get_board_state(new_board, !turn);
                self.children.push(ChildState {
                    state: child_state,
                    last_move: *col,
                    is_flipped,
                });

                // We now refresh the board we're using
                new_board = self.board.clone();
            }
        }

        self.children.iter().map(|c| c.state.clone()).collect()
    }

    /// Used to return the child BoardState corresponding to a particular move.
    ///
    /// Fails if the column chosen isn't an option, because it's full.
    pub fn narrow_possibilities(self, col: u8) -> Rc<RefCell<BoardState>> {
        for child in self.children {
            if child.get_last_move() == col {
                if child.is_flipped == IsFlipped::Flipped {
                    // If the child is flipped, we need to unflip it and adjust the tree
                    child.state.borrow_mut().board.flip();

                    for grandchild in child.state.borrow_mut().children.iter_mut() {
                        grandchild.parent_flipped();
                    }
                }

                return child.state;
            }
        }

        panic!(
            "This BoardState: {:?} was unable to find the col {} in its children!",
            self.board, col
        );
    }

    /// Returns whose turn it is.
    pub fn get_turn(&self) -> bool {
        self.turn
    }

    /// Returns if the game is over and who won if it is.
    pub fn is_game_over(&self) -> GameOver {
        self.game_over
    }

    /// Returns how many moves into the game this board state is
    pub fn get_depth(&self) -> u8 {
        (0..BOARD_WIDTH).map(|col| self.board.get_height(col)).sum()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        consts::BOARD_WIDTH,
        game_engine::{
            board::{Board, OutOfBounds},
            board_state::{BoardState, GameOver, IDEAL_COLUMNS_FIRST},
            transposition::TranspositionTable,
        },
    };

    #[test]
    fn generate_children() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ]);

        let mut board_state = BoardState::new(board, false);
        let mut table = TranspositionTable::default();
        board_state.generate_children(&mut table);

        for (i, child) in board_state.children.iter().enumerate() {
            assert_eq!(
                child.get_last_move() as usize,
                IDEAL_COLUMNS_FIRST[i] as usize
            );
            assert_eq!(child.state.borrow().is_game_over(), GameOver::NoWin);
            assert_eq!(child.state.borrow().get_turn(), true);
            assert_eq!(child.state.borrow().children.len(), 0);

            assert_eq!(child.state.borrow().board.get_piece(3, 0).unwrap(), false);
        }

        assert_eq!(
            // Here the 0th child is really column 4, due to the alpha-beta move generation optimization
            board_state.children[0].state.borrow().board.get_piece(3, 4),
            Ok(false)
        );

        let board = Board::from_arrays([
            [2, 0, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        let mut board_state = BoardState::new(board, true);
        let mut table = TranspositionTable::default();
        board_state.generate_children(&mut table);

        for child in board_state.children.iter() {
            assert_eq!(child.get_last_move() as usize, 1);
            assert_eq!(child.state.borrow().is_game_over(), GameOver::Tie);
            assert_eq!(child.state.borrow().get_turn(), false);
            assert_eq!(child.state.borrow().children.len(), 0);

            assert_eq!(
                child
                    .state
                    .borrow()
                    .board
                    .get_piece(child.get_last_move(), 5)
                    .unwrap(),
                true
            );
        }

        let board = Board::from_arrays([
            [2, 0, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        let mut board_state = BoardState::new(board, false);
        let mut table = TranspositionTable::default();
        board_state.generate_children(&mut table);

        for child in board_state.children.iter() {
            assert_eq!(child.get_last_move() as usize, 1);
            assert_eq!(child.state.borrow().is_game_over(), GameOver::OneWins);
            assert_eq!(child.state.borrow().get_turn(), true);
            assert_eq!(child.state.borrow().children.len(), 0);

            assert_eq!(
                child
                    .state
                    .borrow()
                    .board
                    .get_piece(child.get_last_move(), 5)
                    .unwrap(),
                false
            );
        }

        assert_eq!(board_state.children.len(), 1);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 0, 0, 0, 1, 0],
            [1, 0, 0, 0, 0, 2, 0],
            [1, 1, 0, 0, 0, 2, 0],
            [1, 1, 1, 0, 0, 2, 0],
        ]);

        let mut board_state = BoardState::new(board, true);
        let mut table = TranspositionTable::default();
        board_state.generate_children(&mut table);

        for child in board_state.children.iter() {
            assert_eq!(child.state.borrow().is_game_over(), GameOver::NoWin);
            assert_eq!(child.state.borrow().get_turn(), false);
            assert_eq!(child.state.borrow().children.len(), 0);

            let col = child.get_last_move();
            assert_eq!(
                child
                    .state
                    .borrow()
                    .board
                    .get_piece(col, child.state.borrow().board.get_height(col) - 1)
                    .unwrap(),
                true
            );

            if col != 0 {
                assert_eq!(child.state.borrow().board.get_piece(0, 3), Err(OutOfBounds));
            }
        }

        assert_eq!(board_state.children.len(), 6);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 0, 0, 0],
        ]);

        let mut board_state = BoardState::new(board, true);
        let mut table = TranspositionTable::default();

        for _ in board_state.generate_children(&mut table).iter() {
            panic!("A winning game should never generate children!");
        }

        assert_eq!(board_state.children.len(), 0);
    }

    #[test]
    fn narrow_possibilities() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ]);

        for i in 0..BOARD_WIDTH {
            let mut board_state: Rc<RefCell<BoardState>> =
                RefCell::new(BoardState::new(board.clone(), false)).into();
            let mut table = TranspositionTable::default();
            board_state.borrow_mut().generate_children(&mut table);

            for child in board_state.borrow().children.iter() {
                child.state.borrow_mut().generate_children(&mut table);
            }

            let mut board_clone = board.clone();
            board_clone.drop_piece(i, false).unwrap();

            board_state = board_state.take().narrow_possibilities(i);

            assert_eq!(board_state.borrow().board, board_clone);
            assert_eq!(board_state.borrow().is_game_over(), GameOver::NoWin);
            assert_eq!(board_state.borrow().get_turn(), true);
            assert_eq!(board_state.borrow().children.len(), 7);
        }
    }

    #[test]
    #[should_panic]
    fn narrow_possibilities_panic() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 1, 0, 2],
            [0, 0, 0, 1, 2, 0, 2],
            [0, 0, 2, 1, 1, 0, 1],
            [0, 1, 1, 2, 1, 0, 1],
            [0, 2, 1, 1, 2, 0, 1],
        ]);

        let mut board_state = BoardState::new(board, true);
        let mut table = TranspositionTable::default();
        board_state.generate_children(&mut table);

        board_state.narrow_possibilities(6);
    }

    #[test]
    fn get_depth() {
        let mut board_state = BoardState::new(Board::default(), false);

        for i in 0..21 {
            assert_eq!(i, board_state.get_depth());
            board_state.board.drop_piece(i % 7, (i % 2) == 0).unwrap();
        }
    }

    #[test]
    fn gameover_conversion() {
        for i in 0..4 {
            let game = GameOver::from(i);
            let new_i = game as u8;
            assert_eq!(i, new_i);
        }
    }
}
