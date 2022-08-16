use crate::{
    board::{Board, OutOfBounds},
    consts::NUMBER_TO_WIN,
};

const OOB: Result<bool, OutOfBounds> = Err(OutOfBounds);

/// Used to define how much better an X in a row is to a X-1 in a row
const SCALING_HEURISTIC: isize = 10;

/// A circular buffer used to iterate through all sets of four pieces
///  in a given iterator.
/// It automatically tracks how many of each piece type are within the
///  buffer, and returns the piece_counts with each iteration of itself.
struct CircleBuffer<T>
where
    T: Iterator<Item = Result<bool, OutOfBounds>>,
{
    buffer: [Result<bool, OutOfBounds>; NUMBER_TO_WIN as usize],
    iter: T,
    index: usize,
    piece_counts: [u32; 2],
}

impl<T> CircleBuffer<T>
where
    T: Iterator<Item = Result<bool, OutOfBounds>>,
{
    /// Creates a CircleBuffer using a board iterator
    fn new(mut iter: T) -> CircleBuffer<T> {
        let mut buffer = [OOB; NUMBER_TO_WIN as usize];
        let mut piece_counts = [0; 2];

        // Initializing the buffer
        // We leave off the last entry, which will be filled when we call next for the first time
        for i in 0..NUMBER_TO_WIN as usize {
            // If the iterator is less than
            let piece = iter.next().unwrap_or(OOB);
            if let Ok(value) = piece {
                piece_counts[value as usize] += 1;
            }
            buffer[i] = piece;
        }

        CircleBuffer {
            buffer,
            iter,
            index: 0,
            piece_counts,
        }
    }
}

impl<T> Iterator for CircleBuffer<T>
where
    T: Iterator<Item = Result<bool, OutOfBounds>>,
{
    type Item = [u32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        // We can use the index being out of bounds as our cue to stop iteration
        // This will happen when our internal iterator starts returning None
        // This is done to ensure that CircleBuffer always iterates at least once
        // We want to always be able to retrieve the first set of piece counts
        //  even if the internal iterator has size < NUMBER_TO_WIN
        if self.index >= NUMBER_TO_WIN as usize {
            return None;
        }

        // TODO: Figure out some way to avoid needing to clone these values each time
        let to_return = Some(self.piece_counts.clone());
        // If the iterator is still returning values, we can use it to update our internal
        //  buffer
        if let Some(piece) = self.iter.next() {
            // Before we update the internal buffer, we should update our piece_counts to
            //  reflect the new piece coming in and the old piece leaving
            if let Ok(value) = piece {
                self.piece_counts[value as usize] += 1;
            }
            if let Ok(value) = self.buffer[self.index] {
                self.piece_counts[value as usize] -= 1;
            }

            // Now we can officially overwrite the old piece and increment the index
            self.buffer[self.index] = piece;
            self.index = (self.index + 1) % NUMBER_TO_WIN as usize;
        } else {
            // If the iterator is no longer returning values, we can signal that we should
            //  stop further iteration
            self.index = NUMBER_TO_WIN as usize;
        }

        to_return
    }
}

fn score_circle_buffer<T>(circle_buffer: CircleBuffer<T>) -> isize
where
    T: Iterator<Item = Result<bool, OutOfBounds>>,
{
    let mut score = 0;
    for [false_pieces, true_pieces] in circle_buffer {
        if false_pieces > 0 && true_pieces == 0 {
            // If false has pieces that aren't blocked from a connect four via true
            score -= SCALING_HEURISTIC.pow(false_pieces - 1);
        } else if true_pieces > 0 && false_pieces == 0 {
            // If true has pieces that aren't blocked from a connect four via false
            score += SCALING_HEURISTIC.pow(true_pieces - 1);
        }
    }

    score
}

/// This heuristic judges a board state by trying to determine who is closer
///  to a connect four.
/// This is judged by finding how many X in a rows there are, with bigger Xs
///  leading to a higher score.
fn score_by_closeness_to_win(board: Board) -> isize {
    let mut score = 0;

    // First we can calculate scores along the horizontal strips
    for iter in board.horizontal_strip_iter() {
        score += score_circle_buffer(CircleBuffer::new(iter));
    }

    // Next we can calculate scores along the vertical strips
    for iter in board.vertical_strip_iter(true) {
        score += score_circle_buffer(CircleBuffer::new(iter));
    }

    // Next we can calculate scores along the upward diagonal strips
    for iter in board.upward_diagonal_strip_iter(true) {
        score += score_circle_buffer(CircleBuffer::new(iter));
    }

    // Next we can calculate scores along the downward diagonal strips
    for iter in board.downward_diagonal_strip_iter(true) {
        score += score_circle_buffer(CircleBuffer::new(iter));
    }

    score
}

/// Heuristically determines how good a given board state is
/// Positive values are favorable to true, negative to false
pub fn how_good_is(board: Board) -> isize {
    // TODO: Find a heuristic that doesn't multi count 2 1 1 1 0 0 0 for 1s
    score_by_closeness_to_win(board)
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        heuristics::{score_by_closeness_to_win, score_circle_buffer},
    };

    use super::{CircleBuffer, OOB};

    #[test]
    fn circle_buffer() {
        let iter = [].into_iter();
        let mut cb = CircleBuffer::new(iter);

        assert_eq!(cb.next(), Some([0, 0]));
        assert_eq!(cb.next(), None);

        let iter = [Ok(true), OOB, Ok(false)].into_iter();
        let mut cb = CircleBuffer::new(iter);

        assert_eq!(cb.next(), Some([1, 1]));
        assert_eq!(cb.next(), None);

        let iter = [Ok(true), Ok(true), OOB, OOB].into_iter();
        let mut cb = CircleBuffer::new(iter);

        assert_eq!(cb.next(), Some([0, 2]));
        assert_eq!(cb.next(), None);

        let iter = [
            OOB,
            Ok(true),
            OOB,
            Ok(false),
            Ok(false),
            OOB,
            Ok(false),
            Ok(false),
            Ok(true),
            OOB,
            OOB,
            OOB,
            OOB,
        ]
        .into_iter();
        let mut cb = CircleBuffer::new(iter);

        assert_eq!(cb.next(), Some([1, 1]));
        assert_eq!(cb.next(), Some([2, 1]));
        assert_eq!(cb.next(), Some([2, 0]));
        assert_eq!(cb.next(), Some([3, 0]));
        assert_eq!(cb.next(), Some([3, 0]));
        assert_eq!(cb.next(), Some([2, 1]));
        assert_eq!(cb.next(), Some([2, 1]));
        assert_eq!(cb.next(), Some([1, 1]));
        assert_eq!(cb.next(), Some([0, 1]));
        assert_eq!(cb.next(), Some([0, 0]));
        assert_eq!(cb.next(), None);
    }

    #[test]
    fn scoring_circle_buffer() {
        let iter = [].into_iter();
        let cb = CircleBuffer::new(iter);

        assert_eq!(score_circle_buffer(cb), 0);

        let iter = [Ok(true), OOB, Ok(false)].into_iter();
        let cb = CircleBuffer::new(iter);

        assert_eq!(score_circle_buffer(cb), 0);

        let iter = [Ok(true), Ok(true), OOB, OOB].into_iter();
        let cb = CircleBuffer::new(iter);

        assert_eq!(score_circle_buffer(cb), 10);

        let iter = [
            OOB,
            Ok(true),
            OOB,
            Ok(false),
            Ok(false),
            OOB,
            Ok(false),
            Ok(false),
            Ok(true),
            OOB,
            OOB,
            OOB,
            OOB,
        ]
        .into_iter();
        let cb = CircleBuffer::new(iter);

        assert_eq!(score_circle_buffer(cb), -209);
    }

    #[test]
    fn scoring_board() {
        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
        ]);

        assert_eq!(score_by_closeness_to_win(board), 132);

        let board = Board::from_arrays([
            [2, 2, 2, 1, 2, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        assert_eq!(score_by_closeness_to_win(board), 0);
    }
}
