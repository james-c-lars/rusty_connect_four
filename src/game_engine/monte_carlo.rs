use std::collections::HashMap;

use rand::{seq::SliceRandom, rngs::ThreadRng};

use crate::game_engine::{
    board_state::{BoardState, ChildState},
    game_manager::GameOver,
    transposition::TranspositionStateTable,
};

/// The exploration constant governing how the monte carlo search weighs exploration versus
///  exploitation. Currently square root of two.
const C: f32 = 1.414; // TODO: Find the best exploration constant via experimentation

/// The results of the rollouts conducted from a node.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RolloutResults {
    /// Success is 2 * the number of wins + the number of ties
    success: u32,
    /// Total is 2 * the number of rollouts
    total: u32,
}

/// Returns a score judging how worthwhile it is to perform a rollout for a given node.
///
/// Equation taken from: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation.
fn upper_confidence_bound_one(parent: &RolloutResults, child: &RolloutResults) -> f32 {
    let w = child.success as f32 / 2.0;
    let n = child.total as f32 / 2.0;
    let p = parent.total as f32 / 2.0;

    w / n + C * (p.ln() / n).sqrt()
}

impl BoardState {
    /// Gets the value of each child move.
    pub fn move_scores(&self) -> HashMap<u8, f32> {
        let mut scores = HashMap::new();

        for child in self.children.iter() {
            let results = &child.state.borrow().rollout_results;
            let score = results.success as f32 / results.total as f32;
            scores.insert(child.get_last_move(), score);
        }

        scores
    }

    /// Does a rollout for each child of the BoardState.
    pub fn generate_rollouts(&mut self, table: &mut TranspositionStateTable, thread_rng: &mut ThreadRng) {
        for child in self.children.iter() {
            let mut child_state = child.state.borrow_mut();
            if child_state.is_game_over() == GameOver::NoWin {
                child_state.selection(table, thread_rng);
            }
        }
    }

    /// Navigates the tree to find a new leaf node, and then calls simulation on the leaf node.
    /// 
    /// Chooses children to explore based on UCB1.
    fn selection(&mut self, table: &mut TranspositionStateTable, thread_rng: &mut ThreadRng) -> GameOver {
        // We've hit a leaf node and can begin the rollout simulation
        if self.children.len() == 0 {
            let result = self.simulation(table, thread_rng);
            self.update_rollout_results(result);
            return result;
        }

        let mut potential_moves: Vec<(f32, &ChildState)> = self
            .children
            .iter()
            .map(|c| {
                (
                    upper_confidence_bound_one(
                        &self.rollout_results,
                        &c.state.borrow().rollout_results,
                    ),
                    c,
                )
            })
            .collect();

        // The moves with the highest UCB1 scores will be explored first
        potential_moves.sort_by(|(one, _), (two, _)| two.partial_cmp(one).unwrap());

        let (_, move_to_explore) = potential_moves.pop().unwrap();
        let result = move_to_explore.state.borrow_mut().selection(table, thread_rng);
        self.update_rollout_results(result);

        result
    }

    /// Performs a random rollout on a leaf node.
    /// 
    /// Generates children and then randomly selects one to continue from.
    fn simulation(&mut self, table: &mut TranspositionStateTable, thread_rng: &mut ThreadRng) -> GameOver {
        if self.is_game_over() != GameOver::NoWin {
            let result = self.is_game_over();
            self.update_rollout_results(result);
            return result;
        }

        let children = self.generate_children(table);
        let mut random_child = children.choose(thread_rng).unwrap().borrow_mut();

        let result = random_child.simulation(table, thread_rng);
        self.update_rollout_results(result);

        result
    }

    fn update_rollout_results(&mut self, result: GameOver) {
        match result {
            GameOver::NoWin => panic!(
                "Encountered a NoWin value when exploring {:?}",
                self
            ),
            GameOver::Tie => {
                self.rollout_results.success += 1;
            }
            GameOver::OneWins => {
                if !self.get_turn() {
                    self.rollout_results.success += 2;
                }
            }
            GameOver::TwoWins => {
                if self.get_turn() {
                    self.rollout_results.success += 2;
                }
            }
        }

        self.rollout_results.total += 2;
    }
}
