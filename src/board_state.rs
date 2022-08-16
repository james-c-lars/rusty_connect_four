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
    fn new(
        board: Board,
        children: Vec<BoardState>,
        turn: bool,
        game_over: bool,
        last_move: u8,
    ) -> BoardState {
        BoardState {
            board,
            children,
            turn,
            game_over,
            last_move,
        }
    }

    /// Populates the children vector with new BoardStates
    pub fn generate_children(&mut self) -> &Vec<BoardState> {
        // If this BoardState has an already won game, no children are generated
        if let Some(_) = self.is_game_over() {
            return &self.children;
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
                // If it was successful, we check to see if the move won the game
                let game_over = has_color_won(&new_board, turn);

                // We then add a new BoardState corresponding to the move just played
                self.children.push(BoardState::new(
                    new_board,
                    Vec::<BoardState>::new(),
                    !turn,
                    game_over,
                    col,
                ));

                // We now refresh the board we're using
                new_board = self.board.clone();
            }
        }

        &self.children
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
    #[test]
    fn generate_children() {
        todo!();
    }

    #[test]
    fn narrow_possibilities() {
        todo!();
    }
}
