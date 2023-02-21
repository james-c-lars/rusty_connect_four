use eframe::{egui, epaint::CubicBezierShape};
use egui::{Pos2, Id, Stroke, Color32};
use std::sync::{Arc, Mutex, mpsc::{channel, Sender, Receiver}};

use rusty_connect_four::user_interface::{
    board::{Board, PieceState},
    computer::{ComputerState, computer_process, UIMessage, ComputerMessage, GameOver},
};

pub struct App {
    board: Board,
    sender: Sender<UIMessage>,
    receiver: Receiver<ComputerMessage>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (my_sender, computer_receiver) = channel();
        let (computer_sender, my_receiver) = channel();

        let state = Arc::new(Mutex::new(ComputerState::new(
            cc.egui_ctx.clone(),
            computer_sender,
            computer_receiver,
        )));

        let state_clone = state.clone();
        std::thread::spawn(move || {
            computer_process(state_clone);
        });

        Self {
            board: Board::new(Id::new("Board"), Pos2 { x: 10.0, y: 10.0 }),
            sender: my_sender,
            receiver: my_receiver,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // This is the UI creation
            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    ComputerMessage::MoveMade { game_state, move_scores } => {
                        match game_state {
                            GameOver::NoWin => (),
                            GameOver::Tie => {
                                println!("Tie!");
                                self.board.lock();
                            },
                            GameOver::OneWins => {
                                println!("One Wins!");
                                self.board.lock();
                            },
                            GameOver::TwoWins => {
                                println!("Two Wins!");
                                self.board.lock();
                            },
                        }
                    },
                    ComputerMessage::InvalidMove => panic!("Invalid move!!!"),
                    ComputerMessage::MoveScoresUpdate(_) => (),
                }
            }
            
            for (column, response) in self.board.render(ctx, ui) {
                if response.clicked() {
                    self.board.drop_piece(column, PieceState::PlayerOne);
                }
            }
        });

        // Printing to the console to show that things have rerendered
        println!(".");
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
