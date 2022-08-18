use crate::board_state::BoardState;

/// Iterator used to generate a BoardState decision tree.
///
/// Iteration will stop when the decision tree is complete.
pub struct LayerGenerator<'a> {
    generation_1: Vec<&'a mut BoardState>,
    generation_2: Vec<&'a mut BoardState>,
    generation_1_is_new: bool,
}

impl<'a> LayerGenerator<'a> {
    /// Gets the newest of the two stored generations.
    ///
    /// The new generation will be the one at the bottom of the decision tree.
    fn get_new_generation(&mut self) -> &mut Vec<&'a mut BoardState> {
        if self.generation_1_is_new {
            &mut self.generation_1
        } else {
            &mut self.generation_2
        }
    }

    /// Gets the previous of the two stored generations.
    ///
    /// The previous generation will be the next-to-last layer of the decision tree.
    fn get_previous_generation(&mut self) -> &mut Vec<&'a mut BoardState> {
        if self.generation_1_is_new {
            &mut self.generation_2
        } else {
            &mut self.generation_1
        }
    }

    /// Constructs a new LayerGenerator for a given BoardState
    pub fn new(board: &mut BoardState) -> LayerGenerator {
        let (previous_generation, new_generation) = board.get_bottom_two_layers();

        LayerGenerator {
            generation_1: previous_generation,
            generation_2: new_generation,
            generation_1_is_new: false,
        }
    }
}

impl<'a> Iterator for LayerGenerator<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        // If there are still BoardStates in the previous generation, we can
        //  continue computing from there
        if let Some(board_state) = self.get_previous_generation().pop() {
            self.get_new_generation()
                .extend(board_state.generate_children().iter_mut());

            Some(())
        } else if self.get_new_generation().len() > 0 {
            // Otherwise, as long as there are a new set of BoardStates for
            //  us to compute children for, we can continue computing for
            //  the new set of BoardStates

            // To do this, we flip our generation vectors
            // The empty previous_generation vector becomes the new new_generation
            //  vector and the full new_generation vector becomes the new
            //  previous_generation vector
            self.generation_1_is_new = !self.generation_1_is_new;

            self.next()
        } else {
            // If there are no more nodes needing computation, the decision tree is
            //  complete and we can stop iterating
            None
        }
    }
}

impl BoardState {
    /// Finds the BoardStates at the bottom of the decision tree and returns
    ///  vectors to them.
    ///
    /// Helper function for use in creating a new LayerGenerator.
    ///
    /// Returns a tuple of (previous_generation, new_generation).
    fn get_bottom_two_layers(&mut self) -> (Vec<&mut BoardState>, Vec<&mut BoardState>) {
        // bottom_layers will contain all games that still need children generated
        // This should only consist of one or two distinct generations
        // We can separate the generations via whose turn it is
        let mut bottom_layers: [Vec<&mut BoardState>; 2] = [Vec::new(), Vec::new()];

        // to_explore is a stack of all the nodes left to explore as we search for the
        //  bottom
        let mut to_explore: Vec<&mut BoardState> = vec![self];

        // While our exploration stack still has nodes
        while let Some(curr_state) = to_explore.pop() {
            // If the node already has had its children generated
            if curr_state.children.len() > 0 {
                // Add the children to the stack to be explored
                to_explore.extend(curr_state.children.iter_mut());
            } else if curr_state.is_game_over() == None {
                // Otherwise, if the node isn't a dead end (already won)
                // Add the node to our list of nodes that need children generated
                bottom_layers[curr_state.get_turn() as usize].push(curr_state);
            }
        }

        let [false_layer, true_layer] = bottom_layers;

        // Now we want to determine which of our two generations is the newest
        // We can do this by finding the depth of each layer and then comparing them
        // The depth of each node in a generation should be the same, so we'll check
        //  an arbitrary node from each generation to compare

        // First we need to make sure there's even a node in each generation to compare
        if false_layer.len() > 0 && true_layer.len() > 0 {
            // Then we can calculate the depth
            let false_depth = false_layer[0].get_depth();
            let true_depth = true_layer[0].get_depth();

            if false_depth < true_depth {
                (false_layer, true_layer)
            } else {
                (true_layer, false_layer)
            }
        } else {
            // Otherwise just return the layers based on which one actually has nodes
            if false_layer.len() > 0 {
                (false_layer, true_layer)
            } else {
                (true_layer, false_layer)
            }
            // And in the worst case we'll be returning two empty vectors which
            //  is fine
        }
    }
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
            generation_1: first_generation,
            generation_2: Vec::new(),
            generation_1_is_new: false,
        };

        assert_eq!(layer_generator.next(), Some(()));
        assert_eq!(
            layer_generator.get_new_generation().len(),
            BOARD_WIDTH as usize
        );
        assert_eq!(layer_generator.get_previous_generation().len(), 0);

        for i in 0..BOARD_WIDTH {
            assert_eq!(layer_generator.next(), Some(()));
            assert_eq!(
                layer_generator.get_new_generation().len(),
                (BOARD_WIDTH * (i + 1)) as usize
            );
            assert_eq!(
                layer_generator.get_previous_generation().len(),
                (BOARD_WIDTH - i - 1) as usize
            );
        }

        let last_board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
        ]);

        assert_eq!(board_state.children[6].children[6].board, last_board);

        let mut board_state = BoardState::default();
        let first_generation = vec![&mut board_state];

        let mut layer_generator = LayerGenerator {
            generation_1: first_generation,
            generation_2: Vec::new(),
            generation_1_is_new: false,
        };

        for _ in 0..10_000 {
            layer_generator.next();
        }

        assert_eq!(layer_generator.next(), Some(()));
    }

    #[test]
    fn get_bottom_two_layers() {
        let mut board_state = BoardState::default();

        let (previous, new) = board_state.get_bottom_two_layers();

        assert_eq!(previous.len(), 1);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
        };
        layer_generator.next();

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            BOARD_WIDTH as usize
        );

        let (previous, new) = board_state.get_bottom_two_layers();

        assert_eq!(previous.len(), BOARD_WIDTH as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
        };
        for _ in 0..7 {
            layer_generator.next();
        }

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (BOARD_WIDTH * BOARD_WIDTH) as usize
        );

        let (previous, new) = board_state.get_bottom_two_layers();

        assert_eq!(previous.len(), (BOARD_WIDTH * BOARD_WIDTH) as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
        };

        const SOME_NUMBER: u8 = 4;
        assert!(SOME_NUMBER * 2 < BOARD_WIDTH * BOARD_WIDTH);

        for _ in 0..SOME_NUMBER {
            layer_generator.next();
        }

        assert_eq!(
            layer_generator.get_previous_generation().len(),
            (BOARD_WIDTH * BOARD_WIDTH - SOME_NUMBER) as usize
        );
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (SOME_NUMBER * BOARD_WIDTH) as usize
        );

        let (previous, new) = board_state.get_bottom_two_layers();

        assert_eq!(
            previous.len(),
            (BOARD_WIDTH * BOARD_WIDTH - SOME_NUMBER) as usize
        );
        assert_eq!(new.len(), (SOME_NUMBER * BOARD_WIDTH) as usize);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
        };
        for _ in 0..SOME_NUMBER {
            layer_generator.next();
        }

        assert_eq!(
            layer_generator.get_previous_generation().len(),
            (BOARD_WIDTH * BOARD_WIDTH - 2 * SOME_NUMBER) as usize
        );
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (2 * SOME_NUMBER * BOARD_WIDTH) as usize
        );

        let (previous, new) = board_state.get_bottom_two_layers();

        assert_eq!(previous.len(), (BOARD_WIDTH * BOARD_WIDTH - 8) as usize);
        assert_eq!(new.len(), (2 * SOME_NUMBER * BOARD_WIDTH) as usize);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
        };
        for _ in 0..100_000 {
            layer_generator.next();
        }

        let previous_depth = layer_generator.get_previous_generation()[0].get_depth();
        for previous_state in layer_generator.get_previous_generation().iter() {
            assert_eq!(previous_state.get_depth(), previous_depth);
        }

        let new_depth = layer_generator.get_new_generation()[0].get_depth();
        for new_state in layer_generator.get_new_generation().iter() {
            assert_eq!(new_state.get_depth(), new_depth);
        }

        assert_eq!(previous_depth + 1, new_depth);
    }
}
