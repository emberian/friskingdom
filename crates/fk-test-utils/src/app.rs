use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

/// Build a headless Bevy App with game-logic plugins and spawned entities.
///
/// Uses `TimeUpdateStrategy::ManualDuration` so each `app.update()` advances
/// time by exactly one fixed timestep (1/60s). This ensures `FixedUpdate`
/// fires once per update call — critical for deterministic headless tests.
///
/// Includes `MinimalPlugins`, `StatesPlugin`, `FkCorePlugin`, `FkGamePlugin`,
/// and `FkAiPlugin`. Also initialises `ButtonInput<KeyCode>` (required by
/// `read_input_system` but not provided by `MinimalPlugins`).
pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(fk_core::FkCorePlugin);
    app.add_plugins(fk_game::FkGamePlugin);
    app.add_plugins(fk_ai::FkAiPlugin);

    // Each app.update() = exactly 1 fixed timestep (1/60s),
    // so FixedUpdate fires once per update. Deterministic.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));

    // read_input_system needs ButtonInput<KeyCode> which MinimalPlugins doesn't provide
    app.init_resource::<ButtonInput<KeyCode>>();

    // Spawn entities at startup (before auto_start_game transitions to InGame)
    app.add_systems(
        Startup,
        (
            crate::spawn::spawn_headless_teams,
            crate::spawn::spawn_headless_disc,
        ),
    );

    app
}

/// Run the app for `n` update cycles.
pub fn tick(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}
