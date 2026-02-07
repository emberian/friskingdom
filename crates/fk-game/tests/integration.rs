//! Integration contracts for `fk-game`.
//!
//! Active tests encode behavior that should hold today.
//! Ignored tests encode design contracts from `design-docs/` that will drive
//! upcoming implementation phases.

use bevy::ecs::message::Messages;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use fk_core::components::*;
use fk_core::events::*;
use fk_core::resources::*;
use fk_core::states::*;
use fk_core::types::*;
use fk_test_utils::app::{build_test_app, tick};
use fk_test_utils::assertions::*;
use fk_test_utils::scenarios::*;

const POSITION_EPS: f32 = 0.05;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn advance_to_liveplay(app: &mut App) {
    advance_to_pull(app);
    assert_eq!(get_game_phase(app), GamePhase::Pull);
    execute_pull(app);
    assert_eq!(get_game_phase(app), GamePhase::LivePlay);
}

fn force_has_disc_marker(app: &mut App, holder_id: PlayerId) {
    let world = app.world_mut();
    let mut q = world.query::<(Entity, &Player)>();
    let mut to_add = None;
    let mut to_remove = Vec::new();
    for (entity, player) in q.iter(world) {
        if player.id == holder_id {
            to_add = Some(entity);
        } else {
            to_remove.push(entity);
        }
    }
    for entity in to_remove {
        world.entity_mut(entity).remove::<HasDisc>();
    }
    if let Some(entity) = to_add {
        world.entity_mut(entity).insert(HasDisc);
    }
}

fn set_controlled_player(app: &mut App, target_id: PlayerId) {
    let world = app.world_mut();
    let mut q = world.query::<(Entity, &Player)>();
    let mut to_control = None;
    let mut others = Vec::new();

    for (entity, player) in q.iter(world) {
        if player.id == target_id {
            to_control = Some(entity);
        } else {
            others.push(entity);
        }
    }

    for entity in others {
        world
            .entity_mut(entity)
            .remove::<Controlled>()
            .insert(AiControlled);
    }
    if let Some(entity) = to_control {
        world
            .entity_mut(entity)
            .remove::<AiControlled>()
            .insert(Controlled);
    }
}

fn player_team(app: &mut App, id: PlayerId) -> Team {
    let world = app.world_mut();
    let mut q = world.query::<(&Player, &TeamMember)>();
    for (player, team) in q.iter(world) {
        if player.id == id {
            return team.team;
        }
    }
    panic!("Player {:?} not found", id);
}

fn current_offense_team(app: &mut App) -> Team {
    let world = app.world_mut();
    let mut q = world.query::<(&TeamMember, Option<&OnOffense>)>();
    for (team, on_offense) in q.iter(world) {
        if on_offense.is_some() {
            return team.team;
        }
    }
    panic!("No offensive team found");
}

fn drain_turnovers(app: &mut App) -> Vec<TurnoverOccurred> {
    let world = app.world_mut();
    let mut messages = world.resource_mut::<Messages<TurnoverOccurred>>();
    messages.drain().collect()
}

fn drain_oob_events(app: &mut App) -> Vec<DiscOutOfBounds> {
    let world = app.world_mut();
    let mut messages = world.resource_mut::<Messages<DiscOutOfBounds>>();
    messages.drain().collect()
}

fn wait_for_turnover(app: &mut App, max_ticks: usize) -> Option<TurnoverOccurred> {
    for _ in 0..max_ticks {
        tick(app, 1);
        let events = drain_turnovers(app);
        if let Some(event) = events.into_iter().last() {
            return Some(event);
        }
    }
    None
}

fn launch_disc_toward_sideline(app: &mut App) {
    use frisbee_physics::PhysDVec3;

    let world = app.world_mut();
    let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position)>();
    for (mut disc, mut pos) in disc_query.iter_mut(world) {
        let start = PhysDVec3::new(10.0, 1.5, 50.0);
        let mods = frisbee_physics::ThrowModifications {
            power: 0.95,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        };
        let throw_state = frisbee_physics::create_throw_state(
            frisbee_physics::ThrowType::Backhand,
            &mods,
            start,
            std::f64::consts::FRAC_PI_2, // +X (sideline)
        );
        disc.state = throw_state;
        disc.in_flight = true;
        disc.grounded = false;
        disc.held_by = None;
        pos.0 = Vec3::new(10.0, 1.5, 50.0);
    }

    let mut has_disc_query = world.query::<(Entity, &HasDisc)>();
    let entities: Vec<Entity> = has_disc_query.iter(world).map(|(e, _)| e).collect();
    for entity in entities {
        world.entity_mut(entity).remove::<HasDisc>();
    }
}

fn launch_disc_out_back_of_receiving_endzone_during_pull(app: &mut App) {
    let world = app.world_mut();
    let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position, &mut Velocity)>();
    for (mut disc, mut pos, mut vel) in disc_query.iter_mut(world) {
        disc.in_flight = true;
        disc.grounded = false;
        disc.held_by = None;
        pos.0 = Vec3::new(0.0, 1.0, -1.0); // Behind Home end line
        vel.0 = Vec3::new(0.0, 0.0, -12.0);
    }
}

fn set_midfield_flight(app: &mut App) {
    use frisbee_physics::PhysDVec3;

    let world = app.world_mut();
    let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position, &mut Velocity)>();
    for (mut disc, mut pos, mut vel) in disc_query.iter_mut(world) {
        let mods = frisbee_physics::ThrowModifications {
            power: 0.8,
            aim_angle: 0.0,
            hyzer_adjust: 0.0,
            nose_adjust: 0.0,
        };
        let throw_state = frisbee_physics::create_throw_state(
            frisbee_physics::ThrowType::Backhand,
            &mods,
            PhysDVec3::new(0.0, 1.5, 50.0),
            0.0, // +Z
        );
        disc.state = throw_state;
        disc.in_flight = true;
        disc.grounded = false;
        disc.held_by = None;
        pos.0 = Vec3::new(0.0, 1.5, 50.0);
        vel.0 = Vec3::ZERO;
    }

    let mut has_disc_query = world.query::<(Entity, &HasDisc)>();
    let entities: Vec<Entity> = has_disc_query.iter(world).map(|(e, _)| e).collect();
    for entity in entities {
        world.entity_mut(entity).remove::<HasDisc>();
    }
}

fn player_pose(app: &mut App, id: PlayerId) -> (Vec3, f32) {
    let world = app.world_mut();
    let mut q = world.query::<(&Player, &Position, &FacingDirection)>();
    for (player, pos, facing) in q.iter(world) {
        if player.id == id {
            return (pos.0, facing.0);
        }
    }
    panic!("Player {:?} not found", id);
}

fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

fn vec3_bits(v: Vec3) -> (u32, u32, u32) {
    (v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscSnapshot {
    in_flight: bool,
    grounded: bool,
    held_by: Option<u32>,
    position_bits: (u32, u32, u32),
    velocity_bits: (u32, u32, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerSnapshot {
    id: u32,
    team: Team,
    position_bits: (u32, u32, u32),
    velocity_bits: (u32, u32, u32),
    on_offense: bool,
    on_defense: bool,
    controlled: bool,
    has_disc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimulationSnapshot {
    phase: GamePhase,
    home_score: u32,
    away_score: u32,
    stall_bits: u32,
    disc: DiscSnapshot,
    players: Vec<PlayerSnapshot>,
}

fn snapshot_world(app: &mut App) -> SimulationSnapshot {
    let phase = get_game_phase(app);

    let (home_score, away_score, stall_bits, disc, players) = {
        let world = app.world_mut();
        let (home_score, away_score, stall_bits) = {
            let scoreboard = world.resource::<Scoreboard>();
            let stall = world.resource::<StallCount>();
            (
                scoreboard.home_score,
                scoreboard.away_score,
                stall.count.to_bits(),
            )
        };

        let mut disc_query =
            world.query::<(&DiscPhysicsState, &Position, &Velocity)>();
        let (disc_state, disc_pos, disc_vel) = disc_query
            .single(world)
            .expect("expected exactly one disc");

        let disc = DiscSnapshot {
            in_flight: disc_state.in_flight,
            grounded: disc_state.grounded,
            held_by: disc_state.held_by.map(|id| id.0),
            position_bits: vec3_bits(disc_pos.0),
            velocity_bits: vec3_bits(disc_vel.0),
        };

        let mut player_query = world.query::<(
            &Player,
            &TeamMember,
            &Position,
            &Velocity,
            Option<&OnOffense>,
            Option<&OnDefense>,
            Option<&Controlled>,
            Option<&HasDisc>,
        )>();
        let mut players: Vec<PlayerSnapshot> = player_query
            .iter(world)
            .map(
                |(
                    player,
                    team,
                    pos,
                    vel,
                    on_offense,
                    on_defense,
                    controlled,
                    has_disc,
                )| PlayerSnapshot {
                    id: player.id.0,
                    team: team.team,
                    position_bits: vec3_bits(pos.0),
                    velocity_bits: vec3_bits(vel.0),
                    on_offense: on_offense.is_some(),
                    on_defense: on_defense.is_some(),
                    controlled: controlled.is_some(),
                    has_disc: has_disc.is_some(),
                },
            )
            .collect();
        players.sort_by_key(|p| p.id);

        (
            home_score,
            away_score,
            stall_bits,
            disc,
            players,
        )
    };

    SimulationSnapshot {
        phase,
        home_score,
        away_score,
        stall_bits,
        disc,
        players,
    }
}

// ---------------------------------------------------------------------------
// Active contracts (should pass today)
// ---------------------------------------------------------------------------

#[test]
fn contract_phase_bootstrap_reaches_pull() {
    let mut app = build_test_app();
    for _ in 0..20 {
        app.update();
        if get_game_phase(&app) == GamePhase::Pull {
            return;
        }
    }
    panic!("Expected Pull phase within 20 updates");
}

#[test]
fn contract_prepoint_reset_establishes_7v7_and_puller_possession() {
    let mut app = build_test_app();
    advance_to_pull(&mut app);

    let disc = get_disc_state(&mut app);
    assert!(
        disc.held_by.is_some(),
        "Pull phase should start with a designated puller holding the disc"
    );

    let (home_count, away_count, controlled_count, holder_marked) = {
        let world = app.world_mut();
        let mut q = world.query::<(
            &Player,
            &TeamMember,
            &Position,
            Option<&Controlled>,
            Option<&HasDisc>,
        )>();

        let mut home = 0usize;
        let mut away = 0usize;
        let mut controlled = 0usize;
        let mut holder_marked = false;

        for (player, team, pos, control, has_disc) in q.iter(world) {
            match team.team {
                Team::Home => {
                    home += 1;
                    assert!(
                        approx_eq(pos.0.z, 9.0, 0.5),
                        "Home players should reset to their end zone line, got z={}",
                        pos.0.z
                    );
                }
                Team::Away => {
                    away += 1;
                    assert!(
                        approx_eq(pos.0.z, 91.0, 0.5),
                        "Away players should reset to their end zone line, got z={}",
                        pos.0.z
                    );
                }
            }

            if control.is_some() {
                controlled += 1;
                assert_eq!(
                    team.team,
                    Team::Home,
                    "Human team defaults to Home and should own Controlled player in tests"
                );
            }

            if has_disc.is_some() && disc.held_by == Some(player.id) {
                holder_marked = true;
            }
        }
        (home, away, controlled, holder_marked)
    };

    assert_eq!(home_count, 7, "Expected 7 Home players");
    assert_eq!(away_count, 7, "Expected 7 Away players");
    assert_eq!(controlled_count, 1, "Expected exactly one Controlled player");
    assert!(
        holder_marked,
        "Disc holder should also carry HasDisc marker after reset"
    );
}

#[test]
fn contract_pull_resolution_enters_liveplay_with_offense_possession() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    let disc = get_disc_state(&mut app);
    let holder = disc
        .held_by
        .expect("After pull resolution, offense should possess the disc");

    assert_eq!(
        player_team(&mut app, holder),
        current_offense_team(&mut app),
        "Disc holder must belong to the offensive team in LivePlay"
    );
}

#[test]
fn contract_grounded_disc_causes_turnover_and_new_possession() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    let offense_before = current_offense_team(&mut app);
    force_disc_ground(&mut app, Vec3::new(0.0, 0.0, 50.0));

    tick(&mut app, 1);

    let turnovers = drain_turnovers(&mut app);
    assert!(
        turnovers
            .iter()
            .any(|event| event.reason == TurnoverReason::Drop),
        "Grounded unclaimed disc should emit Drop turnover reason"
    );

    let offense_after = current_offense_team(&mut app);
    assert_ne!(
        offense_before, offense_after,
        "Ground turnover should flip offense"
    );

    let disc = get_disc_state(&mut app);
    let holder = disc
        .held_by
        .expect("Turnover should assign disc to nearest new offense player");
    assert_eq!(
        player_team(&mut app, holder),
        offense_after,
        "Turnover recipient should be on new offense"
    );
}

#[test]
fn contract_stall_out_flips_possession_around_ten_seconds() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    force_disc_to_player(&mut app, PlayerId(0));
    force_has_disc_marker(&mut app, PlayerId(0));
    let offense_before = current_offense_team(&mut app);

    let mut saw_stall = false;
    for _ in 0..700 {
        tick(&mut app, 1);
        let events = drain_turnovers(&mut app);
        if events
            .iter()
            .any(|event| event.reason == TurnoverReason::StallOut)
        {
            saw_stall = true;
            break;
        }
    }

    assert!(saw_stall, "Expected stall-out turnover within ~11.6 seconds");
    let offense_after = current_offense_team(&mut app);
    assert_ne!(offense_before, offense_after, "Stall-out should flip offense");
}

#[test]
fn contract_out_of_bounds_stops_flight_and_flips_possession() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    let offense_before = current_offense_team(&mut app);
    launch_disc_toward_sideline(&mut app);

    let mut saw_oob = false;
    for _ in 0..300 {
        tick(&mut app, 1);
        if !drain_oob_events(&mut app).is_empty() {
            saw_oob = true;
            break;
        }
    }

    assert!(saw_oob, "Expected DiscOutOfBounds event within 300 ticks");

    let disc = get_disc_state(&mut app);
    assert!(
        !disc.in_flight,
        "Out-of-bounds should end disc flight immediately"
    );

    let offense_after = current_offense_team(&mut app);
    assert_ne!(
        offense_before, offense_after,
        "Out-of-bounds turnover should flip offense"
    );
}

#[test]
fn contract_reaching_points_to_win_transitions_to_game_over() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    {
        let world = app.world_mut();
        let mut scoreboard = world.resource_mut::<Scoreboard>();
        scoreboard.home_score = 14;
        scoreboard.away_score = 13;
        scoreboard.points_to_win = 15;
    }

    let home_offense_catcher = {
        let world = app.world_mut();
        let mut q = world.query::<(&Player, &TeamMember, Option<&OnOffense>)>();
        q.iter(world)
            .find(|(player, team, on_offense)| {
                team.team == Team::Home && on_offense.is_some() && player.id != PlayerId(0)
            })
            .map(|(player, _, _)| player.id)
            .expect("Expected Home offense receiver")
    };

    teleport_player(&mut app, home_offense_catcher, Vec3::new(0.0, 0.0, 90.0));

    {
        let world = app.world_mut();
        let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position)>();
        for (mut disc, mut pos) in disc_query.iter_mut(world) {
            disc.in_flight = false;
            disc.grounded = false;
            disc.held_by = Some(home_offense_catcher);
            pos.0 = Vec3::new(0.0, 1.0, 90.0);
        }

        let mut catches = world.resource_mut::<Messages<DiscCaught>>();
        catches.write(DiscCaught {
            catcher: home_offense_catcher,
            position: Vec3::new(0.0, 1.0, 90.0),
            was_layout: false,
        });
    }

    for _ in 0..20 {
        tick(&mut app, 1);
        if get_game_phase(&app) == GamePhase::GameOver {
            break;
        }
    }

    assert_eq!(
        get_game_phase(&app),
        GamePhase::GameOver,
        "Scoring to points_to_win should end the game"
    );

    let scoreboard = get_scoreboard(&app);
    assert_eq!(scoreboard.home_score, 15);
    assert_eq!(scoreboard.away_score, 13);
}

#[test]
fn contract_with_disc_player_pivots_without_translating() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    set_controlled_player(&mut app, PlayerId(0));
    force_disc_to_player(&mut app, PlayerId(0));
    force_has_disc_marker(&mut app, PlayerId(0));

    {
        let world = app.world_mut();
        let mut q = world.query::<(&Player, &mut Velocity)>();
        for (player, mut vel) in q.iter_mut(world) {
            if player.id == PlayerId(0) {
                vel.0 = Vec3::ZERO;
            }
        }

        let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyD);
    }

    let (before_pos, before_facing) = player_pose(&mut app, PlayerId(0));
    tick(&mut app, 5);
    let (after_pos, after_facing) = player_pose(&mut app, PlayerId(0));

    assert!(
        approx_eq(before_pos.x, after_pos.x, POSITION_EPS)
            && approx_eq(before_pos.z, after_pos.z, POSITION_EPS),
        "Pivoting with disc should not translate player (before={:?}, after={:?})",
        before_pos,
        after_pos
    );
    assert!(
        (after_facing - before_facing).abs() > 0.2,
        "Pivot input should rotate facing direction"
    );
}

#[test]
fn contract_manual_fixed_ticks_advance_clock_deterministically() {
    let mut app = build_test_app();
    advance_to_pull(&mut app);

    let before = {
        let world = app.world();
        world.resource::<GameClock>().match_time
    };
    tick(&mut app, 120);
    let after = {
        let world = app.world();
        world.resource::<GameClock>().match_time
    };
    let delta = after - before;

    assert!(
        (delta - 2.0).abs() < 1e-6,
        "120 fixed ticks should be exactly 2.0 seconds, got {delta}"
    );
}

#[test]
fn contract_ai_pull_transitions_to_liveplay() {
    let mut app = build_test_app();
    advance_to_pull(&mut app);
    assert_eq!(get_game_phase(&app), GamePhase::Pull);

    // Let the AI pull system handle it — tick up to 600 frames (~10 seconds).
    // AI decision timer is 0.8-2.0s, then disc flight can take several seconds.
    for _ in 0..600 {
        tick(&mut app, 1);
        if get_game_phase(&app) == GamePhase::LivePlay {
            // Verify offense holds the disc.
            let disc = get_disc_state(&mut app);
            let holder = disc
                .held_by
                .expect("After AI pull resolution, offense should possess the disc");
            assert_eq!(
                player_team(&mut app, holder),
                current_offense_team(&mut app),
                "Disc holder must belong to the offensive team after AI pull"
            );
            return;
        }
    }
    panic!("AI pull did not transition to LivePlay within 600 ticks");
}

#[test]
fn contract_offense_holds_formation_during_pull() {
    let mut app = build_test_app();
    advance_to_pull(&mut app);

    // Record initial positions of AI offense players.
    let initial_positions: Vec<(PlayerId, Vec3)> = {
        let world = app.world_mut();
        let mut q = world.query::<(&Player, &Position, Option<&OnOffense>, Option<&AiControlled>)>();
        q.iter(world)
            .filter(|(_, _, on_offense, ai)| on_offense.is_some() && ai.is_some())
            .map(|(player, pos, _, _)| (player.id, pos.0))
            .collect()
    };

    assert!(
        !initial_positions.is_empty(),
        "Expected at least one AI offense player"
    );

    // Tick 30 frames — offense should be walking to formation, not sprinting aggressively.
    tick(&mut app, 30);

    for (id, initial_pos) in &initial_positions {
        let current_pos = get_player_position(&mut app, *id)
            .unwrap_or_else(|| panic!("Player {:?} not found", id));
        let displacement = (current_pos - *initial_pos).length();
        assert!(
            displacement < 20.0,
            "AI offense player {:?} moved {:.1}m during Pull — expected <20m (formation hold, not aggressive cutting)",
            id, displacement
        );
    }
}

// ---------------------------------------------------------------------------
// Design-driving contracts (intentionally ignored for upcoming phases)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Design contract (03 rules): OOB turnover reason should be OutOfBounds, not Drop"]
fn design_contract_oob_turnover_reason_is_explicit() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);
    launch_disc_toward_sideline(&mut app);

    let event = wait_for_turnover(&mut app, 300).expect("Expected turnover after OOB disc");
    assert_eq!(
        event.reason,
        TurnoverReason::OutOfBounds,
        "Design requires OOB to emit a dedicated turnover reason"
    );
}

#[test]
#[ignore = "Design contract (03 rules): defensive end-zone interception should score a Callahan"]
fn design_contract_callahan_scores_for_defending_team() {
    let mut app = build_test_app();
    advance_to_liveplay(&mut app);

    let away_defender = {
        let world = app.world_mut();
        let mut q = world.query::<(&Player, &TeamMember, Option<&OnDefense>)>();
        q.iter(world)
            .find(|(_, team, on_defense)| team.team == Team::Away && on_defense.is_some())
            .map(|(player, _, _)| player.id)
            .expect("Expected an Away defender")
    };

    {
        let world = app.world_mut();
        let mut disc_query = world.query::<(&mut DiscPhysicsState, &mut Position)>();
        for (mut disc, mut pos) in disc_query.iter_mut(world) {
            disc.in_flight = false;
            disc.grounded = false;
            disc.held_by = Some(away_defender);
            pos.0 = Vec3::new(0.0, 1.0, 10.0); // Away attacking end zone (Home end)
        }

        let mut catches = world.resource_mut::<Messages<DiscCaught>>();
        catches.write(DiscCaught {
            catcher: away_defender,
            position: Vec3::new(0.0, 1.0, 10.0),
            was_layout: false,
        });
    }

    tick(&mut app, 5);

    let scoreboard = get_scoreboard(&app);
    assert_eq!(
        scoreboard.away_score, 1,
        "Callahan should award a point to the intercepting defense"
    );
    assert!(
        get_game_phase(&app) == GamePhase::PointScored || get_game_phase(&app) == GamePhase::GameOver,
        "Callahan should route through scoring state flow"
    );
}

#[test]
#[ignore = "Design contract (03 pull rules): OOB pull should support brick placement"]
fn design_contract_pull_oob_uses_brick_mark_receiving_option() {
    let mut app = build_test_app();
    advance_to_pull(&mut app);

    launch_disc_out_back_of_receiving_endzone_during_pull(&mut app);

    for _ in 0..10 {
        tick(&mut app, 1);
        if get_game_phase(&app) == GamePhase::LivePlay {
            break;
        }
    }

    let disc = get_disc_state(&mut app);
    let holder = disc.held_by.expect("Receiving team should restart with possession");
    let holder_pos =
        get_player_position(&mut app, holder).expect("Expected holder position to exist");

    assert!(
        (holder_pos.z - 18.0).abs() <= 1.0,
        "Brick-mark restart should place offense near z=18, got z={}",
        holder_pos.z
    );
}

#[test]
#[ignore = "Design contract (10 netcode + 15 roadmap): identical inputs must produce identical snapshots"]
fn design_contract_same_inputs_produce_same_snapshot() {
    let mut app_a = build_test_app();
    let mut app_b = build_test_app();

    advance_to_liveplay(&mut app_a);
    advance_to_liveplay(&mut app_b);

    for _ in 0..240 {
        tick(&mut app_a, 1);
        tick(&mut app_b, 1);
    }

    let snap_a = snapshot_world(&mut app_a);
    let snap_b = snapshot_world(&mut app_b);
    assert_eq!(
        snap_a, snap_b,
        "Deterministic simulation contract: same inputs should produce same state"
    );
}

#[test]
#[ignore = "Design contract (02 wind): wind_enabled must meaningfully affect flight"]
fn design_contract_wind_setting_changes_disc_flight() {
    let mut wind_off = build_test_app();
    let mut wind_on = build_test_app();

    advance_to_pull(&mut wind_off);
    advance_to_pull(&mut wind_on);

    {
        let world = wind_off.world_mut();
        world.resource_mut::<MatchConfig>().wind_enabled = false;
    }
    {
        let world = wind_on.world_mut();
        world.resource_mut::<MatchConfig>().wind_enabled = true;
    }

    set_midfield_flight(&mut wind_off);
    set_midfield_flight(&mut wind_on);

    tick(&mut wind_off, 180);
    tick(&mut wind_on, 180);

    let disc_off = get_disc_state(&mut wind_off);
    let disc_on = get_disc_state(&mut wind_on);

    let dx = (disc_off.state.position.x - disc_on.state.position.x).abs();
    let dz = (disc_off.state.position.z - disc_on.state.position.z).abs();
    let horizontal_delta = (dx * dx + dz * dz).sqrt();

    assert!(
        horizontal_delta > 1.0,
        "Wind-enabled flight should diverge from wind-disabled by >1m, got {horizontal_delta:.3}m"
    );
}
