use crate::{
    board::{Board, FullColumn},
    consts::BOARD_WIDTH,
    win_check::has_color_won,
};

// TODO: Reduce the booleans into a single u8

/// A BoardState represents a single state of a possible game.
///
/// It has a board.
/// It has a number of other states:
///  is the game over, who has won, whose turn is it, etc.
/// It also has a number of possible BoardStates which could result from
///  this one, its children.
#[derive(Default, Debug)]
pub struct BoardState {
    board: Board,
    pub children: Vec<BoardState>,
    turn: bool,
    game_over: bool,
    last_move: u8,
}

impl BoardState {
    /// Constructs a new BoardState
    fn new(board: Board, turn: bool, last_move: u8) -> BoardState {
        let game_over = has_color_won(&board, !turn);

        BoardState {
            board,
            children: Vec::<BoardState>::new(),
            turn,
            game_over,
            last_move,
        }
    }

    /// Populates the children vector with new BoardStates
    pub fn generate_children(&mut self) -> &mut Vec<BoardState> {
        // If this BoardState has an already won game, no children are generated
        if let Some(_) = self.is_game_over() {
            return &mut self.children;
        }

        let turn = self.get_turn();
        let mut new_board = self.board.clone();

        // We attempt to generate a new BoardState for each column a piece
        //  can successfully be dropped down
        for col in 0..BOARD_WIDTH {
            if Err(FullColumn) == new_board.drop_piece(col, turn) {
                // If the column is full, we proceed to the next
                continue;
            } else {
                // We then add a new BoardState corresponding to the move just played
                self.children.push(BoardState::new(new_board, !turn, col));

                // We now refresh the board we're using
                new_board = self.board.clone();
            }
        }

        &mut self.children
    }

    /// Used to return the child BoardState corresponding to a particular move
    pub fn narrow_possibilities(self, col: u8) -> BoardState {
        for child in self.children {
            if child.get_last_move() == col {
                return child;
            }
        }

        panic!(
            "This BoardState: {:?} was unable to find the col {} in its children!",
            self.board, col
        );
    }

    /// Returns whose turn it is
    fn get_turn(&self) -> bool {
        self.turn
    }

    /// Returns if the game is over and who won if it is
    pub fn is_game_over(&self) -> Option<bool> {
        if self.game_over {
            Some(!self.turn)
        } else {
            None
        }
    }

    /// Returns what column the last piece was dropped in
    pub fn get_last_move(&self) -> u8 {
        self.last_move
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, OutOfBounds},
        board_state::BoardState,
        consts::BOARD_WIDTH,
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

        let mut board_state = BoardState::new(board, false, 3);

        for (i, child) in board_state.generate_children().iter().enumerate() {
            assert_eq!(child.get_last_move() as usize, i);
            assert_eq!(child.is_game_over(), None);
            assert_eq!(child.get_turn(), true);
            assert_eq!(child.children.len(), 0);

            assert_eq!(child.board.get_piece(i as u8, 0).unwrap(), false);
        }

        assert_eq!(board_state.children[3].board.get_piece(3, 4), Ok(false));

        let board = Board::from_arrays([
            [2, 0, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        let mut board_state = BoardState::new(board, false, 3);

        for child in board_state.generate_children().iter() {
            assert_eq!(child.get_last_move() as usize, 1);
            assert_eq!(child.is_game_over(), Some(false));
            assert_eq!(child.get_turn(), true);
            assert_eq!(child.children.len(), 0);

            assert_eq!(
                child.board.get_piece(child.get_last_move(), 5).unwrap(),
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

        let mut board_state = BoardState::new(board, true, 3);

        for child in board_state.generate_children().iter() {
            assert_eq!(child.is_game_over(), None);
            assert_eq!(child.get_turn(), false);
            assert_eq!(child.children.len(), 0);

            let col = child.get_last_move();
            assert_eq!(
                child
                    .board
                    .get_piece(col, child.board.get_height(col) - 1)
                    .unwrap(),
                true
            );

            if col != 0 {
                assert_eq!(child.board.get_piece(0, 3), Err(OutOfBounds));
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

        let mut board_state = BoardState::new(board, true, 3);

        for _ in board_state.generate_children().iter() {
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
            let mut board_state = BoardState::new(board.clone(), false, 3);
            for child in board_state.generate_children() {
                child.generate_children();
            }

            let mut board_clone = board.clone();
            board_clone.drop_piece(i, false).unwrap();

            board_state = board_state.narrow_possibilities(i);

            assert_eq!(board_state.board, board_clone);
            assert_eq!(board_state.is_game_over(), None);
            assert_eq!(board_state.get_turn(), true);
            assert_eq!(board_state.children.len(), 7);
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

        let mut board_state = BoardState::new(board, true, 0);
        board_state.generate_children();

        board_state.narrow_possibilities(6);
    }
}
