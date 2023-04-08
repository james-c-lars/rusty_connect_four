use std::{collections::HashMap, sync::mpsc::Sender, time::Instant};

use egui::Context;
use rand::seq::SliceRandom;

use crate::{
    consts::BOARD_WIDTH,
    user_interface::{
        board::{Board, PieceState},
        engine_interface::{GameOver, UIMessage},
        settings::{Difficulty, PlayerType, Settings},
    },
};

/// The turn manager devides a computer's turn up into multiple stages.
///
/// WaitingForMoveReceipt is the default stage of waiting to receive notice that a move has been made.
/// Delay allows for some delay before the computer makes its move.
/// WaitingForUpdate is when the turn manager is no longer delaying and now wants to make a move.
/// AnimateToChosenColumn is after the turn manager knows what move it wants to make.
/// GameOver is when the turn manager is no longer processing due to the game being over.
#[derive(Debug, PartialEq, Eq)]
enum TurnStage {
    WaitingForMoveReceipt,
    Delay {
        start: Instant,
        animating_to_column: usize,
    },
    WaitingForUpdate {
        animating_to_column: usize,
    },
    AnimateToChosenColumn {
        chosen_column: usize,
    },
    GameOver,
}

/// Handles transitioning a board between being open for player input and waiting for
/// the computer to make a move.
pub struct TurnManager {
    pub current_player: PieceState,
    current_player_type: PlayerType,
    stage: TurnStage,
}

impl TurnManager {
    /// Creates a new TurnManager.
    pub fn new(players: [PlayerType; 2]) -> TurnManager {
        TurnManager {
            current_player: PieceState::PlayerOne,
            current_player_type: players[0],
            // We're assuming the first player to go is a human by default
            stage: TurnStage::WaitingForMoveReceipt,
        }
    }

    /// Alerts the TurnManager that a move has been made.
    ///
    /// This method handles transitioning between players's turns.
    pub fn move_receipt(
        &mut self,
        game_state: GameOver,
        ctx: &Context,
        board: &mut Board,
        settings: &Settings,
    ) {
        if self.stage != TurnStage::WaitingForMoveReceipt {
            panic!(
                "Received move receipt while in turn stage: {:?}",
                self.stage
            );
        }

        if self.is_game_over(game_state) {
            board.lock();
            self.stage = TurnStage::GameOver;
            return;
        }

        // It is now the other player's turn
        self.current_player = self.current_player.reverse();

        self.current_player_type = match self.current_player {
            PieceState::PlayerOne => settings.players[0],
            PieceState::PlayerTwo => settings.players[1],
            PieceState::Empty => panic!("Current player is empty"),
        };

        if self.current_player_type == PlayerType::Human {
            board.unlock();

            // We stay waiting for a receipt
            return;
        }

        // If the computer is going next, we can start the delay animation
        board.animate_floater(ctx, 0, 0.0);

        self.stage = TurnStage::Delay {
            start: Instant::now(),
            animating_to_column: BOARD_WIDTH as usize - 1,
        };
    }

    /// Returns whether the game state indicates that the game is over.
    fn is_game_over(&self, game_state: GameOver) -> bool {
        match game_state {
            GameOver::NoWin => false,
            GameOver::Tie => {
                println!("Tie!");
                true
            }
            GameOver::OneWins => {
                println!("Player One Wins!");
                true
            }
            GameOver::TwoWins => {
                println!("Player Two Wins!");
                true
            }
        }
    }

    /// Alerts the Turn Manager that the computer has sent an update.
    pub fn update_received(
        &mut self,
        move_scores: &HashMap<u8, isize>,
        ctx: &Context,
        board: &mut Board,
        settings: &Settings,
    ) {
        if let TurnStage::WaitingForUpdate {
            animating_to_column: _,
        } = self.stage
        {
            board.cancel_animation(ctx);

            self.stage = TurnStage::AnimateToChosenColumn {
                chosen_column: choose_computer_move(move_scores, settings),
            };
        }
    }

    /// Handles the main logic for processing a turn.
    pub fn process_turn(
        &mut self,
        ctx: &Context,
        board: &mut Board,
        settings: &Settings,
        sender: &Sender<UIMessage>,
    ) {
        let mut next_stage = None;

        match &mut self.stage {
            TurnStage::WaitingForMoveReceipt => (), // continue
            TurnStage::Delay {
                start,
                animating_to_column,
            } => {
                passively_animate_floater(ctx, board, animating_to_column);

                if start.elapsed().as_secs_f32() > settings.delay {
                    sender
                        .send(UIMessage::RequestUpdate)
                        .expect("Couldn't send RequestUpdate");

                    next_stage = Some(TurnStage::WaitingForUpdate {
                        animating_to_column: *animating_to_column,
                    });
                }
            }
            TurnStage::WaitingForUpdate {
                animating_to_column,
            } => {
                passively_animate_floater(ctx, board, animating_to_column);
            }
            TurnStage::AnimateToChosenColumn { chosen_column } => {
                let completed_animation = board.animate_floater(ctx, *chosen_column, 1.0);

                if completed_animation {
                    board.cancel_animation(ctx);
                    board.drop_piece(ctx, *chosen_column, self.current_player);

                    sender
                        .send(UIMessage::MakeMove(*chosen_column))
                        .expect("Couldn't send move to interface");

                    next_stage = Some(TurnStage::WaitingForMoveReceipt);
                }
            }
            TurnStage::GameOver => (), // continue
        }

        if let Some(stage) = next_stage {
            self.stage = stage;
        }
    }
}

/// Animates the floater piece as going left and right.
///
/// animating_to_column will be modified as the floater changes which direction it's floating.
fn passively_animate_floater(ctx: &Context, board: &mut Board, animating_to_column: &mut usize) {
    let completed_animation = board.animate_floater(ctx, *animating_to_column, 1.5);

    if completed_animation {
        *animating_to_column = BOARD_WIDTH as usize - 1 - *animating_to_column;
    }
}

/// Chooses a move based on the difficulty setting and the engine's move scores.
fn choose_computer_move(move_scores: &HashMap<u8, isize>, settings: &Settings) -> usize {
    if move_scores.len() == 0 {
        panic!("Trying to pick a move when no moves are valid");
    }

    // Highest scoring moves at the end
    let mut sorted_moves = move_scores
        .iter()
        .map(|(column, score)| (*score, *column))
        .collect::<Vec<(isize, u8)>>();
    sorted_moves.sort();

    match settings.difficulty {
        Difficulty::Easy => easy_choose_move(sorted_moves) as usize,
        Difficulty::Medium => medium_choose_move(sorted_moves) as usize,
        Difficulty::Hard => sorted_moves.pop().unwrap().1 as usize,
    }
}

/// Picks one of the moves in the sorted_moves Vector.
///
/// Higher rated moves are more likely to be picked.
fn easy_choose_move(sorted_moves: Vec<(isize, u8)>) -> u8 {
    let mut weighted_moves = Vec::new();
    for (index, (_, column)) in sorted_moves.into_iter().enumerate() {
        for _ in 0..(index + 1) {
            weighted_moves.push(column);
        }
    }

    *weighted_moves.choose(&mut rand::thread_rng()).unwrap()
}

/// Picks one of the moves in the sorted_moves Vector.
///
/// Higher rated moves are more likely to be picked and losing moves will not be considered.
fn medium_choose_move(sorted_moves: Vec<(isize, u8)>) -> u8 {
    let backup_move = sorted_moves[0].1;

    let no_losing_moves = sorted_moves
        .into_iter()
        .filter(|(score, _)| *score != isize::MIN)
        .collect::<Vec<(isize, u8)>>();
    if no_losing_moves.len() == 0 {
        return backup_move;
    }

    easy_choose_move(no_losing_moves)
}
