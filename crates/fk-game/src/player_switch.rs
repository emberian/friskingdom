use bevy::prelude::*;
use fk_core::components::*;
use fk_core::events::PlayerSwitched;
use fk_core::resources::{GameInput, HumanTeam};

/// Switch the human-controlled player to the nearest teammate on the same team.
///
/// Reads `GameInput.switch_player` each frame. When true:
/// 1. Finds the currently `Controlled` player entity.
/// 2. Finds the nearest teammate (same `TeamMember.team`) who is NOT the current player.
/// 3. Removes `Controlled` from the old player and inserts `AiControlled`.
/// 4. Inserts `Controlled` on the new player and removes `AiControlled`.
/// 5. Emits a `PlayerSwitched` event.
pub fn player_switch_system(
    input: Res<GameInput>,
    controlled_query: Query<(Entity, &Player, &TeamMember, &Position), With<Controlled>>,
    teammates_query: Query<(Entity, &Player, &TeamMember, &Position), Without<Controlled>>,
    mut commands: Commands,
    mut switch_events: MessageWriter<PlayerSwitched>,
) {
    if !input.switch_player {
        return;
    }

    // Find the currently controlled player
    let Ok((current_entity, current_player, current_team, current_pos)) =
        controlled_query.single()
    else {
        return;
    };

    // Find the nearest teammate who is not the current player
    let mut nearest_entity = None;
    let mut nearest_id = None;
    let mut nearest_dist = f32::MAX;

    for (entity, player, team_member, pos) in &teammates_query {
        // Must be on the same team
        if team_member.team != current_team.team {
            continue;
        }

        // Skip self (shouldn't appear due to Without<Controlled>, but be safe)
        if player.id == current_player.id {
            continue;
        }

        let dist = (pos.0 - current_pos.0).length();
        if dist < nearest_dist {
            nearest_dist = dist;
            nearest_entity = Some(entity);
            nearest_id = Some(player.id);
        }
    }

    // Perform the switch
    let Some(new_entity) = nearest_entity else {
        return;
    };
    let new_player_id = nearest_id.unwrap();

    // Old player: remove Controlled, add AiControlled
    commands
        .entity(current_entity)
        .remove::<Controlled>()
        .insert(AiControlled);

    // New player: insert Controlled, remove AiControlled
    commands
        .entity(new_entity)
        .remove::<AiControlled>()
        .insert(Controlled);

    switch_events.write(PlayerSwitched {
        new_controlled: new_player_id,
        previous: Some(current_player.id),
    });
}

/// Auto-switch the controlled player on defense to the teammate nearest the disc.
///
/// This makes defense feel responsive — the human always controls the closest
/// defender to the disc rather than manually Tab-switching every time.
/// Only switches when:
/// - The human's team is on defense (no OnOffense on Controlled player)
/// - The disc is in flight
/// - A closer teammate exists (> 3m closer to avoid flickering)
pub fn auto_switch_defense_system(
    human_team: Res<HumanTeam>,
    disc_query: Query<(&DiscPhysicsState, &Position), With<Disc>>,
    controlled_query: Query<(Entity, &Player, &TeamMember, &Position, Option<&OnOffense>), With<Controlled>>,
    teammates_query: Query<(Entity, &Player, &TeamMember, &Position), (Without<Controlled>, Without<Disc>)>,
    mut commands: Commands,
) {
    let Ok((disc_state, disc_pos)) = disc_query.single() else {
        return;
    };

    // Only auto-switch when disc is in flight
    if !disc_state.in_flight {
        return;
    }

    let Ok((current_entity, _current_player, current_team, current_pos, on_offense)) =
        controlled_query.single()
    else {
        return;
    };

    // Only auto-switch on defense
    if on_offense.is_some() {
        return;
    }

    // Must be on human's team
    if current_team.team != human_team.0 {
        return;
    }

    let current_dist = (current_pos.0 - disc_pos.0).length();

    // Find closest teammate on the same team to the disc
    let mut best_entity = None;
    let mut best_dist = current_dist;

    for (entity, _player, team_member, pos) in &teammates_query {
        if team_member.team != current_team.team {
            continue;
        }
        let dist = (pos.0 - disc_pos.0).length();
        if dist < best_dist {
            best_dist = dist;
            best_entity = Some(entity);
        }
    }

    // Only switch if the new player is meaningfully closer (3m+ hysteresis)
    let Some(new_entity) = best_entity else {
        return;
    };
    if current_dist - best_dist < 3.0 {
        return;
    }

    commands
        .entity(current_entity)
        .remove::<Controlled>()
        .insert(AiControlled);

    commands
        .entity(new_entity)
        .remove::<AiControlled>()
        .insert(Controlled);
}
