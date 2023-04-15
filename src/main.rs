use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use egui::{Id, Pos2};

use rusty_connect_four::{
    log::{log_message, LogType},
    user_interface::{
        board::Board,
        engine_interface::{async_engine_process, EngineMessage, TreeSize, UIMessage},
        settings::{Settings, PlayerType},
        turn_manager::TurnManager,
    },
};

/// Stores the current state of the application.
pub struct App {
    board: Board,
    sender: Sender<UIMessage>,
    receiver: Receiver<EngineMessage>,
    settings: Settings,
    turn_manager: TurnManager,
    tree_size: TreeSize,
    move_scores: HashMap<u8, isize>,
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

        // Other set-up
        let settings = Settings::new();
        let turn_manager = TurnManager::new(settings.players);
        let mut board = Board::new(Id::new("Board"), Pos2 { x: 0.0, y: 0.0 });
        if settings.players[0] == PlayerType::Computer {
            board.lock();
        }

        Self {
            board,
            sender: my_sender,
            receiver: my_receiver,
            settings,
            turn_manager,
            tree_size: Default::default(),
            move_scores: HashMap::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Communicating with the engine
            if let Ok(message) = self.receiver.try_recv() {
                log_message(
                    LogType::AsyncMessage,
                    format!("EngineMessage Received - {:?}", message),
                );

                match message {
                    EngineMessage::MoveReceipt {
                        game_state,
                        move_scores,
                        tree_size,
                    } => {
                        self.tree_size = tree_size;
                        self.move_scores = move_scores;

                        self.turn_manager.move_receipt(
                            game_state,
                            ctx,
                            &mut self.board,
                            &self.settings,
                        );
                    }
                    EngineMessage::InvalidMove(error) => panic!("{}", error),
                    EngineMessage::Update {
                        move_scores,
                        tree_size,
                    } => {
                        self.tree_size = tree_size;
                        self.move_scores = move_scores;

                        self.turn_manager.update_received(
                            &self.move_scores,
                            ctx,
                            &mut self.board,
                            &self.settings,
                        );

                        log_message(
                            LogType::EngineUpdate,
                            format!(
                                "Engine Update - depth: {}, size: {}, memory: {}",
                                tree_size.depth, tree_size.size, tree_size.memory
                            ),
                        );
                    }
                }
            }

            self.turn_manager
                .process_turn(ctx, &mut self.board, &self.settings, &self.sender);

            // Generating the UI
            for (column, response) in self.board.render(ctx, ui) {
                if response.clicked() {
                    self.board
                        .drop_piece(ctx, column, self.turn_manager.current_player);
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
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Board::board_size());

    eframe::run_native(
        "Connect 4 Engine",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap();
}
