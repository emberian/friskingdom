use bevy::prelude::*;

#[derive(Resource)]
pub struct DebugFrameControl {
    pub paused: bool,
    pub step_one: bool,
    pub speed_multiplier: f64,
}

impl Default for DebugFrameControl {
    fn default() -> Self {
        Self {
            paused: false,
            step_one: false,
            speed_multiplier: 1.0,
        }
    }
}

pub fn frame_control_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut control: ResMut<DebugFrameControl>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        control.paused = !control.paused;
    }
    if keyboard.just_pressed(KeyCode::KeyN) && control.paused {
        control.step_one = true;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        control.speed_multiplier = if control.speed_multiplier > 1.5 { 1.0 } else { 3.0 };
    }
}

pub fn frame_control_apply_system(
    mut control: ResMut<DebugFrameControl>,
    mut time: ResMut<Time<Virtual>>,
) {
    if control.paused && !control.step_one {
        time.set_relative_speed(0.0);
    } else if control.step_one {
        // Allow one frame at normal speed
        time.set_relative_speed(1.0);
        control.step_one = false;
    } else {
        time.set_relative_speed(control.speed_multiplier as f32);
    }
}
