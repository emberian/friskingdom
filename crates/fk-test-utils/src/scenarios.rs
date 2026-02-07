use bevy::prelude::*;
use fk_core::components::*;
use fk_core::states::*;
use fk_core::types::*;

use crate::app::tick;
use crate::assertions::*;

/// Advance app through Loading -> InGame -> PrePoint -> Pull.
///
/// Returns once the game is in the `Pull` phase.
/// Panics if the phase is not reached within 10 updates.
pub fn advance_to_pull(app: &mut App) {
    for _ in 0..10 {
        app.update();
        if get_game_phase(app) == GamePhase::Pull {
            return;
        }
    }
    panic!("Failed to reach Pull phase after 10 updates");
}

/// Execute a pull throw by directly launching the disc into flight, then tick
/// until it resolves (lands or is caught) and the phase transitions.
///
/// This bypasses the input system entirely, which resets `throw_released` every
/// frame before `pull_system` can see it in headless tests.
///
/// Panics if the disc never resolves after 600 ticks.
pub fn execute_pull(app: &mut App) {
    use frisbee_physics::PhysDVec3;

    // Directly launch the disc into flight toward the opposite end zone
    {
        let world = app.world_mut();
        let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position)>();
        for (mut disc, pos) in disc_query.iter_mut(world) {
            let holder_pos = pos.0;

            // Create a simple throw state heading downfield
            let mods = frisbee_physics::ThrowModifications {
                power: 0.8,
                aim_angle: 0.0,
                hyzer_adjust: 0.0,
                nose_adjust: 0.05,
            };
            let disc_pos = PhysDVec3::new(
                holder_pos.x as f64,
                1.2,
                holder_pos.z as f64,
            );
            // Throw toward +Z (Home attacks toward Away end zone)
            let throw_state = frisbee_physics::create_throw_state(
                frisbee_physics::ThrowType::Backhand,
                &mods,
                disc_pos,
                0.0, // facing +Z
            );
            disc.state = throw_state;
            disc.in_flight = true;
            disc.grounded = false;
            disc.held_by = None;
        }
    }

    // Also remove HasDisc from all players
    {
        let world = app.world_mut();
        let mut has_disc_query = world.query::<(Entity, &HasDisc)>();
        let entities: Vec<Entity> = has_disc_query.iter(world).map(|(e, _)| e).collect();
        for entity in entities {
            world.entity_mut(entity).remove::<HasDisc>();
        }
    }

    // Tick until disc resolves
    for _ in 0..600 {
        tick(app, 1);
        let disc = get_disc_state(app);
        if !disc.in_flight {
            // Give a few more ticks for state transition to propagate
            tick(app, 3);
            return;
        }
    }
    panic!("Pull disc never resolved after 600 ticks");
}


/// Teleport a player to a specific position.
pub fn teleport_player(app: &mut App, id: PlayerId, pos: Vec3) {
    let world = app.world_mut();
    let mut query = world.query::<(&Player, &mut Position)>();
    for (player, mut position) in query.iter_mut(world) {
        if player.id == id {
            position.0 = pos;
            return;
        }
    }
}

/// Force the disc to be on the ground at a specific position with no holder.
pub fn force_disc_ground(app: &mut App, pos: Vec3) {
    let world = app.world_mut();
    let mut query = world.query::<(&mut DiscPhysicsState, &mut Position)>();
    for (mut disc, mut disc_pos) in query.iter_mut(world) {
        disc.in_flight = false;
        disc.grounded = true;
        disc.held_by = None;
        disc_pos.0 = pos;
    }
}

/// Force the disc to be held by a specific player (not in flight, not grounded).
pub fn force_disc_to_player(app: &mut App, holder_id: PlayerId) {
    let world = app.world_mut();
    let mut query = world.query::<&mut DiscPhysicsState>();
    for mut disc in query.iter_mut(world) {
        disc.in_flight = false;
        disc.grounded = false;
        disc.held_by = Some(holder_id);
    }
}
