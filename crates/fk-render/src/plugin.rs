use bevy::prelude::*;
use fk_core::states::AppState;

pub struct FkRenderPlugin;

impl Plugin for FkRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (
                crate::field::spawn_field,
                crate::camera::spawn_camera,
                crate::stickman::spawn_teams,
                crate::disc_visual::spawn_disc,
            ),
        );

        app.add_systems(
            Update,
            (
                crate::camera::camera_follow_system,
                crate::stickman::sync_player_visuals,
                crate::disc_visual::sync_disc_visual,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
