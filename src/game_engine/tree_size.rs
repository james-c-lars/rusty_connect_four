use std::{
    cell::RefCell,
    mem::size_of,
    rc::{Rc, Weak},
};

use crate::game_engine::{
    board_state::{BoardState, ChildState},
    transposition::TranspositionTable,
};

/// Contains different numerical details about the size of a
/// decision tree.
#[derive(Default, Clone, Copy)]
pub struct TreeSize {
    pub depth: usize,
    pub size: usize,
    pub memory: usize,
}

/// Calculates numerical details about a decision tree.
pub fn calculate_size(root: Rc<RefCell<BoardState>>, table: &TranspositionTable) -> TreeSize {
    let mut depth = 0;
    let mut size = 0;

    let mut current_layer = vec![root];
    let mut next_layer = Vec::new();

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

    let mut memory = 0;
    for (_, weak_ref) in table.iter() {
        // Size of the reference in the table
        memory += size_of::<u64>(); // key
        memory += size_of::<Weak<RefCell<BoardState>>>(); // value

        // Size of the reference as a child
        if weak_ref.strong_count() > 0 {
            memory += size_of::<BoardState>();

            memory += size_of::<ChildState>() * weak_ref.strong_count();
        }
    }

    TreeSize {
        depth,
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
        let root = Rc::new(RefCell::new(BoardState::new(
            Board::from_arrays(board_array),
            true,
        )));

        let mut generator = LayerGenerator::new(root.clone(), TranspositionTable::default());
        for _ in 0..(1 + 6 + 36) {
            generator.next();
        }

        let stats = calculate_size(root.clone(), generator.table_ref());

        assert_eq!(stats.depth, 4);
        assert_eq!(stats.size, (1 + 6 + 36 + 216));

        generator.next();

        let stats = calculate_size(root.clone(), generator.table_ref());

        assert_eq!(stats.depth, 5);
        assert_eq!(stats.size, (1 + 6 + 36 + 216) + 6);
    }
}
