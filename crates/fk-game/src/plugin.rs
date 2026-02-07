use bevy::prelude::*;
use fk_core::states::{AppState, GamePhase};

pub struct FkGamePlugin;

impl Plugin for FkGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.init_resource::<crate::disc_system::PhysicsSimulator>();

        // --- Systems that run every tick during InGame ---
        app.add_systems(
            FixedUpdate,
            (
                crate::input::read_input_system,
                crate::game_flow::has_disc_sync_system,
                crate::game_flow::game_clock_system,
                crate::player_movement::player_movement_system,
                crate::disc_system::disc_physics_system,
                crate::disc_system::disc_holder_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        );

        // --- Pull phase: pull throw + OOB + catch + disc resolve ---
        app.add_systems(
            FixedUpdate,
            (
                crate::disc_system::out_of_bounds_system,
                crate::disc_system::catch_system,
                crate::game_flow::pull_system,
                crate::player_switch::auto_switch_defense_system,
            )
                .chain()
                .run_if(in_state(GamePhase::Pull)),
        );

        // --- LivePlay: throwing, OOB, catching, stall, scoring, turnovers ---
        app.add_systems(
            FixedUpdate,
            (
                crate::disc_system::throw_system,
                crate::disc_system::out_of_bounds_system,
                crate::disc_system::catch_system,
                crate::game_flow::stall_count_system,
                crate::game_flow::scoring_system,
                crate::game_flow::turnover_system,
                crate::player_switch::player_switch_system,
                crate::player_switch::auto_switch_defense_system,
            )
                .chain()
                .run_if(in_state(GamePhase::LivePlay)),
        );

        // --- PointScored: check for game over or transition to next point ---
        app.add_systems(
            FixedUpdate,
            crate::game_flow::check_game_over_system
                .run_if(in_state(GamePhase::PointScored)),
        );

        // --- OnEnter(PrePoint): reset positions, assign disc to puller ---
        app.add_systems(
            OnEnter(GamePhase::PrePoint),
            crate::game_flow::point_reset_system,
        );

        // Auto-transition Loading -> InGame (skip menus for now)
        app.add_systems(OnEnter(AppState::Loading), auto_start_game);
    }
}

fn auto_start_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}
