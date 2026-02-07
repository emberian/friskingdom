use bevy::prelude::*;
use fk_core::components::*;
use fk_core::resources::GameInput;

const BASE_SPEED: f32 = 5.0; // m/s walk
const SPRINT_MULTIPLIER: f32 = 1.5;
const ACCELERATION: f32 = 20.0; // m/s^2
const DECELERATION: f32 = 15.0; // m/s^2
const STAMINA_DRAIN: f32 = 15.0; // per second while sprinting
const FIELD_MIN_X: f32 = -18.5; // field half-width (37m / 2)
const FIELD_MAX_X: f32 = 18.5;
const FIELD_MIN_Z: f32 = -3.0; // small margin behind end line
const FIELD_MAX_Z: f32 = 103.0; // small margin beyond far end line

pub fn player_movement_system(
    input: Res<GameInput>,
    time: Res<Time>,
    mut players: Query<(
        &mut Position,
        &mut Velocity,
        &mut FacingDirection,
        &mut Stamina,
        &PlayerStats,
        Option<&Controlled>,
        Option<&HasDisc>,
        Option<&AiMoveTarget>,
    )>,
) {
    let dt = time.delta_secs();

    for (mut pos, mut vel, mut facing, mut stamina, stats, controlled, has_disc, ai_target) in
        &mut players
    {
        let max_speed = BASE_SPEED * (0.7 + 0.3 * stats.speed);

        if controlled.is_some() {
            // Human-controlled player
            let move_input = input.move_dir;

            // Convert 2D input to 3D movement (X = input.x, Z = input.y since field is along Z)
            let desired_vel = if move_input.length_squared() > 0.01 {
                let speed = if input.sprint && !stamina.exhausted && has_disc.is_none() {
                    max_speed * SPRINT_MULTIPLIER
                } else {
                    max_speed
                };
                Vec3::new(move_input.x, 0.0, move_input.y) * speed
            } else {
                Vec3::ZERO
            };

            accelerate_toward(&mut vel.0, desired_vel, stats.acceleration, dt);

            // Sprint stamina drain
            if input.sprint && !stamina.exhausted && vel.0.length() > BASE_SPEED {
                stamina.current -= STAMINA_DRAIN * dt;
                if stamina.current <= 0.0 {
                    stamina.current = 0.0;
                    stamina.exhausted = true;
                }
            } else {
                regen_stamina(&mut stamina, dt);
            }

            // Has disc = can't run, only pivot
            if has_disc.is_some() {
                vel.0 = Vec3::ZERO;
                // Update facing from input direction (pivot in place)
                if move_input.length_squared() > 0.01 {
                    facing.0 = move_input.x.atan2(move_input.y);
                }
            } else {
                // Update facing from movement direction
                if vel.0.length_squared() > 0.1 {
                    facing.0 = vel.0.x.atan2(vel.0.z);
                }
            }
        } else if let Some(ai) = ai_target {
            // AI-controlled player: move toward AI target
            let to_target = ai.target - pos.0;
            let dist = to_target.length();

            if dist > 0.5 {
                let speed = if ai.sprint && !stamina.exhausted {
                    max_speed * SPRINT_MULTIPLIER
                } else {
                    max_speed
                };
                let dir = to_target / dist;
                let desired_vel = dir * speed.min(dist * 4.0); // slow down near target
                accelerate_toward(&mut vel.0, desired_vel, stats.acceleration, dt);

                // Stamina for AI sprint
                if ai.sprint && !stamina.exhausted && vel.0.length() > BASE_SPEED {
                    stamina.current -= STAMINA_DRAIN * dt;
                    if stamina.current <= 0.0 {
                        stamina.current = 0.0;
                        stamina.exhausted = true;
                    }
                } else {
                    regen_stamina(&mut stamina, dt);
                }
            } else {
                // Close enough to target, decelerate
                accelerate_toward(&mut vel.0, Vec3::ZERO, stats.acceleration, dt);
                regen_stamina(&mut stamina, dt);
            }

            // Update facing
            if vel.0.length_squared() > 0.1 {
                facing.0 = vel.0.z.atan2(vel.0.x);
            }
        } else {
            // No AI target and not controlled: stand still, regen
            accelerate_toward(&mut vel.0, Vec3::ZERO, stats.acceleration, dt);
            regen_stamina(&mut stamina, dt);
        }

        // Apply velocity to position
        pos.0 += vel.0 * dt;

        // Clamp to field bounds
        pos.0.x = pos.0.x.clamp(FIELD_MIN_X, FIELD_MAX_X);
        pos.0.z = pos.0.z.clamp(FIELD_MIN_Z, FIELD_MAX_Z);
        /*ROBOTODO: jumping — need vertical velocity, gravity, landing detection.
          PlayerStats.jumping should affect jump height. Layout bids also need
          vertical movement. See design-docs/04-player-controls.md */
        pos.0.y = 0.0;
    }
}

fn accelerate_toward(vel: &mut Vec3, desired: Vec3, accel_stat: f32, dt: f32) {
    let diff = desired - *vel;
    let accel = if desired.length_squared() > vel.length_squared() {
        ACCELERATION
    } else {
        DECELERATION
    };
    let accel_factor = accel * (0.5 + 0.5 * accel_stat);

    if diff.length() > accel_factor * dt {
        *vel += diff.normalize() * accel_factor * dt;
    } else {
        *vel = desired;
    }
}

fn regen_stamina(stamina: &mut Stamina, dt: f32) {
    stamina.current = (stamina.current + stamina.regen_rate * dt).min(stamina.max);
    if stamina.current > stamina.max * 0.2 {
        stamina.exhausted = false;
    }
}
