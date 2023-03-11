use std::{cell::RefCell, rc::Rc};

use crate::game_engine::board_state::BoardState;

/// Contains different numerical details about the size of a
/// decision tree.
pub struct BoardSize {
    pub depth: usize,
    pub size: usize,
    pub memory: usize,
}

/// Calculates numerical details about a decision tree.
pub fn calculate_size(root: Rc<RefCell<BoardState>>) -> BoardSize {
    let mut depth = 0;
    let mut size = 0;

    let mut current_layer = vec![root];
    let mut next_layer = Vec::new();

    while let Some(current_node) = current_layer.pop() {
        size += 1;
        next_layer.extend(current_node.borrow().children.iter().map(|n| n.clone()));

        if current_layer.len() == 0 {
            current_layer = next_layer;
            next_layer = Vec::new();
            depth += 1;
        }
    }

    BoardSize {
        depth,
        size,
        memory: size * std::mem::size_of::<Rc<RefCell<BoardState>>>(),
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::game_engine::{
        board::Board, board_size::calculate_size, board_state::BoardState,
        layer_generator::LayerGenerator,
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
            6,
        )));

        let mut generator = LayerGenerator::new(root.clone());
        for _ in 0..(1 + 6 + 36) {
            generator.next();
        }

        let stats = calculate_size(root.clone());

        assert_eq!(stats.depth, 4);
        assert_eq!(stats.size, (1 + 6 + 36 + 216));

        generator.next();

        let stats = calculate_size(root.clone());

        assert_eq!(stats.depth, 5);
        assert_eq!(stats.size, (1 + 6 + 36 + 216) + 6);
    }
}
