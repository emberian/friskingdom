use bevy::prelude::*;
use fk_core::components::*;
use fk_core::types::PlayerId;

/// Marker for the disc's visual entity.
#[derive(Component)]
pub struct DiscVisualMarker;

/// Spawn the disc entity with both game-logic components and a visual mesh.
///
/// The disc is modelled as a flat cylinder:
/// - Radius: 0.1365 m (standard Ultrastar diameter / 2)
/// - Height: 0.032 m
pub fn spawn_disc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let disc_mesh = meshes.add(Cylinder::new(0.1365, 0.032));
    let disc_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    // Start the disc near the home-team handler
    let start_pos = Vec3::new(0.0, 1.0, 20.0);

    // Build a default DiscState with zeroed physics
    let initial_state = frisbee_physics::DiscState {
        position: frisbee_physics::PhysDVec3::new(
            start_pos.x as f64,
            start_pos.y as f64,
            start_pos.z as f64,
        ),
        velocity: frisbee_physics::PhysDVec3::ZERO,
        orientation: frisbee_physics::Euler::new(0.0, 0.0, 0.0),
        angular_velocity: frisbee_physics::PhysDVec3::ZERO,
        spin_rate: 0.0,
        time: 0.0,
    };

    commands.spawn((
        Disc,
        Position(start_pos),
        Velocity(Vec3::ZERO),
        DiscPhysicsState {
            state: initial_state,
            in_flight: false,
            grounded: false,
            held_by: Some(PlayerId(0)),
        },
        DiscVisualMarker,
        Mesh3d(disc_mesh),
        MeshMaterial3d(disc_material),
        Transform::from_translation(start_pos),
    ));
}

/// Synchronize the disc's visual transform from the ECS `Position` and
/// `DiscPhysicsState` components.
///
/// When the disc is in flight, its orientation is taken from the physics
/// state's Euler angles.
pub fn sync_disc_visual(
    mut query: Query<(&Position, &DiscPhysicsState, &mut Transform), With<Disc>>,
) {
    for (pos, physics, mut transform) in &mut query {
        transform.translation = pos.0;

        if physics.in_flight {
            // Physics uses R = Ry(yaw) * Rx(pitch) * Rz(roll), which is
            // intrinsic Y-X-Z. glam's EulerRot::YXZ matches this convention.
            let euler = &physics.state.orientation;
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                euler.yaw as f32,
                euler.pitch as f32,
                euler.roll as f32,
            );
        }
    }
}
