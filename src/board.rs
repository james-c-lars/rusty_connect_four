use crate::consts::{BOARD_HEIGHT, BOARD_WIDTH};

/// An error state for accessing a piece in a board
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfBounds,
    FullColumn,
}

#[derive(Clone, Default)]
struct Column {
    height: u8,
    piece_bitmap: u8,
}

impl Column {
    /// Gets a boolean representation of a piece from a row in the column
    /// Fails if the row requested is out of bounds
    fn get_piece(&self, row: u8) -> Result<bool, Error> {
        if row < self.height {
            Ok((self.piece_bitmap & (1 << row)) != 0)
        } else {
            Err(Error::OutOfBounds)
        }
    }

    /// Drops a new piece on top of the column corresponding to the boolean
    /// Fails if the column is already full
    fn drop_piece(&mut self, color: bool) -> Result<(), Error> {
        if self.height < BOARD_HEIGHT {
            self.piece_bitmap += (color as u8) << self.height;
            self.height += 1;

            Ok(())
        } else {
            Err(Error::FullColumn)
        }
    }

    /// Returns the height of the pieces in the column
    fn get_height(&self) -> u8 {
        self.height
    }
}

/// A connect four board
#[derive(Clone, Default)]
pub struct Board {
    columns: [Column; BOARD_WIDTH as usize],
}

impl Board {
    /// Gets a boolean representation of a piece given a column and row
    /// Fails if the row requested is out of bounds
    pub fn get_piece(&self, col: u8, row: u8) -> Result<bool, Error> {
        Ok(self.columns[col as usize].get_piece(row)?)
    }

    /// Drops a new piece on top of the given column corresponding to the boolean
    /// Fails if the column is already full
    pub fn drop_piece(&mut self, col: u8, color: bool) -> Result<(), Error> {
        Ok(self.columns[col as usize].drop_piece(color)?)
    }

    /// Returns the height of the pieces in the given column
    pub fn get_height(&self, col: u8) -> u8 {
        self.columns[col as usize].get_height()
    }

    /// Returns the height of the highest column
    pub fn get_max_height(&self) -> u8 {
        (0..BOARD_WIDTH)
            .map(|col| self.get_height(col))
            .max()
            .unwrap()
    }

    /// Used to initialize a board based on a 2d array
    /// It's meant to be used in internal testing functions and can have unexpected outputs
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
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Column, Error},
        consts::{BOARD_HEIGHT, BOARD_WIDTH},
    };

    #[test]
    fn col_get_piece() {
        let col = Column {
            height: 4,
            piece_bitmap: 0b1101,
        };

        assert_eq!(col.piece_bitmap, 13);
        assert_eq!(col.get_piece(4), Err(Error::OutOfBounds));
        assert_eq!(col.get_piece(3), Ok(true));
        assert_eq!(col.get_piece(2), Ok(true));
        assert_eq!(col.get_piece(1), Ok(false));
        assert_eq!(col.get_piece(0), Ok(true));
    }

    #[test]
    fn col_drop_piece() {
        let mut col = Column {
            height: 4,
            piece_bitmap: 0b1101,
        };

        for i in 5..=BOARD_HEIGHT {
            let color = (i % 2) == 0;

            assert_eq!(col.drop_piece(color), Ok(()));
            assert_eq!(col.height, i);
            assert_eq!(col.get_piece(i - 1), Ok(color));
            assert_eq!(col.get_piece(i), Err(Error::OutOfBounds));
        }

        assert_eq!(col.drop_piece(true), Err(Error::FullColumn));
        assert_eq!(col.height, BOARD_HEIGHT);
        assert_eq!(col.get_piece(BOARD_HEIGHT), Err(Error::OutOfBounds));
    }

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
}
