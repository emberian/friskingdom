use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    AiDecision,
    Movement,
    DiscPhysics,
    Collision,
    GameRules,
    Animation,
    Camera,
    Rendering,
    Ui,
}
