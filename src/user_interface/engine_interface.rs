use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use egui::Context;

use crate::game_engine::game_manager::GameManager;
pub use crate::game_engine::game_manager::{BoardSize, GameOver};

/// Stores what the maximum number of nodes we will allow to be generated
/// in the engine.
const MAX_NODES_GENERATED: usize = 1024 * 1024;
/// Stores how many nodes we will generate at once. Higher numbers are more
/// performant, but makes the interface less responsive.
const GENERATED_NODES_PER_ITERATION: usize = 128;

/// Messages that the engine can send to the UI.
pub enum EngineMessage {
    MoveMade {
        game_state: GameOver,
        move_scores: HashMap<u8, isize>,
        board_size: BoardSize,
    },
    InvalidMove,
    Update {
        move_scores: HashMap<u8, isize>,
        board_size: BoardSize,
    },
}

/// Messages that the UI can send to the engine.
pub enum UIMessage {
    MakeMove(usize),
    ResetGame,
}

/// A process meant to be run asynchronously from the UI.
///
/// This process will communicate with the engine according to the
/// messages sent to it from the UI, and will also handle generating
/// new nodes in the engine's decision tree in the downtime.
pub fn async_engine_process(
    ctx: Context,
    sender: Sender<EngineMessage>,
    receiver: Receiver<UIMessage>,
) {
    // Setting the initial state of the process
    let mut manager = GameManager::new_game();
    let mut nodes_generated: usize = 0;

    loop {
        let possible_message = match receiver.try_recv() {
            // If there's a message in the channel we want to address it
            Ok(message) => Some(message),
            // Otherwise we need to choose whether to generate board states or wait
            Err(_) => {
                if nodes_generated >= MAX_NODES_GENERATED {
                    // If our tree is as big as we'll let it be already,
                    // we can block the thread and wait for a message
                    Some(receiver.recv().unwrap())
                } else {
                    // Otherwise we can use the time to continue to grow our tree
                    manager.generate_x_states(GENERATED_NODES_PER_ITERATION);
                    nodes_generated += GENERATED_NODES_PER_ITERATION;

                    sender
                        .send(EngineMessage::Update {
                            move_scores: manager.get_move_scores(),
                            board_size: manager.size(),
                        })
                        .expect(format!("Sending update failed!").as_str());

                    None
                }
            }
        };

        if let Some(message) = possible_message {
            match message {
                UIMessage::MakeMove(column) => {
                    // Telling the engine what move is played, as well as generating a response
                    // for the UI
                    sender
                        .send(match manager.make_move(column as u8) {
                            Ok(_) => {
                                let board_size = manager.size();
                                nodes_generated = board_size.size;

                                EngineMessage::MoveMade {
                                    game_state: manager.is_game_over(),
                                    move_scores: manager.get_move_scores(),
                                    board_size,
                                }
                            }
                            Err(_) => EngineMessage::InvalidMove,
                        })
                        .expect(
                            format!("Sending response to MakeMove({}) failed", column).as_str(),
                        );

                    // Poking the main thread to get it to process the message in a
                    // timely manner
                    ctx.request_repaint();
                }
                UIMessage::ResetGame => {
                    manager = GameManager::new_game();
                    nodes_generated = 0;
                }
            }
        }
    }
}
