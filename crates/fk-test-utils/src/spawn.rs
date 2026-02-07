use bevy::prelude::*;
use fk_core::components::*;
use fk_core::types::*;
use frisbee_physics::{DiscState, Euler, PhysDVec3};

/// Spawn 14 headless players (no rendering components) arranged in two teams.
///
/// Home team: PlayerId(0)..PlayerId(6) at z=9, spread x=-12..+12 (step 4.0).
/// Away team: PlayerId(7)..PlayerId(13) at z=91, same x spread.
///
/// Uses fixed stats for determinism (no RNG). Players with id%7 < 3 are handlers,
/// the rest are cutters.
///
/// Home team starts on offense, Away team on defense.
/// PlayerId(0) gets `Controlled`; all others get `AiControlled`.
pub fn spawn_headless_teams(mut commands: Commands) {
    let handler_stats = PlayerStats {
        speed: 0.55,
        acceleration: 0.55,
        throwing_power: 0.82,
        throwing_accuracy: 0.82,
        catching: 0.72,
        agility: 0.62,
        jumping: 0.47,
        endurance: 0.65,
        disc_iq: 0.82,
    };

    let cutter_stats = PlayerStats {
        speed: 0.82,
        acceleration: 0.77,
        throwing_power: 0.50,
        throwing_accuracy: 0.50,
        catching: 0.77,
        agility: 0.72,
        jumping: 0.70,
        endurance: 0.65,
        disc_iq: 0.55,
    };

    for i in 0u32..14 {
        let team = if i < 7 { Team::Home } else { Team::Away };
        let local_idx = (i % 7) as usize;
        let is_handler = local_idx < 3;

        let base_z: f32 = if team == Team::Home { 9.0 } else { 91.0 };
        let spread_x: f32 = -12.0 + (local_idx as f32) * 4.0;
        let pos = Vec3::new(spread_x, 0.0, base_z);

        let stats = if is_handler {
            handler_stats
        } else {
            cutter_stats
        };

        let role = if is_handler {
            PlayerRole::Handler
        } else {
            PlayerRole::Cutter
        };

        let name = format!(
            "{} {}",
            if team == Team::Home { "Home" } else { "Away" },
            local_idx
        );

        let mut entity_commands = commands.spawn((
            Player {
                id: PlayerId(i),
                name,
                handedness: Handedness::Right,
            },
            TeamMember { team },
            Position(pos),
            Velocity(Vec3::ZERO),
            stats,
            Stamina::new(0.7),
            SwagMeter::default(),
            role,
            FacingDirection(0.0),
            AiMoveTarget::default(),
        ));

        // Offense / defense markers
        if team == Team::Home {
            entity_commands.insert(OnOffense);
        } else {
            entity_commands.insert(OnDefense);
        }

        // Controlled vs AiControlled
        if i == 0 {
            entity_commands.insert(Controlled);
        } else {
            entity_commands.insert(AiControlled);
        }
    }
}

/// Spawn a single headless disc entity (no rendering components).
///
/// The disc starts at position (0, 1, 20), not in flight, not grounded,
/// held by PlayerId(0).
pub fn spawn_headless_disc(mut commands: Commands) {
    let disc_state = DiscState {
        position: PhysDVec3::new(0.0, 1.0, 20.0),
        velocity: PhysDVec3::ZERO,
        orientation: Euler::new(0.0, 0.0, 0.0),
        angular_velocity: PhysDVec3::ZERO,
        spin_rate: 0.0,
        time: 0.0,
    };

    commands.spawn((
        Disc,
        Position(Vec3::new(0.0, 1.0, 20.0)),
        Velocity(Vec3::ZERO),
        DiscPhysicsState {
            state: disc_state,
            in_flight: false,
            grounded: false,
            held_by: Some(PlayerId(0)),
        },
    ));
}
