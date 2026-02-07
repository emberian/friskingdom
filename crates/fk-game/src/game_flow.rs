use bevy::prelude::*;
use fk_core::components::*;
use fk_core::events::*;
use fk_core::resources::*;
use fk_core::states::GamePhase;
use fk_core::types::*;

// ---------------------------------------------------------------------------
// 1. Game clock -- ticks match_time and point_time every frame while InGame
// ---------------------------------------------------------------------------

pub fn game_clock_system(time: Res<Time>, mut clock: ResMut<GameClock>) {
    let dt = time.delta_secs_f64();
    clock.match_time += dt;
    clock.point_time += dt;
}

// ---------------------------------------------------------------------------
// 2. Stall count -- increments while disc is held, triggers stall-out at 10
// ---------------------------------------------------------------------------

pub fn stall_count_system(
    time: Res<Time>,
    mut stall: ResMut<StallCount>,
    disc_query: Query<&DiscPhysicsState, With<Disc>>,
    mut stall_events: MessageWriter<StallOutEvent>,
) {
    let Ok(disc) = disc_query.single() else {
        return;
    };

    if let Some(holder_id) = disc.held_by {
        stall.active = true;
        stall.count += time.delta_secs();

        if stall.count >= 10.0 {
            stall_events.write(StallOutEvent { thrower: holder_id });
            // Reset so we don't fire repeatedly
            stall.count = 0.0;
            stall.active = false;
        }
    } else {
        stall.count = 0.0;
        stall.active = false;
    }
}

// ---------------------------------------------------------------------------
// 3. Scoring -- detect catch in attacking end zone by offense
// ---------------------------------------------------------------------------

pub fn scoring_system(
    mut catch_reader: MessageReader<DiscCaught>,
    player_query: Query<(&Player, &TeamMember, Option<&OnOffense>)>,
    disc_query: Query<Option<&LastThrower>, With<Disc>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut point_events: MessageWriter<PointScored>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    for event in catch_reader.read() {
        // Find the catcher's team and offense status
        let Some((_, catcher_team, catcher_offense)) = player_query
            .iter()
            .find(|(p, _, _)| p.id == event.catcher)
        else {
            continue;
        };

        // Only score if catcher is on offense
        if catcher_offense.is_none() {
            continue;
        }

        // Check if the catch is in the attacking end zone:
        //   Home attacks toward Away end zone (z >= 82.0)
        //   Away attacks toward Home end zone (z <= 18.0)
        let in_end_zone = match catcher_team.team {
            Team::Home => event.position.z >= 82.0,
            Team::Away => event.position.z <= 18.0,
        };

        if !in_end_zone {
            continue;
        }

        // Determine the assister from LastThrower on the disc entity
        let assister = disc_query
            .single()
            .ok()
            .flatten()
            .map(|lt| lt.player);

        // Update scoreboard
        match catcher_team.team {
            Team::Home => scoreboard.home_score += 1,
            Team::Away => scoreboard.away_score += 1,
        }

        point_events.write(PointScored {
            scoring_team: catcher_team.team,
            scorer: event.catcher,
            assister,
            is_callahan: false,
        });

        next_phase.set(GamePhase::PointScored);
    }
}

// ---------------------------------------------------------------------------
// 4. Turnover -- disc on ground without a holder, or stall-out
// ---------------------------------------------------------------------------

pub fn turnover_system(
    mut stall_reader: MessageReader<StallOutEvent>,
    mut disc_query: Query<(&mut DiscPhysicsState, &Position, Entity), With<Disc>>,
    players: Query<(Entity, &Player, &TeamMember, &Position, Option<&OnOffense>), Without<Disc>>,
    mut commands: Commands,
    mut turnover_events: MessageWriter<TurnoverOccurred>,
) {
    // First pass: read disc state immutably to decide if turnover happened
    let (is_ground_turnover, is_stall, disc_pos) = {
        let Ok((disc, pos, _)) = disc_query.single() else {
            // Drain stall events even if no disc
            for _ in stall_reader.read() {}
            return;
        };
        let ground = disc.grounded && disc.held_by.is_none() && !disc.in_flight;
        let stall = stall_reader.read().next().is_some();
        (ground, stall, pos.0)
    };

    let reason = if is_ground_turnover {
        TurnoverReason::Drop
    } else if is_stall {
        TurnoverReason::StallOut
    } else {
        return;
    };

    // Determine which team is currently on offense
    let Some(offense_team) = players
        .iter()
        .find(|(_, _, _, _, on_off)| on_off.is_some())
        .map(|(_, _, tm, _, _)| tm.team)
    else {
        return;
    };

    let new_offense_team = offense_team.opposite();

    // Swap OnOffense / OnDefense markers on all players
    for (entity, _, team_member, _, _) in &players {
        if team_member.team == new_offense_team {
            commands.entity(entity).remove::<OnDefense>().insert(OnOffense);
        } else {
            commands.entity(entity).remove::<OnOffense>().insert(OnDefense);
        }
    }

    // Find nearest player on the new offense team to pick up disc
    let mut nearest_entity = None;
    let mut nearest_id = None;
    let mut nearest_dist = f32::MAX;

    for (entity, player, team_member, pos, _) in &players {
        if team_member.team == new_offense_team {
            let dist = (pos.0 - disc_pos).length();
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_entity = Some(entity);
                nearest_id = Some(player.id);
            }
        }
    }

    // Now mutably access disc to assign it
    if let (Some(pickup_entity), Some(pickup_id)) = (nearest_entity, nearest_id) {
        if let Ok((mut disc, _, disc_entity)) = disc_query.single_mut() {
            disc.held_by = Some(pickup_id);
            disc.grounded = false;
            disc.in_flight = false;

            commands.entity(pickup_entity).insert(HasDisc);
            commands.entity(disc_entity).insert(LastThrower { player: pickup_id });
        }
    }

    turnover_events.write(TurnoverOccurred {
        reason,
        position: disc_pos,
        new_offense: new_offense_team,
    });
}

// ---------------------------------------------------------------------------
// 5. HasDisc sync -- keep HasDisc marker in sync with DiscPhysicsState.held_by
// ---------------------------------------------------------------------------

pub fn has_disc_sync_system(
    disc_query: Query<&DiscPhysicsState, With<Disc>>,
    players: Query<(Entity, &Player), Without<Disc>>,
    mut commands: Commands,
) {
    let Ok(disc) = disc_query.single() else {
        return;
    };

    for (entity, player) in &players {
        if disc.held_by == Some(player.id) {
            commands.entity(entity).insert(HasDisc);
        } else {
            commands.entity(entity).remove::<HasDisc>();
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Point reset -- runs on entering PrePoint, resets positions and state
// ---------------------------------------------------------------------------

pub fn point_reset_system(
    mut players: Query<(Entity, &Player, &TeamMember, &mut Position, &mut FacingDirection, Option<&OnOffense>), Without<Disc>>,
    mut disc_query: Query<(&mut DiscPhysicsState, Entity), With<Disc>>,
    mut stall: ResMut<StallCount>,
    mut clock: ResMut<GameClock>,
    human_team: Res<HumanTeam>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    // Reset stall count and point timer
    stall.count = 0.0;
    stall.active = false;
    clock.point_time = 0.0;

    // After a point, the scoring team goes on defense (receives the pull).
    // On the very first point (point_number == 1), keep offense/defense as-is.
    // For subsequent points, the swap was already done conceptually:
    //   "team that scored now pulls" -- so the scoring team should now be on defense.
    // We rely on check_game_over_system to have already done the swap before
    // transitioning to PrePoint. So here we just position players according to
    // their current offense/defense status.

    // Determine which team is currently on defense (they are the pulling team)
    // Position players in their end zones
    // Home end zone: z = 0..18, center at z = 9
    // Away end zone: z = 82..100, center at z = 91
    let mut home_idx: usize = 0;
    let mut away_idx: usize = 0;

    for (entity, _, team_member, mut pos, mut facing, _on_offense) in &mut players {
        let (base_z, idx, face_angle) = match team_member.team {
            Team::Home => {
                let i = home_idx;
                home_idx += 1;
                // Home in their end zone (z=9), facing +Z (downfield) = 0.0
                (9.0, i, 0.0_f32)
            }
            Team::Away => {
                let i = away_idx;
                away_idx += 1;
                // Away in their end zone (z=91), facing -Z (downfield) = PI
                (91.0, i, std::f32::consts::PI)
            }
        };

        // Spread players across the width of the field
        // 7 players per team, spread from -12 to +12
        let spread_x = -12.0 + (idx as f32) * 4.0;
        pos.0 = Vec3::new(spread_x, 0.0, base_z);
        facing.0 = face_angle;

        // Remove HasDisc and Controlled from all players during reset
        commands.entity(entity).remove::<HasDisc>().remove::<Controlled>().insert(AiControlled);
    }

    // Give the disc to a player on the pulling (defense) team
    if let Ok((mut disc, disc_entity)) = disc_query.single_mut() {
        disc.held_by = None;
        disc.in_flight = false;
        disc.grounded = false;

        // Find a player on the defense team (currently NOT on offense) to hold the disc
        // The pulling team is the one on defense
        let puller = players
            .iter()
            .find(|(_, _, _, _, _, on_off)| on_off.is_none())
            .map(|(_, p, _, _, _, _)| p.id);

        if let Some(puller_id) = puller {
            disc.held_by = Some(puller_id);

            // Mark the puller with HasDisc and set LastThrower
            if let Some((puller_entity, _, _, _, _, _)) = players
                .iter()
                .find(|(_, p, _, _, _, _)| p.id == puller_id)
            {
                commands.entity(puller_entity).insert(HasDisc);
            }
            commands.entity(disc_entity).insert(LastThrower { player: puller_id });
        }

        // Give Controlled to a player on the human's team.
        // If the human's team is pulling (defense), give it to the puller.
        // Otherwise give it to the first player on the human's team.
        let human_player = if puller.is_some() {
            // Check if puller is on human team
            players.iter()
                .find(|(_, p, tm, _, _, _)| Some(p.id) == puller && tm.team == human_team.0)
                .map(|(e, _, _, _, _, _)| e)
                .or_else(|| {
                    // Puller is not on human team, find any human team player
                    players.iter()
                        .find(|(_, _, tm, _, _, _)| tm.team == human_team.0)
                        .map(|(e, _, _, _, _, _)| e)
                })
        } else {
            players.iter()
                .find(|(_, _, tm, _, _, _)| tm.team == human_team.0)
                .map(|(e, _, _, _, _, _)| e)
        };

        if let Some(human_entity) = human_player {
            commands.entity(human_entity)
                .insert(Controlled)
                .remove::<AiControlled>();
        }
    }

    // Transition to Pull phase
    next_phase.set(GamePhase::Pull);
}

// ---------------------------------------------------------------------------
// 7. Pull -- defense throws the disc downfield to start the point
// ---------------------------------------------------------------------------

/// During Pull phase: handles the pull throw, then waits for disc to resolve.
pub fn pull_system(
    input: Res<GameInput>,
    mut disc_query: Query<(&mut DiscPhysicsState, &Position, Entity), With<Disc>>,
    controlled_query: Query<(&Player, &Position, &FacingDirection), With<Controlled>>,
    players: Query<(Entity, &Player, &TeamMember, &Position, Option<&OnOffense>), Without<Disc>>,
    mut throw_events: MessageWriter<DiscThrown>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    let Ok((mut disc, disc_pos, disc_entity)) = disc_query.single_mut() else {
        return;
    };

    // State 1: disc is held — wait for player to throw the pull
    if disc.held_by.is_some() && !disc.in_flight {
        if !input.throw_released {
            return;
        }

        let holder_id = disc.held_by.unwrap();

        for (player, _player_pos, player_facing) in &controlled_query {
            if player.id != holder_id {
                continue;
            }

            let mods = frisbee_physics::ThrowModifications {
                power: input.throw_power.max(0.8) as f64,
                aim_angle: input.aim_dir.x as f64 * 0.5,
                hyzer_adjust: 0.0,
                nose_adjust: 0.05,
            };

            let facing = player_facing.0 as f64;
            let throw_type = input
                .throw_type
                .unwrap_or(frisbee_physics::ThrowType::Backhand);

            let throw_state = frisbee_physics::create_throw_state(
                throw_type,
                &mods,
                glam::DVec3::new(
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
            // Stay in Pull phase — wait for disc to land or be caught
            return;
        }
        return;
    }

    // State 2: disc is in flight — wait for it to land or be caught
    if disc.in_flight {
        return;
    }

    // State 3: disc was caught by a player (held_by is set by catch_system)
    if disc.held_by.is_some() {
        // Disc was caught — transition to LivePlay
        next_phase.set(GamePhase::LivePlay);
        return;
    }

    // State 4: disc is on the ground (grounded) — offense picks it up
    if disc.grounded {
        // Find nearest offense player
        let mut nearest_entity = None;
        let mut nearest_id = None;
        let mut nearest_dist = f32::MAX;

        for (entity, player, _, pos, on_offense) in &players {
            if on_offense.is_some() {
                let dist = (pos.0 - disc_pos.0).length();
                if dist < nearest_dist {
                    nearest_dist = dist;
                    nearest_entity = Some(entity);
                    nearest_id = Some(player.id);
                }
            }
        }

        if let (Some(pickup_entity), Some(pickup_id)) = (nearest_entity, nearest_id) {
            disc.held_by = Some(pickup_id);
            disc.grounded = false;
            commands.entity(pickup_entity).insert(HasDisc);
            commands
                .entity(disc_entity)
                .insert(LastThrower { player: pickup_id });
        }

        next_phase.set(GamePhase::LivePlay);
    }
}

// ---------------------------------------------------------------------------
// 8. Check game over -- after a point, see if someone has won
// ---------------------------------------------------------------------------

pub fn check_game_over_system(
    mut point_reader: MessageReader<PointScored>,
    scoreboard: Res<Scoreboard>,
    mut clock: ResMut<GameClock>,
    players: Query<(Entity, &TeamMember), With<Player>>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    for event in point_reader.read() {
        // Check win condition
        if scoreboard.home_score >= scoreboard.points_to_win
            || scoreboard.away_score >= scoreboard.points_to_win
        {
            next_phase.set(GamePhase::GameOver);
            return;
        }

        // Increment point number
        clock.point_number += 1;

        // After scoring, the scoring team goes on defense (pulls next).
        // Swap offense/defense: scoring team -> defense, other team -> offense.
        let scoring_team = event.scoring_team;

        for (entity, team_member) in &players {
            if team_member.team == scoring_team {
                // Scoring team goes on defense (they pull)
                commands.entity(entity).remove::<OnOffense>().insert(OnDefense);
            } else {
                // Other team goes on offense (they receive)
                commands.entity(entity).remove::<OnDefense>().insert(OnOffense);
            }
        }

        // Transition to PrePoint for reset
        next_phase.set(GamePhase::PrePoint);
    }
}
