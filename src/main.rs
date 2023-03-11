use std::sync::mpsc::{channel, Receiver, Sender};

use egui::{Id, Pos2};

use rusty_connect_four::user_interface::{
    board::{Board, PieceState},
    engine_interface::{async_engine_process, EngineMessage, GameOver, UIMessage},
};

/// Stores the current state of the application.
pub struct App {
    board: Board,
    sender: Sender<UIMessage>,
    receiver: Receiver<EngineMessage>,
    current_player: PieceState,
}

impl App {
    /// Sets the initial state of the application.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setting up the engine interface in another thread
        let (my_sender, engine_receiver) = channel();
        let (engine_sender, my_receiver) = channel();

        let ctx_clone = cc.egui_ctx.clone();

        std::thread::spawn(move || {
            async_engine_process(ctx_clone, engine_sender, engine_receiver);
        });

        Self {
            board: Board::new(Id::new("Board"), Pos2 { x: 10.0, y: 10.0 }),
            sender: my_sender,
            receiver: my_receiver,
            current_player: PieceState::PlayerOne,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Communicating with the engine
            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    EngineMessage::MoveMade {
                        game_state,
                        move_scores,
                        board_size,
                    } => match game_state {
                        GameOver::NoWin => {
                            self.board.unlock();
                            self.current_player = self.current_player.reverse();
                            println!(
                                "Move Made - depth: {}, size: {}, memory: {}",
                                board_size.depth, board_size.size, board_size.memory
                            );
                        }
                        GameOver::Tie => {
                            println!("Tie!");
                            self.board.lock();
                        }
                        GameOver::OneWins => {
                            println!("One Wins!");
                            self.board.lock();
                        }
                        GameOver::TwoWins => {
                            println!("Two Wins!");
                            self.board.lock();
                        }
                    },
                    EngineMessage::InvalidMove => panic!("Invalid move!!!"),
                    EngineMessage::Update {
                        move_scores,
                        board_size,
                    } => println!(
                        "depth: {}, size: {}, memory: {}",
                        board_size.depth, board_size.size, board_size.memory
                    ),
                }
            }

            // Generating the UI
            for (column, response) in self.board.render(ctx, ui) {
                if response.clicked() {
                    self.board.drop_piece(ctx, column, self.current_player);
                    self.board.lock();

                    self.sender
                        .send(UIMessage::MakeMove(column))
                        .expect(format!("Sending MakeMove({}) failed", column).as_str());
                }
            }
        });
    }
}

/// Runs the application.
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Connect 4 Engine",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
