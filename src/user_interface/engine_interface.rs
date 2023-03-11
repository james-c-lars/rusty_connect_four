use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use egui::Context;

use crate::game_engine::game_manager::GameManager;
pub use crate::game_engine::game_manager::{BoardSize, GameOver};

/// Stores what the maximum number of nodes we will allow to be generated
/// in the engine.
const MAX_NODES_ALLOWED: usize = 4 * 1024 * 1024;
/// Stores how many nodes we will generate at once. Higher numbers are more
/// performant, but makes the interface less responsive.
const GENERATED_NODES_PER_ITERATION: usize = 64 * 1024;

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
    let mut tree_complete = false;
    let mut time_since_last_update = Instant::now();

    loop {
        let possible_message = match receiver.try_recv() {
            // If there's a message in the channel we want to address it
            Ok(message) => Some(message),
            // Otherwise we need to choose whether to generate board states or wait
            Err(_) => {
                if nodes_generated >= MAX_NODES_ALLOWED || tree_complete {
                    send_update(&sender, &manager);
                    poke_main_thread(&ctx);

                    // If our tree is as big as we'll let it be already, we can block the thread
                    // and wait for a message
                    // recv only fails if the other side has disconnected, in which case we want
                    // to gracefully exit
                    match receiver.recv() {
                        Ok(message) => Some(message),
                        Err(_) => break,
                    }
                } else {
                    grow_tree(&mut manager, &mut tree_complete, &mut nodes_generated);

                    None
                }
            }
        };

        if let Some(message) = possible_message {
            match message {
                UIMessage::MakeMove(column) => {
                    let response = try_make_move(&mut manager, column, &mut nodes_generated);

                    sender.send(response).expect(
                        format!("Sending response to MakeMove({}) failed", column).as_str(),
                    );
                    poke_main_thread(&ctx);

                    time_since_last_update = Instant::now();
                }
                UIMessage::ResetGame => {
                    manager = GameManager::new_game();
                    nodes_generated = 0;
                    tree_complete = false;
                    time_since_last_update = Instant::now();
                }
            }
        }

        // Sending periodic updates to the UI
        if time_since_last_update.elapsed().as_secs() > 1 {
            send_update(&sender, &manager);
            poke_main_thread(&ctx);

            time_since_last_update = Instant::now();
        }
    }
}

/// 'Pokes' the main thread to get it to rerender.
///
/// Used to ensure the UI responds to a message in a timely fashion.
fn poke_main_thread(ctx: &Context) {
    ctx.request_repaint();
}

/// Tries to make a move, and returns a response corresponding to if it was successful.
fn try_make_move(
    manager: &mut GameManager,
    column: usize,
    nodes_generated: &mut usize,
) -> EngineMessage {
    match manager.make_move(column as u8) {
        Ok(()) => {
            let board_size = manager.size();
            *nodes_generated = board_size.size;

            EngineMessage::MoveMade {
                game_state: manager.is_game_over(),
                move_scores: manager.get_move_scores(),
                board_size,
            }
        }
        Err(_) => EngineMessage::InvalidMove,
    }
}

/// Grows the size of the decision tree.
fn grow_tree(manager: &mut GameManager, tree_complete: &mut bool, nodes_generated: &mut usize) {
    let current_generated = manager.try_generate_x_states(GENERATED_NODES_PER_ITERATION);
    *tree_complete = current_generated < GENERATED_NODES_PER_ITERATION;
    *nodes_generated += current_generated;
}

/// Sends an update to the UI of the current engine state.
fn send_update(sender: &Sender<EngineMessage>, manager: &GameManager) {
    sender
        .send(EngineMessage::Update {
            move_scores: manager.get_move_scores(),
            board_size: manager.size(),
        })
        .expect(format!("Sending update failed!").as_str());
}
