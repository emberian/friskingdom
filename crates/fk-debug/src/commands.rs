use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use fk_core::resources::GameInput;
use serde::Deserialize;
use std::path::Path;

use crate::frame_control::DebugFrameControl;

#[derive(Deserialize)]
struct DebugCommand {
    command: String,
    #[serde(default)]
    data: serde_json::Value,
}

pub fn command_watcher_system(
    mut commands: Commands,
    mut input: ResMut<GameInput>,
    mut control: ResMut<DebugFrameControl>,
) {
    let path = Path::new("/tmp/friskingdom-debug/commands.json");
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    // Delete immediately to avoid re-processing
    let _ = std::fs::remove_file(path);

    let Ok(cmd) = serde_json::from_str::<DebugCommand>(&content) else {
        return;
    };

    match cmd.command.as_str() {
        "screenshot" => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk("/tmp/friskingdom-debug/screenshot.png"));
        }
        "pause" => {
            control.paused = true;
        }
        "resume" => {
            control.paused = false;
        }
        "step" => {
            control.step_one = true;
        }
        "set_speed" => {
            if let Some(mult) = cmd.data.get("multiplier").and_then(|v| v.as_f64()) {
                control.speed_multiplier = mult;
            }
        }
        "inject_input" => {
            if let Some(data) = cmd.data.as_object() {
                if let Some(v) = data.get("throw_released").and_then(|v| v.as_bool()) {
                    input.throw_released = v;
                }
                if let Some(v) = data.get("throw_power").and_then(|v| v.as_f64()) {
                    input.throw_power = v as f32;
                }
                if let Some(v) = data.get("sprint").and_then(|v| v.as_bool()) {
                    input.sprint = v;
                }
                if let Some(arr) = data.get("move_dir").and_then(|v| v.as_array()) {
                    if arr.len() == 2 {
                        input.move_dir.x = arr[0].as_f64().unwrap_or(0.0) as f32;
                        input.move_dir.y = arr[1].as_f64().unwrap_or(0.0) as f32;
                    }
                }
                if let Some(t) = data.get("throw_type").and_then(|v| v.as_str()) {
                    input.throw_type = match t {
                        "Backhand" => Some(frisbee_physics::ThrowType::Backhand),
                        "Forehand" => Some(frisbee_physics::ThrowType::Forehand),
                        "Hammer" => Some(frisbee_physics::ThrowType::Hammer),
                        _ => None,
                    };
                }
                if let Some(v) = data.get("switch_player").and_then(|v| v.as_bool()) {
                    input.switch_player = v;
                }
            }
        }
        "dump_state" => {
            // The state_dump_system runs on its own schedule,
            // but we can trigger an immediate dump by doing nothing special --
            // just log that we got the command. The next frame will dump.
            // In a more sophisticated version we'd use a trigger resource.
        }
        _ => {}
    }
}
