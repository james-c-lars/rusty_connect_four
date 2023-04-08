use std::{cell::RefCell, rc::Rc};

use crate::game_engine::{
    board_state::BoardState, transposition::TranspositionTable, win_check::GameOver,
};

/// Iterator used to generate a BoardState decision tree. Each iteration will
/// return how many new board states were generated.
///
/// Iteration will stop when the decision tree is complete.
#[derive(Debug)]
pub struct LayerGenerator {
    // TODO: Could these be HashSets? Prevent duplicate calls to generate_children
    generation_1: Vec<Rc<RefCell<BoardState>>>,
    generation_2: Vec<Rc<RefCell<BoardState>>>,
    generation_1_is_new: bool,
    table: TranspositionTable,
}

impl LayerGenerator {
    /// Gets the newest of the two stored generations.
    ///
    /// The new generation will be the one at the bottom of the decision tree.
    fn get_new_generation(&mut self) -> &mut Vec<Rc<RefCell<BoardState>>> {
        if self.generation_1_is_new {
            &mut self.generation_1
        } else {
            &mut self.generation_2
        }
    }

    /// Gets the previous of the two stored generations.
    ///
    /// The previous generation will be the next-to-last layer of the decision tree.
    fn get_previous_generation(&mut self) -> &mut Vec<Rc<RefCell<BoardState>>> {
        if self.generation_1_is_new {
            &mut self.generation_2
        } else {
            &mut self.generation_1
        }
    }

    /// Returns a reference to the TranspositionTable used to generate BoardStates.
    pub fn table_ref(&self) -> &TranspositionTable {
        &self.table
    }

    /// Constructs a new LayerGenerator for a given BoardState.
    pub fn new(board: Rc<RefCell<BoardState>>, table: TranspositionTable) -> LayerGenerator {
        let (previous_generation, new_generation) = LayerGenerator::get_bottom_two_layers(board);

        LayerGenerator {
            generation_1: previous_generation,
            generation_2: new_generation,
            generation_1_is_new: false,
            table,
        }
    }

    /// Restarts the LayerGeneration process, using a new root for the decision tree.
    pub fn restart(&mut self, board: Rc<RefCell<BoardState>>) {
        let (previous_generation, new_generation) = LayerGenerator::get_bottom_two_layers(board);

        self.generation_1 = previous_generation;
        self.generation_2 = new_generation;
        self.generation_1_is_new = false;
    }

    /// Finds the BoardStates at the bottom of the decision tree and returns
    ///  vectors to them.
    ///
    /// Helper function for use in creating a new LayerGenerator.
    ///
    /// Returns a tuple of (previous_generation, new_generation).
    fn get_bottom_two_layers(
        board: Rc<RefCell<BoardState>>,
    ) -> (Vec<Rc<RefCell<BoardState>>>, Vec<Rc<RefCell<BoardState>>>) {
        // bottom_layers will contain all games that still need children generated
        // This should only consist of one or two distinct generations
        // We can separate the generations via whose turn it is
        let mut bottom_layers: [Vec<Rc<RefCell<BoardState>>>; 2] = [Vec::new(), Vec::new()];

        // to_explore is a stack of all the nodes left to explore as we search for the
        //  bottom
        let mut to_explore: Vec<Rc<RefCell<BoardState>>> = vec![board];

        // While our exploration stack still has nodes
        while let Some(curr_state) = to_explore.pop() {
            // If the node already has had its children generated
            if curr_state.borrow().children.len() > 0 {
                // Add the children to the stack to be explored
                to_explore.extend(curr_state.borrow().children.iter().map(|c| c.state.clone()));
            } else if curr_state.borrow().is_game_over() == GameOver::NoWin {
                // Otherwise, if the node isn't a dead end (already won)
                // Add the node to our list of nodes that need children generated
                bottom_layers[curr_state.borrow().get_turn() as usize].push(curr_state.clone());
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
            let false_depth = false_layer[0].borrow().get_depth();
            let true_depth = true_layer[0].borrow().get_depth();

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

impl Iterator for LayerGenerator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // If there are still BoardStates in the previous generation, we can
        //  continue computing from there
        if let Some(board_state) = self.get_previous_generation().pop() {
            let generated_children = board_state.borrow_mut().generate_children(&mut self.table);
            let num_generated = generated_children.len();

            self.get_new_generation().extend(generated_children);

            Some(num_generated)
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        consts::BOARD_WIDTH,
        game_engine::{
            board::Board, board_state::BoardState, layer_generator::LayerGenerator,
            transposition::TranspositionTable,
        },
    };

    #[test]
    fn layer_generator() {
        let board_state = Rc::new(RefCell::new(BoardState::default()));
        let first_generation = vec![board_state.clone()];

        let mut layer_generator = LayerGenerator {
            generation_1: first_generation,
            generation_2: Vec::new(),
            generation_1_is_new: false,
            table: TranspositionTable::default(),
        };

        assert!(layer_generator.next().is_some());
        assert_eq!(
            layer_generator.get_new_generation().len(),
            BOARD_WIDTH as usize
        );
        assert_eq!(layer_generator.get_previous_generation().len(), 0);

        for i in 0..BOARD_WIDTH {
            assert!(layer_generator.next().is_some());
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

        assert_eq!(
            // Here the 5th child is really column 7, due to the alpha-beta move generation optimization
            board_state.borrow().children[5].state.borrow().children[5]
                .state
                .borrow()
                .board,
            last_board
        );

        let board_state = Rc::new(RefCell::new(BoardState::default()));
        let first_generation = vec![board_state];
        let mut layer_generator = LayerGenerator {
            generation_1: first_generation,
            generation_2: Vec::new(),
            generation_1_is_new: false,
            table: TranspositionTable::default(),
        };

        for _ in 0..10_000 {
            layer_generator.next();
        }

        assert!(layer_generator.next().is_some());
    }

    #[test]
    fn get_bottom_two_layers() {
        let board_state = Rc::new(RefCell::new(BoardState::default()));
        let (previous, new) = LayerGenerator::get_bottom_two_layers(board_state.clone());

        assert_eq!(previous.len(), 1);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: TranspositionTable::default(),
        };
        layer_generator.next();

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            BOARD_WIDTH as usize
        );

        let (previous, new) = LayerGenerator::get_bottom_two_layers(board_state.clone());

        assert_eq!(previous.len(), BOARD_WIDTH as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: TranspositionTable::default(),
        };
        for _ in 0..7 {
            layer_generator.next();
        }

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (BOARD_WIDTH * BOARD_WIDTH) as usize
        );

        let (previous, new) = LayerGenerator::get_bottom_two_layers(board_state.clone());

        assert_eq!(previous.len(), (BOARD_WIDTH * BOARD_WIDTH) as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: TranspositionTable::default(),
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

        let (previous, new) = LayerGenerator::get_bottom_two_layers(board_state.clone());

        assert_eq!(
            previous.len(),
            (BOARD_WIDTH * BOARD_WIDTH - SOME_NUMBER) as usize
        );
        assert_eq!(new.len(), (SOME_NUMBER * BOARD_WIDTH) as usize);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: TranspositionTable::default(),
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

        let (previous, new) = LayerGenerator::get_bottom_two_layers(board_state.clone());

        assert_eq!(previous.len(), (BOARD_WIDTH * BOARD_WIDTH - 8) as usize);
        assert_eq!(new.len(), (2 * SOME_NUMBER * BOARD_WIDTH) as usize);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: TranspositionTable::default(),
        };
        for _ in 0..100_000 {
            layer_generator.next();
        }

        let previous_depth = layer_generator.get_previous_generation()[0]
            .borrow()
            .get_depth();
        for previous_state in layer_generator.get_previous_generation().iter() {
            assert_eq!(previous_state.borrow().get_depth(), previous_depth);
        }

        let new_depth = layer_generator.get_new_generation()[0].borrow().get_depth();
        for new_state in layer_generator.get_new_generation().iter() {
            assert_eq!(new_state.borrow().get_depth(), new_depth);
        }

        assert_eq!(previous_depth + 1, new_depth);
    }

    #[test]
    fn try_generate_counts_correctly() {
        let board = Board::from_arrays([
            [2, 2, 2, 1, 0, 2, 2],
            [1, 1, 1, 2, 1, 1, 1],
            [2, 2, 1, 1, 1, 2, 1],
            [1, 1, 2, 2, 1, 1, 2],
            [2, 2, 1, 1, 2, 2, 1],
            [2, 2, 1, 1, 2, 1, 2],
        ]);

        let root = Rc::new(RefCell::new(BoardState::new(board, true)));

        let mut generator = LayerGenerator::new(root, TranspositionTable::default());

        assert_eq!(generator.next(), Some(1));

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
        ]);

        let root = Rc::new(RefCell::new(BoardState::new(board, true)));

        let mut generator = LayerGenerator::new(root, TranspositionTable::default());

        for _ in 0..(7 + 49 + 343) {
            assert_eq!(generator.next(), Some(6));
        }
    }
}
