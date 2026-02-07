use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

pub fn screenshot_system(
    mut commands: Commands,
    frame_count: Res<FrameCount>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let should_screenshot = frame_count.0 % 120 == 0 || keyboard.just_pressed(KeyCode::F11);

    if should_screenshot {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("/tmp/friskingdom-debug/screenshot.png"));
    }
}
