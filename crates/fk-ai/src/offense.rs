use bevy::prelude::*;
use fk_core::components::*;
use fk_core::types::*;
use rand::Rng;

// ---------------------------------------------------------------------------
// Offense AI state component
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffensePhase {
    /// At home position, waiting for the right time to cut.
    Holding,
    /// Actively making a cut (in-cut or out-cut).
    Cutting,
    /// Clearing out to the side after a cut to reset.
    Clearing,
}

#[derive(Component, Debug, Clone)]
pub struct OffenseAiState {
    pub phase: OffensePhase,
    /// Countdown timer for phase transitions (seconds).
    pub timer: f32,
    /// The world-space position this player is cutting/clearing toward.
    pub cut_target: Vec3,
}

impl Default for OffenseAiState {
    fn default() -> Self {
        Self {
            phase: OffensePhase::Holding,
            timer: 2.0,
            cut_target: Vec3::ZERO,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns +1.0 when the team attacks toward z=100 (Home on offense),
/// -1.0 when attacking toward z=0 (Away on offense).
fn attack_direction(team: Team) -> f32 {
    match team {
        Team::Home => 1.0,
        Team::Away => -1.0,
    }
}

/// Clamp a position inside the playing rectangle.
fn clamp_to_field(mut p: Vec3) -> Vec3 {
    p.x = p.x.clamp(-17.0, 17.0);
    p.z = p.z.clamp(2.0, 98.0);
    p.y = 0.0;
    p
}

// ---------------------------------------------------------------------------
// System: receiving formation during Pull phase
// ---------------------------------------------------------------------------

pub fn offense_pull_formation_system(
    disc_q: Query<&Position, With<Disc>>,
    mut ai_players: Query<
        (&Position, &TeamMember, &PlayerRole, &mut AiMoveTarget),
        (With<AiControlled>, With<OnOffense>),
    >,
) {
    if disc_q.iter().next().is_none() {
        return;
    }

    let mut index = 0usize;
    for (_pos, team_member, role, mut ai_move) in &mut ai_players {
        let dir = attack_direction(team_member.team);
        // Receiving end zone: Home offense receives near z=9, Away offense near z=91
        let base_z = if dir > 0.0 { 15.0 } else { 85.0 };

        let target = match role {
            PlayerRole::Handler => {
                // Handlers slightly behind, spread across width
                let x = -8.0 + (index as f32 % 3.0) * 8.0;
                Vec3::new(x, 0.0, base_z - dir * 3.0)
            }
            PlayerRole::Cutter | PlayerRole::Hybrid => {
                // Cutters slightly ahead, spread across width
                let x = -10.0 + (index as f32 % 4.0) * 7.0;
                Vec3::new(x, 0.0, base_z + dir * 5.0)
            }
        };

        ai_move.target = clamp_to_field(target);
        ai_move.sprint = false;
        index += 1;
    }
}

// ---------------------------------------------------------------------------
// System: ensure every AI offensive player has an OffenseAiState
// ---------------------------------------------------------------------------

pub fn ensure_offense_ai_state(
    mut commands: Commands,
    query: Query<Entity, (With<AiControlled>, With<OnOffense>, Without<OffenseAiState>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(OffenseAiState::default());
    }
}

// ---------------------------------------------------------------------------
// Main offense system
// ---------------------------------------------------------------------------

pub fn offense_ai_system(
    time: Res<Time>,
    disc_q: Query<&Position, With<Disc>>,
    _disc_state_q: Query<&DiscPhysicsState, With<Disc>>,
    thrower_q: Query<(&Position, &TeamMember), With<HasDisc>>,
    mut ai_players: Query<
        (
            Entity,
            &Player,
            &Position,
            &TeamMember,
            &PlayerRole,
            &mut AiMoveTarget,
            &mut OffenseAiState,
        ),
        (With<AiControlled>, With<OnOffense>, Without<HasDisc>),
    >,
) {
    let dt = time.delta_secs();

    // Find the disc position.
    let disc_pos = match disc_q.iter().next() {
        Some(p) => p.0,
        None => return,
    };

    // Find out who holds the disc (if anyone) to determine downfield direction.
    let _holder_team = thrower_q.iter().next().map(|(_, tm)| tm.team);

    // Collect player entity list for index-based spreading.
    let player_ids: Vec<Entity> = ai_players.iter().map(|(e, ..)| e).collect();

    for (entity, _player, pos, team_member, role, mut ai_move, mut ai_state) in &mut ai_players {
        let dir = attack_direction(team_member.team);
        let player_index = player_ids.iter().position(|&e| e == entity).unwrap_or(0);

        match role {
            PlayerRole::Handler => {
                handler_ai(
                    &mut ai_move,
                    pos.0,
                    disc_pos,
                    dir,
                    player_index,
                    &player_ids,
                );
            }
            PlayerRole::Cutter | PlayerRole::Hybrid => {
                cutter_ai(
                    &mut ai_move,
                    &mut ai_state,
                    pos.0,
                    disc_pos,
                    dir,
                    player_index,
                    &player_ids,
                    dt,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Handler AI
// ---------------------------------------------------------------------------

fn handler_ai(
    ai_move: &mut AiMoveTarget,
    _pos: Vec3,
    disc_pos: Vec3,
    attack_dir: f32,
    index: usize,
    _all_players: &[Entity],
) {
    let handler_count = 2.max(1); // at least pretend there are 2 handler slots
    let handler_slot = index % handler_count;

    let target = if handler_slot == 0 {
        // Primary handler / dump: position behind and to the side of the disc
        let behind_offset = -attack_dir * 7.0;
        let side_offset = if index % 2 == 0 { -6.0 } else { 6.0 };
        Vec3::new(
            (disc_pos.x + side_offset).clamp(-15.0, 15.0),
            0.0,
            disc_pos.z + behind_offset,
        )
    } else {
        // Secondary handler: position slightly downfield and to the opposite side
        let forward_offset = attack_dir * 3.0;
        let side_offset = if index % 2 == 0 { 5.0 } else { -5.0 };
        Vec3::new(
            (disc_pos.x + side_offset).clamp(-15.0, 15.0),
            0.0,
            disc_pos.z + forward_offset,
        )
    };

    ai_move.target = clamp_to_field(target);
    ai_move.sprint = false; // handlers walk into position
}

// ---------------------------------------------------------------------------
// Cutter AI
// ---------------------------------------------------------------------------

fn cutter_ai(
    ai_move: &mut AiMoveTarget,
    state: &mut OffenseAiState,
    pos: Vec3,
    disc_pos: Vec3,
    attack_dir: f32,
    index: usize,
    all_players: &[Entity],
    dt: f32,
) {
    let mut rng = rand::thread_rng();

    state.timer -= dt;

    match state.phase {
        OffensePhase::Holding => {
            // Stand at a home/reset position spread across the width, at moderate depth.
            let spread_count = all_players.len().max(1) as f32;
            let spread_x = -12.0 + (index as f32 / spread_count) * 24.0;
            let depth = disc_pos.z + attack_dir * 15.0;

            let home = clamp_to_field(Vec3::new(spread_x, 0.0, depth));
            ai_move.target = home;
            ai_move.sprint = false;

            // When timer expires, start a cut.
            if state.timer <= 0.0 {
                let is_in_cut = rng.gen_bool(0.5);
                if is_in_cut {
                    // In-cut: run toward the disc/handler area.
                    let side_offset = rng.gen_range(-8.0..8.0);
                    let forward = rng.gen_range(5.0..10.0);
                    state.cut_target = clamp_to_field(Vec3::new(
                        disc_pos.x + side_offset,
                        0.0,
                        disc_pos.z + attack_dir * forward,
                    ));
                } else {
                    // Out-cut / deep cut: run toward the attacking end zone.
                    let side_offset = rng.gen_range(-6.0..6.0);
                    let deep = rng.gen_range(15.0..25.0);
                    state.cut_target = clamp_to_field(Vec3::new(
                        disc_pos.x + side_offset,
                        0.0,
                        disc_pos.z + attack_dir * deep,
                    ));
                }
                state.phase = OffensePhase::Cutting;
                state.timer = rng.gen_range(1.5..3.0); // cut duration
            }
        }

        OffensePhase::Cutting => {
            ai_move.target = state.cut_target;
            ai_move.sprint = true;

            let dist_to_target = (pos - state.cut_target).length();

            // Transition to clearing when close to target or timer expires.
            if dist_to_target < 2.0 || state.timer <= 0.0 {
                // Clear to the side of the field.
                let clear_side = if pos.x > 0.0 { 14.0 } else { -14.0 };
                let clear_depth = disc_pos.z + attack_dir * 10.0;
                state.cut_target = clamp_to_field(Vec3::new(clear_side, 0.0, clear_depth));
                state.phase = OffensePhase::Clearing;
                state.timer = rng.gen_range(2.0..4.0);
            }
        }

        OffensePhase::Clearing => {
            ai_move.target = state.cut_target;
            ai_move.sprint = false;

            let dist_to_target = (pos - state.cut_target).length();

            // Return to holding once we reach the clear spot or timer expires.
            if dist_to_target < 2.0 || state.timer <= 0.0 {
                state.phase = OffensePhase::Holding;
                state.timer = rng.gen_range(2.0..4.0);
            }
        }
    }
}
