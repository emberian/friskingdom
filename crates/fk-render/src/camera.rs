use bevy::prelude::*;
use fk_core::components::{Controlled, Disc, MainCamera, Position};

/// Spawn a broadcast-style camera positioned at the sideline, elevated,
/// looking at the center of the field.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(45.0, 25.0, 50.0)
            .looking_at(Vec3::new(0.0, 0.0, 50.0), Vec3::Y),
    ));
}

/// Camera smoothly tracks a point between the disc and the controlled player.
///
/// - If the disc is in flight, the look-at target is the midpoint between the
///   disc and the controlled player.
/// - If no disc is found, the camera tracks the controlled player alone.
/// - Falls back to field center (0, 0, 50) if nothing is available.
///
/// The camera maintains its broadcast angle (elevated sideline view) and
/// smoothly interpolates toward the target using lerp.
pub fn camera_follow_system(
    disc_query: Query<&Position, With<Disc>>,
    controlled_query: Query<&Position, With<Controlled>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let Ok(mut cam_transform) = camera_query.single_mut() else {
        return;
    };

    // Determine the look-at target on the ground plane (y = 0)
    let field_center = Vec3::new(0.0, 0.0, 50.0);

    let controlled_pos = controlled_query
        .iter()
        .next()
        .map(|p| p.0);

    let disc_pos = disc_query
        .iter()
        .next()
        .map(|p| p.0);

    let target_ground = match (disc_pos, controlled_pos) {
        (Some(d), Some(c)) => (d + c) * 0.5,
        (Some(d), None) => d,
        (None, Some(c)) => c,
        (None, None) => field_center,
    };

    // Keep target at ground level for a stable look-at point
    let target_look = Vec3::new(target_ground.x, 0.0, target_ground.z);

    // Camera offset: maintain the broadcast sideline perspective
    let camera_offset = Vec3::new(45.0, 25.0, 0.0);
    let desired_camera_pos = Vec3::new(
        target_look.x + camera_offset.x,
        camera_offset.y,
        target_look.z + camera_offset.z,
    );

    // Smooth interpolation
    let lerp_factor = (time.delta_secs() * 3.0).min(1.0);

    cam_transform.translation = cam_transform
        .translation
        .lerp(desired_camera_pos, lerp_factor);

    // Smoothly update what the camera is looking at
    let current_forward = cam_transform.forward().as_vec3();
    let desired_forward = (target_look - cam_transform.translation).normalize_or_zero();

    if desired_forward.length_squared() > 0.001 {
        let smoothed_forward = current_forward
            .lerp(desired_forward, lerp_factor)
            .normalize_or_zero();

        if smoothed_forward.length_squared() > 0.001 {
            cam_transform.look_to(smoothed_forward, Vec3::Y);
        }
    }
}
