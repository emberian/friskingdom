use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "FrisKingdom".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(fk_core::FkCorePlugin)
    .add_plugins(fk_game::FkGamePlugin)
    .add_plugins(fk_ai::FkAiPlugin)
    .add_plugins(fk_render::FkRenderPlugin)
    .add_plugins(fk_audio::FkAudioPlugin)
    .add_plugins(fk_ui::FkUiPlugin)
    .add_plugins(fk_season::FkSeasonPlugin)
    .add_plugins(fk_tutorial::FkTutorialPlugin);

    #[cfg(feature = "debug")]
    app.add_plugins(fk_debug::FkDebugPlugin);

    app.run();
}
