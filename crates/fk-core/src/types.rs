use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    Home,
    Away,
}

impl Team {
    pub fn opposite(&self) -> Self {
        match self {
            Team::Home => Team::Away,
            Team::Away => Team::Home,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Handedness {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Beginner,
    Casual,
    Competitive,
    Elite,
    Spirit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayCall {
    VertStack,
    HorizStack,
    Isolation,
    Zone,
    Man,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputContext {
    WithDisc,
    OffenseNoCut,
    Defense,
    Pull,
    Menu,
    Replay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnoverReason {
    Drop,
    OutOfBounds,
    StallOut,
    Interception,
    HandBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwagReason {
    Goal,
    Assist,
    LayoutCatch,
    LayoutFail,
    Callahan,
    Greatest,
    GreatThrow,
    Turnover,
    DropCatch,
    ScoredOn,
    GotBlocked,
    ConsecutiveCompletions,
    TeammateScore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VenueType {
    Park,
    Beach,
    Stadium,
    Indoor,
    ForestClearing,
    Rooftop,
}
