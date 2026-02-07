use bevy::prelude::*;
use fk_core::components::*;
use fk_core::resources::GameInput;
use fk_core::types::InputContext;

pub fn read_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut input: ResMut<GameInput>,
    disc_query: Query<&DiscPhysicsState, With<Disc>>,
    controlled_query: Query<(&Player, Option<&OnOffense>), With<Controlled>>,
) {
    // Reset per-frame flags
    input.throw_released = false;
    input.switch_player = false;
    input.layout_bid = false;

    // Try gamepad first, fall back to keyboard
    if let Some(gamepad) = gamepads.iter().next() {
        // Gamepad input
        let lx = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let ly = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        input.move_dir = Vec2::new(lx, ly);

        let rx = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let ry = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
        input.aim_dir = Vec2::new(rx, ry);

        input.sprint = gamepad.pressed(GamepadButton::LeftTrigger);

        /*ROBOTODO: GamepadAxis::RightZ may not map to the right trigger on all
          controllers. Consider using GamepadButton::RightTrigger2 for digital
          or finding the correct analog axis per platform. */
        let rt = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
        if rt > 0.1 {
            input.throw_charging = true;
            input.throw_power = (input.throw_power + rt * 0.02).min(1.0);
        } else if input.throw_charging {
            input.throw_released = true;
            input.throw_charging = false;
        }

        // Quick throws
        if gamepad.just_pressed(GamepadButton::South) {
            input.throw_type = Some(frisbee_physics::ThrowType::Backhand);
            if !input.throw_charging {
                input.throw_power = 0.5;
                input.throw_released = true;
            }
        }
        if gamepad.just_pressed(GamepadButton::West) {
            input.throw_type = Some(frisbee_physics::ThrowType::Forehand);
            if !input.throw_charging {
                input.throw_power = 0.5;
                input.throw_released = true;
            }
        }
        if gamepad.just_pressed(GamepadButton::North) {
            input.throw_type = Some(frisbee_physics::ThrowType::Hammer);
            if !input.throw_charging {
                input.throw_power = 0.5;
                input.throw_released = true;
            }
        }

        if gamepad.just_pressed(GamepadButton::RightTrigger) {
            input.switch_player = true;
        }
        if gamepad.just_pressed(GamepadButton::East) {
            input.layout_bid = true;
        }
    } else {
        /*ROBOTODO: keyboard has no aim input — aim_dir stays Vec2::ZERO so throws
          always go straight. Add mouse-aim or arrow-key aim for keyboard players.
          See design-docs/04-player-controls.md */
        // Keyboard input
        let mut dir = Vec2::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        input.move_dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::ZERO
        };

        input.sprint = keyboard.pressed(KeyCode::ShiftLeft);

        // Throw with number keys: 1 = backhand, 2 = forehand, 3 = hammer
        if keyboard.just_pressed(KeyCode::Digit1) {
            input.throw_type = Some(frisbee_physics::ThrowType::Backhand);
            input.throw_power = 0.7;
            input.throw_released = true;
        }
        if keyboard.just_pressed(KeyCode::Digit2) {
            input.throw_type = Some(frisbee_physics::ThrowType::Forehand);
            input.throw_power = 0.7;
            input.throw_released = true;
        }
        if keyboard.just_pressed(KeyCode::Digit3) {
            input.throw_type = Some(frisbee_physics::ThrowType::Hammer);
            input.throw_power = 0.7;
            input.throw_released = true;
        }

        if keyboard.just_pressed(KeyCode::Tab) {
            input.switch_player = true;
        }
        if keyboard.just_pressed(KeyCode::Space) {
            input.layout_bid = true;
        }
    }

    // Determine context based on who holds the disc relative to controlled player
    if let Ok(disc_state) = disc_query.single() {
        if let Ok((controlled_player, on_offense)) = controlled_query.single() {
            if let Some(holder_id) = disc_state.held_by {
                if holder_id == controlled_player.id {
                    input.context = InputContext::WithDisc;
                } else if on_offense.is_some() {
                    input.context = InputContext::OffenseNoCut;
                } else {
                    input.context = InputContext::Defense;
                }
            } else if disc_state.in_flight {
                if on_offense.is_some() {
                    input.context = InputContext::OffenseNoCut;
                } else {
                    input.context = InputContext::Defense;
                }
            } else {
                // Disc on ground
                if on_offense.is_some() {
                    input.context = InputContext::OffenseNoCut;
                } else {
                    input.context = InputContext::Defense;
                }
            }
        }
    }
}
