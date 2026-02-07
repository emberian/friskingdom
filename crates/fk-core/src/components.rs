use bevy::prelude::*;
use std::collections::VecDeque;

use crate::types::{Handedness, PlayerId, Team};

// ---------------------------------------------------------------------------
// Player components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub handedness: Handedness,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMember {
    pub team: Team,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerStats {
    pub speed: f32,
    pub acceleration: f32,
    pub throwing_power: f32,
    pub throwing_accuracy: f32,
    pub catching: f32,
    pub agility: f32,
    pub jumping: f32,
    pub endurance: f32,
    pub disc_iq: f32,
}

impl PlayerStats {
    /// Generate plausible stats for a handler archetype.
    /// Handlers have high throwing stats and medium speed.
    pub fn random_handler(rng: &mut impl rand::Rng) -> Self {
        Self {
            speed: rng.gen_range(0.4..0.7),
            acceleration: rng.gen_range(0.4..0.7),
            throwing_power: rng.gen_range(0.7..0.95),
            throwing_accuracy: rng.gen_range(0.7..0.95),
            catching: rng.gen_range(0.6..0.85),
            agility: rng.gen_range(0.5..0.75),
            jumping: rng.gen_range(0.3..0.65),
            endurance: rng.gen_range(0.5..0.8),
            disc_iq: rng.gen_range(0.7..0.95),
        }
    }

    /// Generate plausible stats for a cutter archetype.
    /// Cutters have high speed and medium throwing stats.
    pub fn random_cutter(rng: &mut impl rand::Rng) -> Self {
        Self {
            speed: rng.gen_range(0.7..0.95),
            acceleration: rng.gen_range(0.65..0.9),
            throwing_power: rng.gen_range(0.35..0.65),
            throwing_accuracy: rng.gen_range(0.35..0.65),
            catching: rng.gen_range(0.65..0.9),
            agility: rng.gen_range(0.6..0.85),
            jumping: rng.gen_range(0.55..0.85),
            endurance: rng.gen_range(0.5..0.8),
            disc_iq: rng.gen_range(0.4..0.7),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
    pub exhausted: bool,
}

impl Stamina {
    /// Create a new Stamina from an endurance stat (0.0 to 1.0).
    /// Max stamina: 80 + 40 * endurance
    /// Regen rate: 5 + 5 * endurance
    pub fn new(endurance: f32) -> Self {
        let max = 80.0 + 40.0 * endurance;
        Self {
            current: max,
            max,
            regen_rate: 5.0 + 5.0 * endurance,
            exhausted: false,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SwagMeter {
    pub current: f32,
    pub base: f32,
    pub momentum: f32,
    pub decay_rate: f32,
}

impl Default for SwagMeter {
    fn default() -> Self {
        Self {
            current: 0.5,
            base: 0.5,
            momentum: 0.0,
            decay_rate: 0.02,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerRole {
    Handler,
    Cutter,
    Hybrid,
}

// ---------------------------------------------------------------------------
// Disc components
// ---------------------------------------------------------------------------

/// Marker component for the disc entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Disc;

#[derive(Component, Debug, Clone)]
pub struct DiscPhysicsState {
    pub state: frisbee_physics::DiscState,
    pub in_flight: bool,
    pub grounded: bool,
    pub held_by: Option<PlayerId>,
}

/*ROBOTODO: DiscVisual is defined but never constructed or used. Should be
  attached to disc entity and updated by a render system to drive spin blur
  and trail rendering. See design-docs/07-rendering.md */
#[derive(Component, Debug, Clone)]
pub struct DiscVisual {
    pub spin_rate_visual: f32,
    pub trail_positions: VecDeque<Vec3>,
}

// ---------------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Copy)]
pub struct Field {
    pub length: f32,
    pub width: f32,
    pub end_zone_depth: f32,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            length: 100.0,
            width: 37.0,
            end_zone_depth: 18.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the entity currently controlled by the player.
#[derive(Component, Debug, Clone, Copy)]
pub struct Controlled;

/// Marks an AI-controlled entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct AiControlled;

/// Marks a player on the offensive team.
#[derive(Component, Debug, Clone, Copy)]
pub struct OnOffense;

/// Marks a player on the defensive team.
#[derive(Component, Debug, Clone, Copy)]
pub struct OnDefense;

/// Marks the player currently holding the disc.
#[derive(Component, Debug, Clone, Copy)]
pub struct HasDisc;

/// Marks a defender assigned as the mark.
#[derive(Component, Debug, Clone, Copy)]
pub struct MarkerDefender;

/// Marks a player designated as the active cut target.
#[derive(Component, Debug, Clone, Copy)]
pub struct CutTarget;

/// Marker for the primary camera entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

/// Player's facing direction in radians. 0 = +Z (downfield), PI/2 = +X.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FacingDirection(pub f32);

/// AI movement command — written by AI systems, consumed by player_movement.
#[derive(Component, Debug, Clone, Copy)]
pub struct AiMoveTarget {
    pub target: Vec3,
    pub sprint: bool,
}

impl Default for AiMoveTarget {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            sprint: false,
        }
    }
}

/// Tracks which player a defender is assigned to guard (man defense).
#[derive(Component, Debug, Clone, Copy)]
pub struct DefenseAssignment {
    pub marking: PlayerId,
}

/// Tracks the last thrower for assist/completion tracking.
#[derive(Component, Debug, Clone, Copy)]
pub struct LastThrower {
    pub player: PlayerId,
}
