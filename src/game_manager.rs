use crate::{
    board::Board,
    board_state::{BoardState, GameOver},
    consts::{BOARD_HEIGHT, BOARD_WIDTH},
    layer_generator::LayerGenerator,
    tree_analysis::how_good_is,
};

static mut CURRENT_BOARD_STATE: Option<BoardState> = Some(BoardState::default_const());
static mut LAYER_GENERATOR: Option<LayerGenerator> = None;

/// Starts a new game with an empty board
pub fn new_game() {
    unsafe {
        CURRENT_BOARD_STATE = Some(BoardState::default_const());
    }
}

/// Starts a new game from a position
/// 
/// The position is given as array[row][col]
pub fn start_from_position(
    position: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    turn: bool,
    last_move: u8,
) {
    unsafe {
        CURRENT_BOARD_STATE = Some(BoardState::new(
            Board::from_arrays(position),
            turn,
            last_move,
        ));
    }
}

/// Returns the current position of the game as array[row][col]
pub fn get_position() -> [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] {
    let board;
    unsafe {
        if let None = CURRENT_BOARD_STATE {
            panic!("Could not find the current board state!");
        }

        board = &CURRENT_BOARD_STATE.as_ref().unwrap().board;
    }

    let mut position = [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            position[row as usize][col as usize] = match board.get_piece(col, row) {
                Ok(piece) => piece as u8,
                Err(_) => 0,
            };
        }
    }

    position
}

/// Generates up to x board states in the decision tree
pub fn generate_x_states(x: isize) {
    let generator;

    unsafe {
        if let None = CURRENT_BOARD_STATE {
            panic!("Could not find the current board state!");
        }

        if let None = LAYER_GENERATOR {
            LAYER_GENERATOR = Some(LayerGenerator::new(CURRENT_BOARD_STATE.as_mut().unwrap()));
            generator = LAYER_GENERATOR.as_mut().unwrap();
        } else {
            generator = LAYER_GENERATOR.as_mut().unwrap();
        }
    }

    for _ in 0..x {
        if let None = generator.next() {
            break;
        }
    }
}

/// Drop a piece down the corresponding column
/// 
/// Returns a tuple containing if the move was made successfully,
/// if the game is over, and who won the game (0 is tie)
pub fn make_move(col: u8) -> (bool, bool, u8) {
    unsafe {
        if let None = CURRENT_BOARD_STATE {
            panic!("Could not find the current board state!");
        }

        for child in CURRENT_BOARD_STATE.as_ref().unwrap().children.iter() {
            if child.get_last_move() == col {
                CURRENT_BOARD_STATE = Some(
                    CURRENT_BOARD_STATE
                        .take()
                        .unwrap()
                        .narrow_possibilities(col)
                );

                return match CURRENT_BOARD_STATE.as_ref().unwrap().is_game_over() {
                    GameOver::NoWin => (true, false, 0),
                    GameOver::Tie => (true, true, 0),
                    GameOver::OneWins => (true, true, 1),
                    GameOver::TwoWins => (true, true, 2),
                };
            }
        }
    }

    (false, false, 0)
}

/// Returns a vector of moves and their corresponding scores
pub fn get_move_scores() -> Vec<(u8, isize)> {
    let mut move_scores = Vec::new();

    let child_iter;
    let whose_turn;

    unsafe {
        if let None = CURRENT_BOARD_STATE {
            panic!("Could not find the current board state!");
        }

        child_iter = CURRENT_BOARD_STATE.as_ref().unwrap().children.iter();
        whose_turn = CURRENT_BOARD_STATE.as_ref().unwrap().get_turn();
    }

    for child in child_iter {
        let child_score = if whose_turn {
            how_good_is(child)
        } else {
            -how_good_is(child)
        };

        move_scores.push((child.get_last_move(), child_score));
    }

    move_scores
}
