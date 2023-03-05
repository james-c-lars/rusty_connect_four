use eframe::{egui, epaint::CubicBezierShape};
use egui::{Color32, Id, Pos2, Stroke};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use rusty_connect_four::user_interface::{
    board::{Board, PieceState},
    computer::{computer_process, ComputerMessage, GameOver, UIMessage},
};

pub struct App {
    board: Board,
    sender: Sender<UIMessage>,
    receiver: Receiver<ComputerMessage>,
    current_player: PieceState,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (my_sender, computer_receiver) = channel();
        let (computer_sender, my_receiver) = channel();

        let ctx_clone = cc.egui_ctx.clone();

        std::thread::spawn(move || {
            computer_process(ctx_clone, computer_sender, computer_receiver);
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
            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    ComputerMessage::MoveMade {
                        game_state,
                        move_scores,
                    } => match game_state {
                        GameOver::NoWin => {
                            self.board.unlock();
                            self.current_player = self.current_player.reverse();
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
                    ComputerMessage::InvalidMove => panic!("Invalid move!!!"),
                    ComputerMessage::MoveScoresUpdate(_) => (),
                }
            }

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

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Connect 4 Engine",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
