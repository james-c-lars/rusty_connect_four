use std::rc::Rc;

use crate::{
    board::{Board, FullColumn},
    consts::BOARD_WIDTH,
    win_check::has_color_won,
};

/// This represents whether the game is over, and if so how
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum GameOver {
    NoWin,
    Tie,
    OneWins,
    TwoWins,
}

impl From<u8> for GameOver {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::NoWin,
            1 => Self::Tie,
            2 => Self::OneWins,
            3 => Self::TwoWins,
            _ => panic!("Tried to convert a number greater than 3 to a GameOver enum"),
        }
    }
}

/// A BoardState represents a single state of a possible game.
///
/// It has a board.
/// It has a number of other states:
///  is the game over, who has won, whose turn is it, etc.
/// It also has a number of possible BoardStates which could result from
///  this one, its children.
#[derive(Default, Debug)]
pub struct BoardState {
    pub board: Board,
    pub children: Vec<Rc<BoardState>>,
    last_move: u8,
    metadata: u8,
}

impl BoardState {
    /// Constructs a new BoardState.
    pub fn new(board: Board, turn: bool, last_move: u8) -> BoardState {
        let game_over = if has_color_won(&board, !turn) {
            match !turn {
                false => GameOver::OneWins,
                true => GameOver::TwoWins,
            }
        } else if board.is_full() {
            GameOver::Tie
        } else {
            GameOver::NoWin
        };

        let metadata = (turn as u8) + ((game_over as u8) << 4);

        BoardState {
            board,
            children: Vec::new(),
            last_move,
            metadata,
        }
    }

    pub const fn default_const() -> BoardState {
        BoardState {
            board: Board::default_const(),
            children: Vec::new(),
            last_move: 0,
            metadata: 0,
        }
    }

    /// Populates the children vector with new BoardStates.
    pub fn generate_children(&mut self) -> Vec<Rc<BoardState>> {
        // If this BoardState has an already won game, no children are generated
        match self.is_game_over() {
            GameOver::NoWin => (),
            _ => return self.children.clone(),
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
                self.children.push(BoardState::new(new_board, !turn, col).into());

                // We now refresh the board we're using
                new_board = self.board.clone();
            }
        }

        self.children.clone()
    }

    /// Used to return the child BoardState corresponding to a particular move.
    ///
    /// Fails if the column chosen isn't an option, because it's full.
    pub fn narrow_possibilities(self, col: u8) -> Rc<BoardState> {
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

    /// Returns whose turn it is.
    pub fn get_turn(&self) -> bool {
        self.metadata % 2 == 1
    }

    /// Returns if the game is over and who won if it is.
    pub fn is_game_over(&self) -> GameOver {
        GameOver::from(self.metadata >> 4)
    }

    /// Returns what column the last piece was dropped in.
    pub fn get_last_move(&self) -> u8 {
        self.last_move
    }

    /// Returns how many moves into the game this board state is
    pub fn get_depth(&self) -> u8 {
        (0..BOARD_WIDTH).map(|col| self.board.get_height(col)).sum()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        board::{Board, OutOfBounds},
        board_state::{BoardState, GameOver},
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
            assert_eq!(child.is_game_over(), GameOver::NoWin);
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

        let mut board_state = BoardState::new(board, true, 3);

        for child in board_state.generate_children().iter() {
            assert_eq!(child.get_last_move() as usize, 1);
            assert_eq!(child.is_game_over(), GameOver::Tie);
            assert_eq!(child.get_turn(), false);
            assert_eq!(child.children.len(), 0);

            assert_eq!(
                child.board.get_piece(child.get_last_move(), 5).unwrap(),
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

        let mut board_state = BoardState::new(board, false, 3);

        for child in board_state.generate_children().iter() {
            assert_eq!(child.get_last_move() as usize, 1);
            assert_eq!(child.is_game_over(), GameOver::OneWins);
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
            assert_eq!(child.is_game_over(), GameOver::NoWin);
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
            let mut board_state: Rc<BoardState> = BoardState::new(board.clone(), false, 3).into();
            for child in board_state.generate_children() {
                child.generate_children();
            }

            let mut board_clone = board.clone();
            board_clone.drop_piece(i, false).unwrap();

            board_state = board_state.narrow_possibilities(i);

            assert_eq!(board_state.board, board_clone);
            assert_eq!(board_state.is_game_over(), GameOver::NoWin);
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

    #[test]
    fn get_depth() {
        let mut board_state = BoardState::new(Board::default(), false, 0);

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
