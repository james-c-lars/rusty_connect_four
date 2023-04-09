use std::{cell::RefCell, cmp::max, collections::HashMap, rc::{Rc, Weak}, time::Instant};

use crate::{
    game_engine::{
        board_state::BoardState, transposition::TranspositionTable, win_check::GameOver,
    },
    log::{log_message, LogType},
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
    table: TranspositionTable<Weak<RefCell<BoardState>>>,
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

    /// Gets the sizes of the internal buffers.
    pub fn buffer_size(&self) -> usize {
        self.generation_1.len() + self.generation_2.len()
    }

    /// Returns a reference to the TranspositionTable used to generate BoardStates.
    pub fn table_ref(&self) -> &TranspositionTable<Weak<RefCell<BoardState>>> {
        &self.table
    }

    /// Constructs a new LayerGenerator for a given BoardState.
    pub fn new(table: TranspositionTable<Weak<RefCell<BoardState>>>) -> LayerGenerator {
        assert_ne!(table.len(), 0);

        let (previous_generation, new_generation) = LayerGenerator::get_bottom_two_layers(&table);

        LayerGenerator {
            generation_1: previous_generation,
            generation_2: new_generation,
            generation_1_is_new: false,
            table,
        }
    }

    /// Restarts the LayerGeneration process, rescanning the tranposition table.
    pub fn restart(&mut self) {
        let sub_start = Instant::now();
        // In order to determine which leaves aren't in use we need to remove our own
        //  references to them.
        self.generation_1.clear();
        self.generation_2.clear();
        self.table.clean();
        log_message(
            LogType::Performance,
            format!(
                "Restart Layer Generator [Clean] - {}",
                sub_start.elapsed().as_secs_f32()
            ),
        );

        let sub_start = Instant::now();
        let (previous_generation, new_generation) =
            LayerGenerator::get_bottom_two_layers(&self.table);
        log_message(
            LogType::Performance,
            format!(
                "Restart Layer Generator [Get Bottom Two Layers] - {}",
                sub_start.elapsed().as_secs_f32()
            ),
        );

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
        table: &TranspositionTable<Weak<RefCell<BoardState>>>,
    ) -> (Vec<Rc<RefCell<BoardState>>>, Vec<Rc<RefCell<BoardState>>>) {
        let mut depth_sorted_nodes: HashMap<u8, Vec<Rc<RefCell<BoardState>>>> = HashMap::new();
        let mut max_depth = 0;

        for (_, weak_ref) in table.iter() {
            if let Some(board_state) = weak_ref.upgrade() {
                if board_state.borrow().children.len() > 0
                    || board_state.borrow().is_game_over() != GameOver::NoWin
                {
                    continue;
                }

                let current_depth = board_state.borrow().get_depth();
                max_depth = max(current_depth, max_depth);

                if current_depth == max_depth || current_depth + 1 == max_depth {
                    if let Some(depth_array) = depth_sorted_nodes.get_mut(&current_depth) {
                        depth_array.push(board_state);
                    } else {
                        depth_sorted_nodes.insert(current_depth, vec![board_state]);
                    }
                }
            }
        }

        let mut previous_generation;
        let mut new_generation;
        if max_depth > 0 {
            previous_generation = depth_sorted_nodes
                .remove(&(max_depth - 1))
                .unwrap_or(Vec::new());
            new_generation = depth_sorted_nodes.remove(&max_depth).unwrap();

            if previous_generation.len() == 0 {
                previous_generation = new_generation;
                new_generation = Vec::new();
            }
        } else {
            // Max Depth = 0 when starting a new game or at the end of a game
            previous_generation = depth_sorted_nodes.remove(&max_depth).unwrap_or(Vec::new());
            new_generation = Vec::new();
        }

        (previous_generation, new_generation)
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

        for _ in 0..BOARD_WIDTH {
            assert!(layer_generator.next().is_some());
        }
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (BOARD_WIDTH * 4) as usize
        );
        assert_eq!(layer_generator.get_previous_generation().len(), 0);

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
        let mut table = TranspositionTable::default();
        let (root, _) = table.get_board_state(Board::default(), false);

        let (previous, new) = LayerGenerator::get_bottom_two_layers(&table);

        assert_eq!(previous.len(), 1);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table,
        };
        layer_generator.next();

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            BOARD_WIDTH as usize
        );

        let (previous, new) = LayerGenerator::get_bottom_two_layers(&layer_generator.table);

        assert_eq!(previous.len(), (BOARD_WIDTH / 2 + 1) as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: layer_generator.table,
        };
        for _ in 0..(BOARD_WIDTH / 2 + 1) {
            layer_generator.next();
        }

        assert_eq!(layer_generator.get_previous_generation().len(), 0);
        assert_eq!(
            layer_generator.get_new_generation().len(),
            (BOARD_WIDTH * 4) as usize
        );

        let (previous, new) = LayerGenerator::get_bottom_two_layers(&layer_generator.table);

        assert_eq!(previous.len(), (BOARD_WIDTH * BOARD_WIDTH / 2 + 1) as usize);
        assert_eq!(new.len(), 0);

        let mut layer_generator = LayerGenerator {
            generation_1: previous,
            generation_2: new,
            generation_1_is_new: false,
            table: layer_generator.table,
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

        drop(root);
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

        let mut table = TranspositionTable::default();
        let (root, _) = table.get_board_state(board, true);

        let mut generator = LayerGenerator::new(table);

        assert_eq!(generator.next(), Some(1));

        drop(root);

        let board = Board::from_arrays([
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
        ]);

        let mut table = TranspositionTable::default();
        let (root, _) = table.get_board_state(board, true);

        let mut generator = LayerGenerator::new(table);

        for _ in 0..(7 + 49 + 343) {
            let num_generated = generator.next().unwrap();
            assert!(num_generated == 6 || num_generated == 0);
        }

        drop(root);
    }
}
