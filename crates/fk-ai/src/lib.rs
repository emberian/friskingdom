pub mod defense;
pub mod offense;
pub mod throwing;

use bevy::prelude::*;
use fk_core::states::{AppState, GamePhase};

pub struct FkAiPlugin;

impl Plugin for FkAiPlugin {
    fn build(&self, app: &mut App) {
        // Component-setup systems run every InGame tick (all phases).
        app.add_systems(
            FixedUpdate,
            (
                offense::ensure_offense_ai_state,
                throwing::ensure_ai_throw_state,
                defense::assign_defenders_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        );

        // LivePlay AI: offense cutting, defense, and throw decisions.
        // Gated to NOT run during Pull (prevents aggressive cutting & puller confusion).
        app.add_systems(
            FixedUpdate,
            (
                offense::offense_ai_system,
                defense::defense_ai_system,
                throwing::ai_throw_decision_system,
            )
                .chain()
                .run_if(in_state(AppState::InGame))
                .run_if(not(in_state(GamePhase::Pull))),
        );

        // Pull-phase AI: pull throw + receiving formation.
        app.add_systems(
            FixedUpdate,
            (
                throwing::ai_pull_system,
                offense::offense_pull_formation_system,
            )
                .run_if(in_state(GamePhase::Pull)),
        );
    }
}
