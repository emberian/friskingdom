use bevy::prelude::*;
use fk_core::components::*;
use fk_core::resources::*;
use fk_core::states::*;
use fk_core::types::*;

/// Return the current `GamePhase` sub-state.
pub fn get_game_phase(app: &App) -> GamePhase {
    app.world().resource::<State<GamePhase>>().get().clone()
}

/// Return the current `AppState`.
pub fn get_app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

/// Return a clone of the `Scoreboard` resource.
pub fn get_scoreboard(app: &App) -> Scoreboard {
    app.world().resource::<Scoreboard>().clone()
}

/// Return a clone of the `StallCount` resource.
pub fn get_stall_count(app: &App) -> StallCount {
    app.world().resource::<StallCount>().clone()
}

/// Return a clone of the `DiscPhysicsState` from the single disc entity.
pub fn get_disc_state(app: &mut App) -> DiscPhysicsState {
    let world = app.world_mut();
    let mut state = world.query::<&DiscPhysicsState>();
    state.single(world).expect("expected exactly one Disc entity").clone()
}

/// Mutate the `GameInput` resource via a closure.
pub fn inject_input(app: &mut App, mutator: impl FnOnce(&mut GameInput)) {
    let mut input = app.world_mut().resource_mut::<GameInput>();
    mutator(&mut input);
}

/// Get position of a specific player by `PlayerId`.
pub fn get_player_position(app: &mut App, id: PlayerId) -> Option<Vec3> {
    let world = app.world_mut();
    let mut query = world.query::<(&Player, &Position)>();
    for (player, pos) in query.iter(world) {
        if player.id == id {
            return Some(pos.0);
        }
    }
    None
}

/// Check if any player on the given team has the `OnOffense` marker.
pub fn is_team_on_offense(app: &mut App, team: Team) -> bool {
    let world = app.world_mut();
    let mut query = world.query::<(&TeamMember, &OnOffense)>();
    query.iter(world).any(|(tm, _)| tm.team == team)
}
