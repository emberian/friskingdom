use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    PostGame,
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::InGame)]
pub enum GamePhase {
    #[default]
    PrePoint,
    Pull,
    LivePlay,
    PointScored,
    HalfTime,
    GameOver,
}
