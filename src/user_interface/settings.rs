#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Computer,
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct Settings {
    pub players: [PlayerType; 2],
    pub delay: f32,
    pub difficulty: Difficulty,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            players: [PlayerType::Human, PlayerType::Computer],
            delay: 3.0,
            difficulty: Difficulty::Hard,
        }
    }
}
