use bevy::prelude::*;

use crate::types::{PlayerId, SwagReason, Team, TurnoverReason};

// ---------------------------------------------------------------------------
// Disc events
// ---------------------------------------------------------------------------

#[derive(Message, Debug, Clone)]
pub struct DiscThrown {
    pub thrower: PlayerId,
    pub throw_type: frisbee_physics::ThrowType,
}

#[derive(Message, Debug, Clone)]
pub struct DiscCaught {
    pub catcher: PlayerId,
    pub position: Vec3,
    pub was_layout: bool,
}

#[derive(Message, Debug, Clone)]
pub struct DiscDropped {
    pub position: Vec3,
}

#[derive(Message, Debug, Clone)]
pub struct DiscOutOfBounds {
    pub crossing_point: Vec3,
}

#[derive(Message, Debug, Clone)]
pub struct DiscLanded {
    pub position: Vec3,
}

// ---------------------------------------------------------------------------
// Game flow events
// ---------------------------------------------------------------------------

#[derive(Message, Debug, Clone)]
pub struct PointScored {
    pub scoring_team: Team,
    pub scorer: PlayerId,
    pub assister: Option<PlayerId>,
    pub is_callahan: bool,
}

#[derive(Message, Debug, Clone)]
pub struct TurnoverOccurred {
    pub reason: TurnoverReason,
    pub position: Vec3,
    pub new_offense: Team,
}

#[derive(Message, Debug, Clone)]
pub struct StallOutEvent {
    pub thrower: PlayerId,
}

#[derive(Message, Debug, Clone)]
pub struct PlayerSwitched {
    pub new_controlled: PlayerId,
    pub previous: Option<PlayerId>,
}

// ---------------------------------------------------------------------------
// Swag events
// ---------------------------------------------------------------------------

#[derive(Message, Debug, Clone)]
pub struct SwagEvent {
    pub player: PlayerId,
    pub delta: f32,
    pub reason: SwagReason,
}
