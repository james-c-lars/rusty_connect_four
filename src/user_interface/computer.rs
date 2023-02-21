use std::{sync::{
    Mutex,
    Arc,
    mpsc::{Sender, Receiver},
}, collections::HashMap};

use egui::Context;

use crate::game_engine::game_manager::GameManager;
pub use crate::game_engine::game_manager::GameOver;

const MAX_NODES_GENERATED: usize = 1024 * 1024;
const GENERATED_NODES_PER_ITERATION: usize = 128;

pub enum ComputerMessage {
    MoveMade {
        game_state: GameOver,
        move_scores: HashMap<u8, isize>,
    },
    InvalidMove,
    MoveScoresUpdate(HashMap<u8, isize>),
}

pub enum UIMessage {
    MakeMove(usize),
    ResetGame,
}

pub struct ComputerState {
    ctx: Context,
    sender: Sender<ComputerMessage>,
    receiver: Receiver<UIMessage>,
}

impl ComputerState {
    pub fn new(
        ctx: Context,
        sender: Sender<ComputerMessage>,
        receiver: Receiver<UIMessage>
    ) -> Self {
        Self {
            ctx,
            sender,
            receiver
        }
    }
}

pub fn computer_process(state: Arc<Mutex<ComputerState>>) {
    let mut manager = GameManager::new_game();
    let mut nodes_generated: usize = 0;

    loop {
        let possible_message = match state.lock().unwrap().receiver.try_recv() {
            // If there's a message in the tube we want to address it
            Ok(message) => Some(message),
            // Otherwise we need to choose whether to generate board states or wait
            Err(_) => {
                if nodes_generated >= MAX_NODES_GENERATED {
                    // If our tree is as big as we'll let it be already,
                    // we can wait for a message
                    Some(state.lock().unwrap().receiver.recv().unwrap())
                } else {
                    // Otherwise we can use the time to continue to grow our tree
                    manager.generate_x_states(GENERATED_NODES_PER_ITERATION);
                    nodes_generated += GENERATED_NODES_PER_ITERATION;

                    None
                }
            },
        };

        if let Some(message) = possible_message {
            match message {
                UIMessage::MakeMove(column) => {
                    state.lock().unwrap().sender.send(
                        match manager.make_move(column as u8) {
                            Ok(_) => {
                                // Guessing how many nodes are left
                                nodes_generated /= 7;

                                ComputerMessage::MoveMade {
                                    game_state: manager.is_game_over(),
                                    move_scores: manager.get_move_scores(),
                                }
                            },
                            Err(_) => ComputerMessage::InvalidMove,
                        }
                    ).unwrap();

                    // Poking the main thread to get it to process the message in a
                    // timely manner
                    state.lock().unwrap().ctx.request_repaint();
                },
                UIMessage::ResetGame => {
                    manager = GameManager::new_game();
                    nodes_generated = 0;
                },
            }
        }
    }
}