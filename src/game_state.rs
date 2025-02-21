use bevy::prelude::{self, States};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Initialization,
    DecideFirstToPlay,
    CardsDistribution,
    GameLoop,
    CollectCards,
}
