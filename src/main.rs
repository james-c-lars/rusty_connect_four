use eframe::egui;
use std::sync::{Arc, Mutex};

mod game_engine;

use game_engine::game_manager::GameManager;

fn slow_process(state_clone: Arc<Mutex<State>>) {
    let _manager = GameManager::new_game();

    loop {
        let duration = 1000;
        println!("going to sleep for {}ms", duration);
        std::thread::sleep(std::time::Duration::from_millis(duration));
        state_clone.lock().unwrap().duration = duration;
        let ctx = &state_clone.lock().unwrap().ctx;
        match ctx {
            Some(x) => x.request_repaint(),
            None => panic!("error in Option<>"),
        }
    }
}

struct State {
    duration: u64,
    ctx: Option<egui::Context>,
}

impl State {
    pub fn new() -> Self {
        Self {
            duration: 0,
            ctx: None,
        }
    }
}

pub struct App {
    state: Arc<Mutex<State>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(State::new()));
        state.lock().unwrap().ctx = Some(cc.egui_ctx.clone());
        let state_clone = state.clone();
        std::thread::spawn(move || {
            slow_process(state_clone);
        });
        Self { state }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // This is the UI creation
            
            // Connect 4 heading
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            
                ui.label("Connect 4");
            });

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
