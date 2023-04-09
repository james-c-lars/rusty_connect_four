use crate::consts::{BOARD_HEIGHT, BOARD_WIDTH};

/// An error state when accessing a nonexistant piece.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OutOfBounds;

/// An error state when dropping a piece in a full column.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FullColumn;

/// A connect four board.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Board {
    column_heights: [u8; BOARD_WIDTH as usize],
    column_bitmaps: [u8; BOARD_WIDTH as usize],
}

impl Board {
    /// Gets a boolean representation of a piece given a column and row.
    ///
    /// Fails if the row requested is out of bounds.
    pub fn get_piece(&self, col: u8, row: u8) -> Result<bool, OutOfBounds> {
        if row < self.get_height(col) {
            Ok((self.column_bitmaps[col as usize] & (1 << row)) != 0)
        } else {
            Err(OutOfBounds)
        }
    }

    /// Drops a new piece on top of the given column corresponding to the boolean.
    ///
    /// Fails if the column is already full.
    pub fn drop_piece(&mut self, col: u8, color: bool) -> Result<(), FullColumn> {
        let col_height = self.get_height(col);
        if col_height < BOARD_HEIGHT {
            self.column_bitmaps[col as usize] += (color as u8) << col_height;
            self.set_height(col, col_height + 1);

            Ok(())
        } else {
            Err(FullColumn)
        }
    }

    /// Returns the height of the pieces in the given column.
    pub fn get_height(&self, col: u8) -> u8 {
        self.column_heights[col as usize]
    }

    /// Sets the height of the given column.
    fn set_height(&mut self, col: u8, height: u8) {
        self.column_heights[col as usize] = height;
    }

    /// Returns the height of the highest column.
    pub fn get_max_height(&self) -> u8 {
        (0..BOARD_WIDTH)
            .map(|col| self.get_height(col))
            .max()
            .unwrap()
    }

    /// Returns if the board is full.
    pub fn is_full(&self) -> bool {
        for col in 0..BOARD_WIDTH {
            if self.get_height(col) != BOARD_HEIGHT {
                return false;
            }
        }
        return true;
    }

    /// Gets an iterator over the board's contents. Used for hashing the board.
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.column_heights
            .iter()
            .chain(self.column_bitmaps.iter())
            .map(|i| *i)
    }

    /// Gets an iterator over the board's content reversed symetrically. Used for hashing the board.
    pub fn flipped_iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.column_heights
            .iter()
            .rev()
            .chain(self.column_bitmaps.iter().rev())
            .map(|i| *i)
    }

    /// Flips this Board horizontally.
    pub fn flip(&mut self) {
        for (i, val) in self.column_heights.into_iter().rev().enumerate() {
            self.column_heights[i] = val;
        }
        for (i, val) in self.column_bitmaps.into_iter().rev().enumerate() {
            self.column_bitmaps[i] = val;
        }
    }

    /// Used to initialize a board based on a 2d array.
    ///
    /// If the board contains floating pieces, it will have unexpected results.
    pub fn from_arrays(arrays: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize]) -> Board {
        let mut board = Board::default();

        for row in arrays.iter().rev() {
            for (col, piece) in row.iter().enumerate() {
                if *piece == 1 {
                    board.drop_piece(col as u8, false).unwrap();
                } else if *piece == 2 {
                    board.drop_piece(col as u8, true).unwrap();
                } else if *piece > 2 {
                    panic!("No value in the given array should be greater than 2.");
                }
            }
        }

        board
    }

    /// Used to get the current state of the board as a 2d array.
    pub fn to_arrays(&self) -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
        let mut position = [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                position[(BOARD_HEIGHT - 1 - row) as usize][col as usize] =
                    match self.get_piece(col, row) {
                        Ok(piece) => piece as u8 + 1,
                        Err(_) => 0,
                    };
            }
        }

        position
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::{BOARD_HEIGHT, BOARD_WIDTH},
        game_engine::board::{Board, FullColumn, OutOfBounds},
    };

    #[test]
    fn from_arrays() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2, 1],
            [0, 1, 2, 0, 0, 1, 2],
            [0, 1, 2, 0, 2, 1, 2],
        ]);

        let ones: [(u8, u8); 6] = [(1, 0), (1, 1), (5, 0), (5, 1), (6, 2), (6, 3)];
        let twos: [(u8, u8); 9] = [
            (1, 2),
            (2, 0),
            (2, 1),
            (4, 0),
            (5, 2),
            (6, 0),
            (6, 1),
            (6, 4),
            (6, 5),
        ];

        for col in 0..BOARD_WIDTH {
            for row in 0..BOARD_HEIGHT {
                match board.get_piece(col, row) {
                    Ok(piece) => {
                        if piece {
                            assert!(twos.contains(&(col, row)));
                        } else {
                            assert!(ones.contains(&(col, row)));
                        }
                    }
                    Err(_) => {
                        assert!(!ones.contains(&(col, row)));
                        assert!(!twos.contains(&(col, row)));
                    }
                }
            }
        }
    }

    #[test]
    fn get_piece() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2, 1],
            [0, 1, 2, 0, 0, 1, 2],
            [0, 1, 2, 0, 2, 1, 2],
        ]);

        assert_eq!(board.column_bitmaps[0], 0);
        assert_eq!(board.get_piece(0, 5), Err(OutOfBounds));
        assert_eq!(board.get_piece(0, 3), Err(OutOfBounds));
        assert_eq!(board.get_piece(0, 2), Err(OutOfBounds));
        assert_eq!(board.get_piece(0, 1), Err(OutOfBounds));
        assert_eq!(board.get_piece(0, 0), Err(OutOfBounds));

        assert_eq!(board.column_bitmaps[1], 4);
        assert_eq!(board.get_piece(1, 5), Err(OutOfBounds));
        assert_eq!(board.get_piece(1, 3), Err(OutOfBounds));
        assert_eq!(board.get_piece(1, 2), Ok(true));
        assert_eq!(board.get_piece(1, 1), Ok(false));
        assert_eq!(board.get_piece(1, 0), Ok(false));

        assert_eq!(board.column_bitmaps[6], 51);
        assert_eq!(board.get_piece(6, 5), Ok(true));
        assert_eq!(board.get_piece(6, 3), Ok(false));
        assert_eq!(board.get_piece(6, 2), Ok(false));
        assert_eq!(board.get_piece(6, 1), Ok(true));
        assert_eq!(board.get_piece(6, 0), Ok(true));
    }

    #[test]
    fn drop_piece() {
        let mut board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2, 1],
            [0, 1, 2, 0, 0, 1, 2],
            [0, 1, 2, 0, 2, 1, 2],
        ]);

        for i in 1..=BOARD_HEIGHT {
            let color = (i % 2) == 0;

            assert_eq!(board.drop_piece(3, color), Ok(()));
            assert_eq!(board.get_height(3), i);
            assert_eq!(board.get_piece(3, i - 1), Ok(color));
            assert_eq!(board.get_piece(3, i), Err(OutOfBounds));
        }

        assert_eq!(board.drop_piece(3, true), Err(FullColumn));
        assert_eq!(board.get_height(3), BOARD_HEIGHT);
        assert_eq!(board.get_piece(3, BOARD_HEIGHT), Err(OutOfBounds));
    }

    #[test]
    fn get_max_height() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ]);

        assert_eq!(board.get_max_height(), 0);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 2, 0, 0],
            [0, 0, 1, 0, 2, 0, 0],
        ]);

        assert_eq!(board.get_max_height(), 2);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 2, 0, 0, 0],
            [1, 0, 0, 2, 0, 1, 0],
        ]);

        assert_eq!(board.get_max_height(), 4);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 2],
        ]);

        assert_eq!(board.get_max_height(), 6);
    }

    #[test]
    fn board_flip() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 2, 0, 1, 0, 2, 1],
            [0, 1, 2, 1, 0, 1, 2],
            [0, 1, 2, 1, 2, 1, 2],
        ]);

        let mut flipped_board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 1, 0, 0, 0],
            [1, 2, 0, 1, 0, 2, 0],
            [2, 1, 0, 1, 2, 1, 0],
            [2, 1, 2, 1, 2, 1, 0],
        ]);
        flipped_board.flip();

        assert_eq!(board, flipped_board);
    }
}
