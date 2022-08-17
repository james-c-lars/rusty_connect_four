use std::cmp::{max, min};

use crate::{
    board::{Board, OutOfBounds},
    consts::{BOARD_HEIGHT, BOARD_WIDTH, NUMBER_TO_WIN},
};

/// Iterates through a single horizontal strip of a board.
/// 
/// Acquired from a HorizontalStripIter.
pub struct HorizontalIter<'a> {
    board: &'a Board,
    col: u8,
    row: u8,
}

impl Iterator for HorizontalIter<'_> {
    type Item = Result<bool, OutOfBounds>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col < BOARD_WIDTH {
            let result = Some(self.board.get_piece(self.col, self.row));
            self.col += 1;

            result
        } else {
            None
        }
    }
}

impl ExactSizeIterator for HorizontalIter<'_> {
    fn len(&self) -> usize {
        (BOARD_WIDTH - self.col) as usize
    }
}

/// Iterates through the different horizontal strips of a board.
/// 
/// Yields a HorizontalIter to each strip until the max_height of the board is reached.
pub struct HorizontalStripIter<'a> {
    board: &'a Board,
    max_height: u8,
    row: u8,
}

impl<'a> Iterator for HorizontalStripIter<'a> {
    type Item = HorizontalIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.max_height {
            let result = Some(HorizontalIter {
                board: self.board,
                col: 0,
                row: self.row,
            });
            self.row += 1;

            result
        } else {
            None
        }
    }
}

/// Iterates through a single vertical strip of a board until the max_height of the board is reached.
/// 
/// Acquired from a VerticalStripIter.
pub struct VerticalIter<'a> {
    board: &'a Board,
    max_height: u8,
    col: u8,
    row: u8,
}

impl Iterator for VerticalIter<'_> {
    type Item = Result<bool, OutOfBounds>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.max_height {
            let result = Some(self.board.get_piece(self.col, self.row));
            self.row += 1;

            result
        } else {
            None
        }
    }
}

impl ExactSizeIterator for VerticalIter<'_> {
    fn len(&self) -> usize {
        (self.max_height - self.row) as usize
    }
}

/// Iterates through the different vertical strips of a board.
/// 
/// Yields a VerticalIter for each strip, exlcuding empty columns.
pub struct VerticalStripIter<'a> {
    board: &'a Board,
    col: u8,
    full: bool,
}

impl<'a> Iterator for VerticalStripIter<'a> {
    type Item = VerticalIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col < BOARD_WIDTH {
            // Calculating the max_height that the new iterator should have
            let mut col_height = self.board.get_height(self.col);

            // We can also use this value to determine if we should skip a column
            // This changes based on if we're creating full iterators or not
            if col_height == 0 || (!self.full && col_height < NUMBER_TO_WIN) {
                self.col += 1;
                return self.next();
            }

            // If we do want full iterators, then we want to iterate until we're only getting
            //  empty pieces and then stop
            // If we don't want full iterators then we want to stop at the first empty piece
            if self.full {
                col_height = min(col_height + NUMBER_TO_WIN - 1, BOARD_HEIGHT);
            }

            let result = Some(VerticalIter {
                board: self.board,
                max_height: col_height,
                col: self.col,
                row: 0,
            });
            self.col += 1;

            result
        } else {
            None
        }
    }
}

/// Iterates through a single upward diagonal strip of a board.
/// 
/// Acquired from a UpwardDiagonalStripIter.
pub struct UpwardDiagonalIter<'a> {
    board: &'a Board,
    max_height: u8,
    col: u8,
    row: u8,
}

impl Iterator for UpwardDiagonalIter<'_> {
    type Item = Result<bool, OutOfBounds>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col < BOARD_WIDTH && self.row < self.max_height {
            let result = Some(self.board.get_piece(self.col, self.row));
            self.col += 1;
            self.row += 1;

            result
        } else {
            None
        }
    }
}

/// Iterates through the different upward diagonal strips of a board.
/// 
/// Yields a UpwardDiagonalIter to each strip of size >= NUMBER_TO_WIN until the max_height
///  of the board is reached.
pub struct UpwardDiagonalStripIter<'a> {
    board: &'a Board,
    max_height: u8,
    col: u8,
    row: u8,
    full: bool,
}

impl<'a> Iterator for UpwardDiagonalStripIter<'a> {
    type Item = UpwardDiagonalIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop iteration if either we go out of bounds,
        //  or if the iterator isn't full and there aren't enough pieces for a connect four
        if self.col + NUMBER_TO_WIN > BOARD_WIDTH || (!self.full && self.max_height < NUMBER_TO_WIN)
        {
            return None;
        }

        let result = Some(UpwardDiagonalIter {
            board: self.board,
            max_height: self.max_height,
            col: self.col,
            row: self.row,
        });

        if self.row > 0 {
            self.row -= 1;
        } else {
            self.col += 1;
        }

        result
    }
}

impl ExactSizeIterator for UpwardDiagonalIter<'_> {
    fn len(&self) -> usize {
        min(self.max_height - self.row, BOARD_WIDTH - self.col) as usize
    }
}

/// Iterates through a single downward diagonal strip of a board.
/// 
/// Acquired from a DownwardDiagonalStripIter.
pub struct DownwardDiagonalIter<'a> {
    board: &'a Board,
    max_height: u8,
    col: u8,
    row: u8,
}

impl Iterator for DownwardDiagonalIter<'_> {
    type Item = Result<bool, OutOfBounds>;

    fn next(&mut self) -> Option<Self::Item> {
        // self.col is effectively one ahead of where it should really be
        // this is to avoid needing to look for u8 underflows when we do 0 - 1
        if self.col > 0 && self.row < self.max_height {
            self.col -= 1;
            let result = Some(self.board.get_piece(self.col, self.row));
            self.row += 1;

            result
        } else {
            None
        }
    }
}

impl ExactSizeIterator for DownwardDiagonalIter<'_> {
    fn len(&self) -> usize {
        min(self.max_height - self.row, self.col) as usize
    }
}

/// Iterates through the different downward diagonal strips of a board.
/// 
/// Yields a DownwardDiagonalIter to each strip of size >= NUMBER_TO_WIN until the max_height
///  of the board is reached.
pub struct DownwardDiagonalStripIter<'a> {
    board: &'a Board,
    max_height: u8,
    col: u8,
    row: u8,
    full: bool,
}

impl<'a> Iterator for DownwardDiagonalStripIter<'a> {
    type Item = DownwardDiagonalIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop iteration if either we go out of bounds,
        //  or if the iterator isn't full and there aren't enough pieces for a connect four
        if self.col < NUMBER_TO_WIN || (!self.full && self.max_height < NUMBER_TO_WIN) {
            return None;
        }

        let result = Some(DownwardDiagonalIter {
            board: self.board,
            max_height: self.max_height,
            col: self.col,
            row: self.row,
        });

        if self.row > 0 {
            self.row -= 1;
        } else {
            self.col -= 1;
        }

        result
    }
}

impl Board {
    /// Returns an iterator that yields an iterator to each horizontal strip of a board.
    /// 
    /// Exits early at the max_height of the board.
    pub fn horizontal_strip_iter(&self) -> HorizontalStripIter {
        HorizontalStripIter {
            board: self,
            max_height: self.get_max_height(),
            row: 0,
        }
    }

    /// Returns an iterator that yields an iterator to each vertical strip of a board.
    /// 
    /// Each VerticalIter exits early at the max_height of the board.
    /// 
    /// `full` determines if iterators are created for strips of size < NUMBER_TO_WIN.
    /// `full` also determines how early the iterator will halt.
    pub fn vertical_strip_iter(&self, full: bool) -> VerticalStripIter {
        VerticalStripIter {
            board: self,
            col: 0,
            full,
        }
    }

    /// Returns an iterator that yields an iterator to each upward diagonal strip of a board.
    /// 
    /// Each UpwardDiagonalIter exits early at the max_height of the board and doesn't include
    ///  strips less than size NUMBER_TO_WIN.
    /// 
    /// `full` determines if iterators are created for strips of size < NUMBER_TO_WIN.
    /// `full` also determines how early the iterator will halt.
    pub fn upward_diagonal_strip_iter(&self, full: bool) -> UpwardDiagonalStripIter {
        // Our max_height changes based on if we want full iterators that iterate
        //  until only empty rows are being reached, or if we want non full iterators
        //  that iterate until any empty rows are reached
        // We don't need to care about the case with an empty board
        let max_height = if full {
            min(self.get_max_height() + NUMBER_TO_WIN - 1, BOARD_HEIGHT)
        } else {
            self.get_max_height()
        };

        // The row that we start iterating through strips at
        let starting_row = max((max_height as i8) - (NUMBER_TO_WIN as i8), 0i8) as u8;

        UpwardDiagonalStripIter {
            board: self,
            max_height,
            col: 0,
            row: starting_row,
            full,
        }
    }

    /// Returns an iterator that yields an iterator to each downward diagonal strip of a board.
    /// 
    /// Each DownwardDiagonalIter exits early at the max_height of the board and doesn't include
    ///  strips less than size NUMBER_TO_WIN.
    /// 
    /// `full` determines if iterators are created for strips of size < NUMBER_TO_WIN.
    /// `full` also determines how early the iterator will halt.
    pub fn downward_diagonal_strip_iter(&self, full: bool) -> DownwardDiagonalStripIter {
        // Our max_height changes based on if we want full iterators that iterate
        //  until only empty rows are being reached, or if we want non full iterators
        //  that iterate until any empty rows are reached
        // We don't need to care about the case with an empty board
        let max_height = if full {
            min(self.get_max_height() + NUMBER_TO_WIN - 1, BOARD_HEIGHT)
        } else {
            self.get_max_height()
        };

        // The row that we start iterating through strips at
        let starting_row = max((max_height as i8) - (NUMBER_TO_WIN as i8), 0i8) as u8;

        DownwardDiagonalStripIter {
            board: self,
            max_height,
            col: BOARD_WIDTH,
            row: starting_row,
            full,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, OutOfBounds};

    fn piece_to_num(piece: Result<bool, OutOfBounds>) -> u8 {
        match piece {
            Ok(value) => value as u8 + 1,
            Err(_) => 0,
        }
    }

    fn super_collect<T, U>(iter: T) -> Vec<Vec<u8>>
    where
        T: Iterator<Item = U>,
        U: Iterator<Item = Result<bool, OutOfBounds>>,
    {
        iter.map(|iter| iter.map(|p| piece_to_num(p)).collect())
            .collect()
    }

    #[test]
    fn horizontal_strip_iter() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ]);

        let combined_strips = super_collect(board.horizontal_strip_iter());

        assert_eq!(combined_strips, Vec::<Vec::<u8>>::new());

        let mut board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.horizontal_strip_iter());

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 1, 2, 0, 1, 2, 2],
                vec![1, 2, 2, 0, 0, 1, 1],
                vec![1, 2, 2, 0, 0, 0, 0],
                vec![1, 0, 0, 0, 0, 0, 0],
                vec![1, 0, 0, 0, 0, 0, 0],
            ]
        );

        board.drop_piece(0, true).unwrap();

        let combined_strips = super_collect(board.horizontal_strip_iter());

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 1, 2, 0, 1, 2, 2],
                vec![1, 2, 2, 0, 0, 1, 1],
                vec![1, 2, 2, 0, 0, 0, 0],
                vec![1, 0, 0, 0, 0, 0, 0],
                vec![1, 0, 0, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn vertical_strip_iter() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ]);

        let combined_strips = super_collect(board.vertical_strip_iter(false));

        assert_eq!(combined_strips, Vec::<Vec::<u8>>::new());

        let mut board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.vertical_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![vec![1, 1, 1, 1, 1, 2], vec![2, 2, 2, 1],]
        );

        board.drop_piece(6, true).unwrap();

        let combined_strips = super_collect(board.vertical_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![vec![1, 1, 1, 1, 1, 2], vec![2, 2, 2, 1], vec![2, 1, 2, 2],]
        );
    }

    #[test]
    fn upward_diagonal_strip_iter() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![1, 2, 1, 0, 0],
                vec![1, 2, 2, 0, 0, 0],
                vec![1, 2, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 2, 1, 0],
                vec![1, 2, 2, 0, 0],
                vec![1, 2, 0, 0, 0],
                vec![2, 0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(false));

        assert_eq!(combined_strips, Vec::<Vec::<u8>>::new());
    }

    #[test]
    fn downward_diagonal_strip_iter() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![
                vec![2, 0, 0, 0],
                vec![1, 0, 0, 0, 0],
                vec![2, 1, 0, 0, 0, 0],
                vec![2, 0, 0, 1, 0, 2],
                vec![1, 0, 2, 0, 1],
                vec![0, 2, 2, 1],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(false));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![2, 1, 0, 0, 0],
                vec![2, 0, 0, 1, 0],
                vec![1, 0, 2, 0, 1],
                vec![0, 2, 2, 1],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(false));

        assert_eq!(combined_strips, Vec::<Vec::<u8>>::new());
    }

    #[test]
    fn vertical_strip_iter_full() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ]);

        let combined_strips = super_collect(board.vertical_strip_iter(true));

        assert_eq!(combined_strips, Vec::<Vec::<u8>>::new());

        let mut board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.vertical_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 1, 1, 1, 1, 2],
                vec![1, 2, 2, 0, 0, 0],
                vec![2, 2, 2, 1, 0, 0],
                vec![1, 0, 0, 0],
                vec![2, 1, 0, 0, 0],
                vec![2, 1, 2, 0, 0, 0],
            ]
        );

        board.drop_piece(6, true).unwrap();

        let combined_strips = super_collect(board.vertical_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 1, 1, 1, 1, 2],
                vec![1, 2, 2, 0, 0, 0],
                vec![2, 2, 2, 1, 0, 0],
                vec![1, 0, 0, 0],
                vec![2, 1, 0, 0, 0],
                vec![2, 1, 2, 2, 0, 0],
            ]
        );
    }

    #[test]
    fn upward_diagonal_strip_iter_full() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![1, 2, 1, 0, 0],
                vec![1, 2, 2, 0, 0, 0],
                vec![1, 2, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![1, 2, 1, 0, 0],
                vec![1, 2, 2, 0, 0, 0],
                vec![1, 2, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![1, 2, 0, 0, 0],
                vec![1, 2, 2, 0, 0, 0],
                vec![1, 2, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.upward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![1, 0, 0, 0],
                vec![1, 0, 0, 0],
                vec![2, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn downward_diagonal_strip_iter_full() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![2, 0, 0, 0],
                vec![1, 0, 0, 0, 0],
                vec![2, 1, 0, 0, 0, 0],
                vec![2, 0, 0, 1, 0, 2],
                vec![1, 0, 2, 0, 1],
                vec![0, 2, 2, 1],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![2, 0, 0, 0],
                vec![1, 0, 0, 0, 0],
                vec![2, 1, 0, 0, 0, 0],
                vec![2, 0, 0, 1, 0, 0],
                vec![1, 0, 2, 0, 1],
                vec![0, 2, 2, 1],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![2, 0, 0, 0],
                vec![1, 0, 0, 0, 0],
                vec![2, 1, 0, 0, 0, 0],
                vec![2, 0, 0, 0, 0, 0],
                vec![1, 0, 2, 0, 0],
                vec![0, 2, 2, 0],
            ]
        );

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let combined_strips = super_collect(board.downward_diagonal_strip_iter(true));

        assert_eq!(
            combined_strips,
            vec![
                vec![2, 0, 0, 0],
                vec![2, 0, 0, 0],
                vec![1, 0, 0, 0],
                vec![0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn iters_len() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let horizontal_lens: Vec<usize> = board
            .horizontal_strip_iter()
            .map(|iter| iter.len())
            .collect();
        assert_eq!(horizontal_lens, vec![7, 7, 7, 7, 7, 7]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![6, 4]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 6, 6, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 6, 6, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let horizontal_lens: Vec<usize> = board
            .horizontal_strip_iter()
            .map(|iter| iter.len())
            .collect();
        assert_eq!(horizontal_lens, vec![7, 7, 7, 7, 7]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![5, 4]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 5, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 5, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let horizontal_lens: Vec<usize> = board
            .horizontal_strip_iter()
            .map(|iter| iter.len())
            .collect();
        assert_eq!(horizontal_lens, vec![7, 7, 7, 7]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![4, 4]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 4, 4, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 4, 4, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let horizontal_lens: Vec<usize> = board
            .horizontal_strip_iter()
            .map(|iter| iter.len())
            .collect();
        assert_eq!(horizontal_lens, vec![7, 7, 7]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, Vec::<usize>::new());

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, Vec::<usize>::new());

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, Vec::<usize>::new());

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let horizontal_lens: Vec<usize> = board
            .horizontal_strip_iter()
            .map(|iter| iter.len())
            .collect();
        assert_eq!(horizontal_lens, vec![7]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, Vec::<usize>::new());

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, Vec::<usize>::new());

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(false)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, Vec::<usize>::new());
    }

    #[test]
    fn iters_len_full() {
        let board = Board::from_arrays([
            [2, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![6, 6, 6, 4, 5, 6]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 6, 6, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 6, 6, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![6, 6, 6, 4, 5, 6]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 6, 6, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 6, 6, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![6, 6, 6, 4, 5, 6]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 6, 6, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 6, 6, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 2, 0, 0, 0, 2],
            [1, 2, 2, 0, 0, 1, 1],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![6, 6, 6, 4, 5, 6]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 5, 6, 6, 5, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 5, 6, 6, 5, 4]);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 2, 0, 1, 2, 2],
        ]);

        let vertical_lens: Vec<usize> = board
            .vertical_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(vertical_lens, vec![4, 4, 4, 4, 4, 4]);

        let upward_lens: Vec<usize> = board
            .upward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(upward_lens, vec![4, 4, 4, 4]);

        let downward_lens: Vec<usize> = board
            .downward_diagonal_strip_iter(true)
            .map(|iter| iter.len())
            .collect();
        assert_eq!(downward_lens, vec![4, 4, 4, 4]);
    }
}
