use std::{
    cell::RefCell,
    cmp::max,
    mem::size_of,
    rc::{Rc, Weak},
};

use crate::game_engine::{
    board_state::{BoardState, ChildState},
    layer_generator::LayerGenerator,
};

/// Contains different numerical details about the size of a
/// decision tree.
#[derive(Default, Debug, Clone, Copy)]
pub struct TreeSize {
    pub depth: usize,
    pub size: usize,
    pub memory: usize,
}

/// Calculates numerical details about a decision tree.
pub fn calculate_size(root: Rc<RefCell<BoardState>>, generator: &LayerGenerator) -> TreeSize {
    let mut depth = 0;
    let mut size = 0;
    let mut memory = 0;

    for (_, weak_ref) in generator.table_ref().iter() {
        // Size of the reference in the table
        memory += size_of::<u64>(); // key
        memory += size_of::<Weak<RefCell<BoardState>>>(); // value

        // Size of the reference as a child
        if weak_ref.strong_count() > 0 {
            memory += size_of::<BoardState>();
            memory += size_of::<ChildState>() * weak_ref.strong_count();

            size += weak_ref.strong_count();

            let current_depth = weak_ref.upgrade().unwrap().borrow().get_depth();
            depth = max(current_depth, depth);
        }
    }

    size -= generator.buffer_size();

    TreeSize {
        depth: (depth - root.borrow().get_depth() + 1) as usize,
        size,
        memory,
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::game_engine::{
        board::Board, board_state::BoardState, layer_generator::LayerGenerator,
        transposition::TranspositionTable, tree_size::calculate_size,
    };

    #[test]
    fn correct_size() {
        let board_array = [
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2, 1],
            [0, 1, 2, 0, 0, 1, 2],
            [0, 1, 2, 0, 2, 1, 2],
        ];

        let mut table = TranspositionTable::default();
        let (root, _) = table.get_board_state(Board::from_arrays(board_array), false);

        let mut generator = LayerGenerator::new(table);
        for _ in 0..(1 + 6 + 36) {
            generator.next();
        }

        let stats = calculate_size(root.clone(), &generator);

        let (depth, size) = calculate_from_root(root.clone());
        assert_eq!(stats.depth, depth);
        assert!(
            stats.size <= size + 1,
            "calculated: {}, manually: {}",
            stats.size,
            size
        );

        for _ in 0..1000 {
            generator.next();
        }

        let stats = calculate_size(root.clone(), &generator);

        let (depth, size) = calculate_from_root(root.clone());
        assert_eq!(stats.depth, depth);
        assert!(
            stats.size < size,
            "calculated: {}, manually: {}",
            stats.size,
            size
        );
    }

    fn calculate_from_root(root: Rc<RefCell<BoardState>>) -> (usize, usize) {
        let mut current_layer = vec![root];
        let mut next_layer = Vec::new();

        let mut size = 0;
        let mut depth = 0;
        while let Some(current_node) = current_layer.pop() {
            size += 1;
            next_layer.extend(
                current_node
                    .borrow()
                    .children
                    .iter()
                    .map(|n| n.state.clone()),
            );

            if current_layer.len() == 0 {
                current_layer = next_layer;
                next_layer = Vec::new();
                depth += 1;
            }
        }

        (depth, size)
    }
}
