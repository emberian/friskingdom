use bevy::prelude::*;
use fk_core::components::*;
use fk_core::events::*;
use fk_core::resources::StallCount;
use fk_core::types::*;
use frisbee_physics::ThrowType;
use glam::DVec3;
use rand::Rng;

// ---------------------------------------------------------------------------
// AI pull system — runs only during GamePhase::Pull
// ---------------------------------------------------------------------------

pub fn ai_pull_system(
    time: Res<Time>,
    mut commands: Commands,
    mut pullers: Query<
        (
            Entity,
            &Player,
            &Position,
            &TeamMember,
            &mut FacingDirection,
            &mut AiThrowState,
        ),
        (With<AiControlled>, With<HasDisc>, With<OnDefense>),
    >,
    mut disc_q: Query<(Entity, &mut DiscPhysicsState, &Position), With<Disc>>,
    mut throw_events: MessageWriter<DiscThrown>,
) {
    let dt = time.delta_secs();

    for (puller_entity, player, puller_pos, team, mut facing, mut ai_state) in &mut pullers {
        ai_state.decision_timer -= dt;
        if ai_state.decision_timer > 0.0 {
            continue;
        }

        // Determine pull direction based on team.
        // Home defense pulls toward +Z (Away end zone), Away defense pulls toward -Z (Home end zone).
        let mut rng = rand::thread_rng();
        let target_z = match team.team {
            Team::Home => rng.gen_range(75.0..90.0),  // toward Away end zone
            Team::Away => rng.gen_range(10.0..25.0),  // toward Home end zone
        };

        let target_x = rng.gen_range(-5.0..5.0);
        let dx = target_x - puller_pos.0.x;
        let dz = target_z - puller_pos.0.z;
        let throw_facing = dx.atan2(dz);

        let power = rng.gen_range(0.75..0.95);
        let mods = frisbee_physics::ThrowModifications {
            power: power as f64,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.05,
        };

        let disc_pos_dvec = DVec3::new(
            puller_pos.0.x as f64,
            1.2,
            puller_pos.0.z as f64,
        );

        let throw_state = frisbee_physics::create_throw_state(
            ThrowType::Backhand,
            &mods,
            disc_pos_dvec,
            throw_facing as f64,
        );

        let Ok((disc_entity, mut disc, _)) = disc_q.single_mut() else {
            continue;
        };

        disc.state = throw_state;
        disc.in_flight = true;
        disc.grounded = false;
        disc.held_by = None;

        commands
            .entity(disc_entity)
            .insert(LastThrower { player: player.id });

        facing.0 = throw_facing;
        commands
            .entity(puller_entity)
            .remove::<HasDisc>()
            .remove::<AiThrowState>();

        throw_events.write(DiscThrown {
            thrower: player.id,
            throw_type: ThrowType::Backhand,
        });
    }
}

// ---------------------------------------------------------------------------
// AI throw state component
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct AiThrowState {
    /// Countdown before the AI releases the throw (simulates decision time).
    pub decision_timer: f32,
    /// The teammate we intend to throw to (set when timer expires).
    pub target_player: Option<PlayerId>,
    /// The throw type we'll use.
    pub throw_type: ThrowType,
}

// ---------------------------------------------------------------------------
// System: attach AiThrowState when an AI player gains HasDisc
// ---------------------------------------------------------------------------

pub fn ensure_ai_throw_state(
    mut commands: Commands,
    query: Query<Entity, (With<AiControlled>, With<HasDisc>, Without<AiThrowState>)>,
) {
    let mut rng = rand::thread_rng();
    for entity in &query {
        commands.entity(entity).insert(AiThrowState {
            decision_timer: rng.gen_range(0.8..2.0),
            target_player: None,
            throw_type: ThrowType::Backhand,
        });
    }
}

// ---------------------------------------------------------------------------
// Main AI throwing decision system
// ---------------------------------------------------------------------------

pub fn ai_throw_decision_system(
    time: Res<Time>,
    stall: Res<StallCount>,
    mut commands: Commands,
    mut throwers: Query<
        (
            Entity,
            &Player,
            &Position,
            &TeamMember,
            &mut FacingDirection,
            &mut AiThrowState,
        ),
        (With<AiControlled>, With<HasDisc>),
    >,
    teammates: Query<
        (&Player, &Position, &TeamMember),
        (With<OnOffense>, Without<HasDisc>, Without<Disc>),
    >,
    defenders: Query<&Position, With<OnDefense>>,
    mut disc_q: Query<(Entity, &mut DiscPhysicsState, &Position), With<Disc>>,
    mut throw_events: MessageWriter<DiscThrown>,
) {
    let dt = time.delta_secs();

    for (thrower_entity, player, thrower_pos, team, mut facing, mut ai_state) in &mut throwers {
        // Tick down the decision timer.
        ai_state.decision_timer -= dt;
        if ai_state.decision_timer > 0.0 {
            continue;
        }

        // --- Pick the best receiver ---

        let attack_dir = attack_direction(team.team);
        let stall_pressure = stall.count;

        let mut best_score = f32::NEG_INFINITY;
        let mut best_target: Option<(PlayerId, Vec3)> = None;

        for (tm_player, tm_pos, tm_team) in &teammates {
            if tm_team.team != team.team {
                continue;
            }
            // Don't throw to yourself (shouldn't happen with Without<HasDisc>, but be safe).
            if tm_player.id == player.id {
                continue;
            }

            let to_receiver = tm_pos.0 - thrower_pos.0;
            let dist = to_receiver.length();

            // Distance score: prefer 5-20m range.
            let dist_score = if dist < 5.0 {
                dist / 5.0 // 0..1
            } else if dist <= 20.0 {
                1.0 // sweet spot
            } else {
                (1.0 - (dist - 20.0) / 20.0).max(0.0)
            };

            // Openness: how far is the nearest defender from this receiver?
            let openness = defenders
                .iter()
                .map(|def_pos| (def_pos.0 - tm_pos.0).length())
                .fold(f32::INFINITY, f32::min);
            let open_score = (openness / 5.0).min(1.0); // 5m+ = fully open

            // Downfield bonus: receivers closer to the attacking end zone.
            let downfield = (tm_pos.0.z - thrower_pos.0.z) * attack_dir;
            let downfield_score = (downfield / 20.0).clamp(-0.5, 1.0);

            // Stall pressure: if stall > 6, boost dump (behind-disc) targets.
            let dump_bonus = if stall_pressure > 6.0 && downfield < 0.0 {
                (stall_pressure - 6.0) / 4.0 // up to 1.0 bonus at stall 10
            } else {
                0.0
            };

            let score = dist_score * 3.0 + open_score * 4.0 + downfield_score * 2.0 + dump_bonus * 5.0;

            if score > best_score {
                best_score = score;
                best_target = Some((tm_player.id, tm_pos.0));
            }
        }

        // If no valid target found, reset timer and try again soon.
        let Some((target_id, target_pos)) = best_target else {
            ai_state.decision_timer = 0.3;
            continue;
        };

        // --- Calculate the throw ---

        let dx = target_pos.x - thrower_pos.0.x;
        let dz = target_pos.z - thrower_pos.0.z;
        let throw_facing = dx.atan2(dz); // 0 = +Z convention
        let dist = (dx * dx + dz * dz).sqrt();

        // Choose throw type.
        let throw_type = choose_throw_type(throw_facing, facing.0, stall_pressure, dz * attack_dir);

        // Power scales with distance: 0.3 for short, up to 0.9 for 30m+.
        let power = (dist / 30.0).clamp(0.3, 0.9);

        let mods = frisbee_physics::ThrowModifications {
            power: power as f64,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        };

        let disc_pos_dvec = DVec3::new(
            thrower_pos.0.x as f64,
            1.2, // hand height
            thrower_pos.0.z as f64,
        );

        let throw_state =
            frisbee_physics::create_throw_state(throw_type, &mods, disc_pos_dvec, throw_facing as f64);

        // --- Update the disc and thrower ---

        let Ok((disc_entity, mut disc, _)) = disc_q.single_mut() else {
            continue;
        };

        disc.state = throw_state;
        disc.in_flight = true;
        disc.grounded = false;
        disc.held_by = None;

        // Update LastThrower on the disc entity.
        commands
            .entity(disc_entity)
            .insert(LastThrower { player: player.id });

        // Update the thrower: face the target, remove throw state & HasDisc.
        facing.0 = throw_facing;
        commands
            .entity(thrower_entity)
            .remove::<HasDisc>()
            .remove::<AiThrowState>();

        // Store decision for debugging / reference.
        ai_state.target_player = Some(target_id);
        ai_state.throw_type = throw_type;

        throw_events.write(DiscThrown {
            thrower: player.id,
            throw_type,
        });
    }
}

// ---------------------------------------------------------------------------
// Throw type selection helper
// ---------------------------------------------------------------------------

fn choose_throw_type(
    throw_angle: f32,
    current_facing: f32,
    stall: f32,
    downfield_component: f32,
) -> ThrowType {
    // Angle difference between current facing and throw direction.
    let mut angle_diff = throw_angle - current_facing;
    // Normalize to [-PI, PI].
    while angle_diff > std::f32::consts::PI {
        angle_diff -= 2.0 * std::f32::consts::PI;
    }
    while angle_diff < -std::f32::consts::PI {
        angle_diff += 2.0 * std::f32::consts::PI;
    }

    // Desperation hammer: high stall, target is behind.
    if stall > 8.0 && downfield_component < 0.0 {
        return ThrowType::Hammer;
    }

    let abs_diff = angle_diff.abs();

    if abs_diff < std::f32::consts::FRAC_PI_6 {
        // Within ~30 degrees of facing: backhand.
        ThrowType::Backhand
    } else if angle_diff > 0.0 {
        // Target is to the right (flick side for right-hander): forehand.
        ThrowType::Forehand
    } else {
        // Target is to the left but beyond backhand range: still backhand
        // (player would pivot). Could be forehand for a lefty, but keep it simple.
        ThrowType::Backhand
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn attack_direction(team: Team) -> f32 {
    match team {
        Team::Home => 1.0,
        Team::Away => -1.0,
    }
}
