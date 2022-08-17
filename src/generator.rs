use crate::board_state::BoardState;

/// Iterator used to generate a single layer of a BoardState decision tree.
///
/// Iteration will stop when the new generation is fully completed.
/// The new layer is not guaranteed to be generated in any particular order.
struct LayerGenerator<'a> {
    previous_generation: Vec<&'a mut BoardState>,
    new_generation: Vec<&'a mut BoardState>,
}

impl<'a> Iterator for LayerGenerator<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(board_state) = self.previous_generation.pop() {
            self.new_generation
                .extend(board_state.generate_children().iter_mut());

            Some(())
        } else {
            None
        }
    }
}

/// Iterator used to generate a BoardState decision tree.
///
/// Iteration will stop only if the full decision tree has been explored.
pub struct Generator<'a> {
    current_board: &'a mut BoardState,
    current_layer_generator: Option<LayerGenerator<'a>>,
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, board_state::BoardState, consts::BOARD_WIDTH};

    use super::LayerGenerator;

    #[test]
    fn layer_generator() {
        let mut board_state = BoardState::default();
        let first_generation = vec![&mut board_state];

        let mut layer_generator = LayerGenerator {
            previous_generation: first_generation,
            new_generation: Vec::new(),
        };

        assert_eq!(layer_generator.next(), Some(()));
        assert_eq!(layer_generator.new_generation.len(), BOARD_WIDTH as usize);
        assert_eq!(layer_generator.next(), None);

        let second_generation = layer_generator.new_generation;

        let mut layer_generator = LayerGenerator {
            previous_generation: second_generation,
            new_generation: Vec::new(),
        };

        for i in 0..BOARD_WIDTH {
            assert_eq!(layer_generator.next(), Some(()));
            assert_eq!(
                layer_generator.new_generation.len(),
                (BOARD_WIDTH * (i + 1)) as usize
            );
        }
        assert_eq!(layer_generator.next(), None);

        let last_board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
        ]);

        assert_eq!(board_state.children[6].children[6].board, last_board);
    }
}
