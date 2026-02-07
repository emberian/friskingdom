use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use fk_core::components::*;
use fk_core::resources::*;
use fk_core::states::*;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct StateSnapshot {
    frame: u32,
    game_phase: String,
    scoreboard: ScoreboardDto,
    clock: ClockDto,
    stall: StallDto,
    disc: DiscDto,
    players: Vec<PlayerDto>,
}

#[derive(Serialize)]
struct ScoreboardDto {
    home: u32,
    away: u32,
    points_to_win: u32,
}

#[derive(Serialize)]
struct ClockDto {
    match_time: f64,
    point_time: f64,
    half: u8,
    point_number: u32,
}

#[derive(Serialize)]
struct StallDto {
    count: f32,
    active: bool,
}

#[derive(Serialize)]
struct DiscDto {
    position: [f32; 3],
    velocity: [f32; 3],
    in_flight: bool,
    grounded: bool,
    held_by: Option<u32>,
}

#[derive(Serialize)]
struct PlayerDto {
    id: u32,
    name: String,
    team: String,
    position: [f32; 3],
    velocity: [f32; 3],
    role: String,
    on_offense: bool,
    has_disc: bool,
    controlled: bool,
    ai_controlled: bool,
    stamina: f32,
    stamina_max: f32,
    facing: f32,
}

pub fn state_dump_system(
    frame_count: Res<FrameCount>,
    scoreboard: Res<Scoreboard>,
    clock: Res<GameClock>,
    stall: Res<StallCount>,
    game_phase: Res<State<GamePhase>>,
    disc_query: Query<(&Position, &Velocity, &DiscPhysicsState), With<Disc>>,
    player_query: Query<
        (
            &Player,
            &TeamMember,
            &Position,
            &Velocity,
            &PlayerRole,
            &FacingDirection,
            &Stamina,
            Option<&OnOffense>,
            Option<&HasDisc>,
            Option<&Controlled>,
            Option<&AiControlled>,
        ),
        Without<Disc>,
    >,
) {
    // Only dump every 60 frames (1 second at 60Hz)
    if frame_count.0 % 60 != 0 {
        return;
    }

    let phase_str = format!("{:?}", game_phase.get());

    let disc = disc_query
        .iter()
        .next()
        .map(|(pos, vel, state)| DiscDto {
            position: [pos.0.x, pos.0.y, pos.0.z],
            velocity: [vel.0.x, vel.0.y, vel.0.z],
            in_flight: state.in_flight,
            grounded: state.grounded,
            held_by: state.held_by.map(|id| id.0),
        })
        .unwrap_or(DiscDto {
            position: [0.0; 3],
            velocity: [0.0; 3],
            in_flight: false,
            grounded: false,
            held_by: None,
        });

    let mut players = Vec::new();
    for (player, team, pos, vel, role, facing, stamina, on_offense, has_disc, controlled, ai_controlled) in &player_query {
        players.push(PlayerDto {
            id: player.id.0,
            name: player.name.clone(),
            team: format!("{:?}", team.team),
            position: [pos.0.x, pos.0.y, pos.0.z],
            velocity: [vel.0.x, vel.0.y, vel.0.z],
            role: format!("{:?}", role),
            on_offense: on_offense.is_some(),
            has_disc: has_disc.is_some(),
            controlled: controlled.is_some(),
            ai_controlled: ai_controlled.is_some(),
            stamina: stamina.current,
            stamina_max: stamina.max,
            facing: facing.0,
        });
    }
    // Sort by ID for stable output
    players.sort_by_key(|p| p.id);

    let snapshot = StateSnapshot {
        frame: frame_count.0,
        game_phase: phase_str,
        scoreboard: ScoreboardDto {
            home: scoreboard.home_score,
            away: scoreboard.away_score,
            points_to_win: scoreboard.points_to_win,
        },
        clock: ClockDto {
            match_time: clock.match_time,
            point_time: clock.point_time,
            half: clock.half,
            point_number: clock.point_number,
        },
        stall: StallDto {
            count: stall.count,
            active: stall.active,
        },
        disc,
        players,
    };

    // Atomic write: write to temp file, then rename
    let tmp_path = "/tmp/friskingdom-debug/state.json.tmp";
    let final_path = "/tmp/friskingdom-debug/state.json";
    if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
        if let Ok(mut file) = std::fs::File::create(tmp_path) {
            if file.write_all(json.as_bytes()).is_ok() {
                let _ = std::fs::rename(tmp_path, final_path);
            }
        }
    }
}
