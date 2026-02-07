mod state_dump;
mod screenshot;
mod frame_control;
mod commands;

use bevy::prelude::*;

pub struct FkDebugPlugin;

impl Plugin for FkDebugPlugin {
    fn build(&self, app: &mut App) {
        // Create debug output directory
        std::fs::create_dir_all("/tmp/friskingdom-debug").ok();

        app.init_resource::<frame_control::DebugFrameControl>();

        app.add_systems(
            Update,
            (
                frame_control::frame_control_input_system,
                frame_control::frame_control_apply_system,
                commands::command_watcher_system,
                state_dump::state_dump_system,
                screenshot::screenshot_system,
            ),
        );
    }
}
