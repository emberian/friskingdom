use bevy::prelude::*;
use fk_core::components::*;
use fk_core::events::*;
use fk_core::resources::GameInput;
use frisbee_physics::{DiscParams, NullWind, Simulator};
use glam::DVec3;

/// Resource holding the physics simulator.
#[derive(Resource)]
pub struct PhysicsSimulator {
    pub simulator: Simulator,
}

impl Default for PhysicsSimulator {
    fn default() -> Self {
        Self {
            simulator: Simulator::new(DiscParams::ultrastar()),
        }
    }
}

/// Step disc physics each fixed update (4 substeps at 240Hz).
pub fn disc_physics_system(
    simulator: Res<PhysicsSimulator>,
    mut disc_query: Query<(&mut DiscPhysicsState, &mut Position, &mut Velocity), With<Disc>>,
) {
    /*ROBOTODO: replace NullWind with a MatchWindField resource that implements
      WindField trait. Should be created from MatchConfig.wind_enabled + random
      wind direction/speed at match start. See design-docs/02-wind-field.md */
    let wind = NullWind;

    for (mut disc, mut pos, mut vel) in &mut disc_query {
        if !disc.in_flight {
            continue;
        }

        // 4 substeps per game tick (240Hz physics at 60Hz game tick)
        let mut state = disc.state;
        for _ in 0..4 {
            state = simulator.simulator.step(&state, &wind);
        }

        // Ground collision
        if state.position.y <= 0.0 {
            state.position.y = 0.0;
            disc.in_flight = false;
            disc.grounded = true;
            disc.held_by = None;
        }

        disc.state = state;
        pos.0 = Vec3::new(
            state.position.x as f32,
            state.position.y as f32,
            state.position.z as f32,
        );
        vel.0 = Vec3::new(
            state.velocity.x as f32,
            state.velocity.y as f32,
            state.velocity.z as f32,
        );
    }
}

/// Keep disc attached to holding player.
pub fn disc_holder_system(
    players: Query<(&Player, &Position, &FacingDirection), Without<Disc>>,
    mut disc_query: Query<(&mut Position, &DiscPhysicsState), With<Disc>>,
) {
    for (mut disc_pos, disc_state) in &mut disc_query {
        if let Some(holder_id) = disc_state.held_by {
            for (player, player_pos, facing) in &players {
                if player.id == holder_id {
                    // Disc floats at player's hand height, offset in facing direction
                    let (sin_f, cos_f) = facing.0.sin_cos();
                    let hand_offset = Vec3::new(sin_f * 0.3, 1.2, cos_f * 0.3);
                    disc_pos.0 = player_pos.0 + hand_offset;
                    break;
                }
            }
        }
    }
}

/// Handle throw input.
pub fn throw_system(
    input: Res<GameInput>,
    mut disc_query: Query<(&mut DiscPhysicsState, &Position), With<Disc>>,
    controlled_query: Query<(&Player, &Position, &PlayerStats, &FacingDirection), With<Controlled>>,
    mut throw_events: MessageWriter<DiscThrown>,
) {
    if !input.throw_released {
        return;
    }

    let throw_type = input
        .throw_type
        .unwrap_or(frisbee_physics::ThrowType::Backhand);

    for (mut disc, disc_pos) in &mut disc_query {
        if disc.held_by.is_none() || disc.in_flight {
            continue;
        }

        let holder_id = disc.held_by.unwrap();

        // Find the controlled player who should be the holder
        for (player, _player_pos, _stats, player_facing) in &controlled_query {
            if player.id != holder_id {
                continue;
            }

            // Create throw state from input
            let mods = frisbee_physics::ThrowModifications {
                power: input.throw_power as f64,
                aim_angle: input.aim_dir.x as f64 * 0.5, // scale aim input
                hyzer_adjust: 0.0,
                nose_adjust: input.aim_dir.y as f64 * 0.1,
            };

            // Use the player's actual facing direction
            let facing = player_facing.0 as f64;

            let throw_state = frisbee_physics::create_throw_state(
                throw_type,
                &mods,
                DVec3::new(
                    disc_pos.0.x as f64,
                    disc_pos.0.y as f64,
                    disc_pos.0.z as f64,
                ),
                facing,
            );

            disc.state = throw_state;
            disc.in_flight = true;
            disc.grounded = false;
            disc.held_by = None;

            throw_events.write(DiscThrown {
                thrower: player.id,
                throw_type,
            });

            break;
        }
    }
}

/// Detect when a disc in flight crosses the field boundaries and stop it.
///
/// Field runs along Z from 0..100 with sidelines at X = -18.5..+18.5.
/// When the disc crosses any boundary the position is clamped to the nearest
/// edge, the disc is grounded, and a `DiscOutOfBounds` message is emitted so
/// that `turnover_system` can handle the possession change.
pub fn out_of_bounds_system(
    mut disc_query: Query<(&mut DiscPhysicsState, &mut Position), With<Disc>>,
    mut oob_events: MessageWriter<DiscOutOfBounds>,
) {
    const SIDELINE_MIN: f32 = -18.5;
    const SIDELINE_MAX: f32 = 18.5;
    const ENDLINE_MIN: f32 = 0.0;
    const ENDLINE_MAX: f32 = 100.0;

    for (mut disc, mut pos) in &mut disc_query {
        if !disc.in_flight {
            continue;
        }

        let p = pos.0;

        let out = p.x < SIDELINE_MIN
            || p.x > SIDELINE_MAX
            || p.z < ENDLINE_MIN
            || p.z > ENDLINE_MAX;

        if !out {
            continue;
        }

        // Clamp to the nearest boundary point
        let clamped = Vec3::new(
            p.x.clamp(SIDELINE_MIN, SIDELINE_MAX),
            p.y,
            p.z.clamp(ENDLINE_MIN, ENDLINE_MAX),
        );

        pos.0 = clamped;
        disc.in_flight = false;
        disc.grounded = true;
        disc.held_by = None;

        oob_events.write(DiscOutOfBounds {
            crossing_point: clamped,
        });
    }
}

/// Basic catch detection.
pub fn catch_system(
    mut disc_query: Query<(&mut DiscPhysicsState, &Position, Option<&LastThrower>), With<Disc>>,
    players: Query<(&Player, &Position, &PlayerStats), Without<Disc>>,
    mut catch_events: MessageWriter<DiscCaught>,
    mut _drop_events: MessageWriter<DiscDropped>,
) {
    for (mut disc, disc_pos, last_thrower) in &mut disc_query {
        if !disc.in_flight {
            continue;
        }

        // Only check catches when disc is below reasonable catch height (3m)
        if disc_pos.0.y > 3.0 {
            continue;
        }

        const CATCH_RADIUS: f32 = 1.0; // meters

        for (player, player_pos, _stats) in &players {
            // Prevent the thrower from immediately catching their own throw
            if let Some(lt) = last_thrower {
                if lt.player == player.id {
                    continue;
                }
            }

            let dist = (player_pos.0 - disc_pos.0).length();
            if dist < CATCH_RADIUS {
                /*ROBOTODO: catch probability based on PlayerStats.catching,
                  disc speed, angle of arrival, layout vs standing,
                  and defender proximity. See design-docs/03-ultimate-rules-gameplay.md */
                /*ROBOTODO: team awareness — defense catching should trigger
                  interception turnover (Callahan if in end zone). Currently any
                  player catches regardless of team. See design-docs/03-ultimate-rules-gameplay.md */
                disc.in_flight = false;
                disc.grounded = false;
                disc.held_by = Some(player.id);

                catch_events.write(DiscCaught {
                    catcher: player.id,
                    position: disc_pos.0,
                    was_layout: false,
                });

                break;
            }
        }
    }
}
