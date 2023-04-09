use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use egui::Context;

pub use crate::game_engine::game_manager::{GameOver, TreeSize};
use crate::{
    game_engine::game_manager::GameManager,
    log::{log_message, LogType},
};

/// Stores what the maximum amount of memory we will allow to be used by the engine.
const MAX_MEMORY_USAGE: usize = 256 * 1024 * 1024;
/// Stores how many nodes we will generate at once. Higher numbers are more
/// performant, but makes the interface less responsive.
const GENERATED_NODES_PER_ITERATION: usize = 128 * 1024;

/// Messages that the engine can send to the UI.
#[derive(Debug)]
pub enum EngineMessage {
    MoveReceipt {
        game_state: GameOver,
        move_scores: HashMap<u8, isize>,
        tree_size: TreeSize,
    },
    InvalidMove(String),
    Update {
        move_scores: HashMap<u8, isize>,
        tree_size: TreeSize,
    },
}

/// Messages that the UI can send to the engine.
#[derive(Debug)]
pub enum UIMessage {
    MakeMove(usize),
    ResetGame,
    RequestUpdate,
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
    let mut memory_usage: usize = 0;
    let mut tree_complete = false;
    let mut time_since_last_update = Instant::now();

    loop {
        let possible_message = match receiver.try_recv() {
            // If there's a message in the channel we want to address it
            Ok(message) => Some(message),
            // Otherwise we need to choose whether to generate board states or wait
            Err(_) => {
                if memory_usage >= MAX_MEMORY_USAGE || tree_complete {
                    log_message(
                        LogType::MaxMemHit,
                        format!("Max Memory Hit -  tree complete: {}", tree_complete),
                    );

                    send_update(&sender, &manager, &mut memory_usage);
                    poke_main_thread(&ctx);

                    // If our tree is as big as we'll let it be already, we can block the thread
                    // and wait for a message
                    // recv only fails if the other side has disconnected, in which case we want
                    // to gracefully exit
                    log_message(LogType::AsyncMessage, "Waiting for UI Message".to_owned());
                    match receiver.recv() {
                        Ok(message) => Some(message),
                        Err(_) => break,
                    }
                } else {
                    log_message(LogType::Detail, "Growing tree".to_owned());
                    grow_tree(&mut manager, &mut tree_complete, &mut memory_usage);

                    None
                }
            }
        };

        if let Some(message) = possible_message {
            log_message(
                LogType::AsyncMessage,
                format!("UIMessage Received - {:?}", message),
            );

            match message {
                UIMessage::MakeMove(column) => {
                    let response = try_make_move(&mut manager, column, &mut memory_usage);

                    sender.send(response).expect(
                        format!("Sending response to MakeMove({}) failed", column).as_str(),
                    );
                    poke_main_thread(&ctx);
                    time_since_last_update = Instant::now();
                }
                UIMessage::ResetGame => {
                    manager = GameManager::new_game();
                    memory_usage = 0;
                    tree_complete = false;
                }
                UIMessage::RequestUpdate => {
                    send_update(&sender, &manager, &mut memory_usage);
                    poke_main_thread(&ctx);
                    time_since_last_update = Instant::now();
                }
            }

            log_message(
                LogType::AsyncMessage,
                "UI Message done processing".to_owned(),
            );
        }

        // Sending periodic updates to the UI
        if time_since_last_update.elapsed().as_secs() > 1 {
            log_message(LogType::AsyncMessage, "Sending periodic update".to_owned());

            send_update(&sender, &manager, &mut memory_usage);
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
    memory_usage: &mut usize,
) -> EngineMessage {
    match manager.make_move(column as u8) {
        Ok(()) => {
            let tree_size = manager.size();
            *memory_usage = tree_size.memory;

            EngineMessage::MoveReceipt {
                game_state: manager.is_game_over(),
                move_scores: manager.get_move_scores(),
                tree_size,
            }
        }
        Err(error_message) => EngineMessage::InvalidMove(error_message),
    }
}

/// Grows the size of the decision tree.
fn grow_tree(manager: &mut GameManager, tree_complete: &mut bool, memory_usage: &mut usize) {
    let current_generated = manager.try_generate_x_states(GENERATED_NODES_PER_ITERATION);
    *tree_complete = current_generated < GENERATED_NODES_PER_ITERATION;
    *memory_usage = manager.size().memory;
}

/// Sends an update to the UI of the current engine state.
fn send_update(sender: &Sender<EngineMessage>, manager: &GameManager, memory_usage: &mut usize) {
    let tree_size = manager.size();
    *memory_usage = tree_size.memory;

    sender
        .send(EngineMessage::Update {
            move_scores: manager.get_move_scores(),
            tree_size,
        })
        .expect(format!("Sending update failed!").as_str());
}
