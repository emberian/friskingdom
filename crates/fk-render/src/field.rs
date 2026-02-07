use bevy::prelude::*;
use fk_core::components::Field;

/// Spawn the ultimate frisbee playing field with line markings, end zones,
/// lighting, and a ground plane.
///
/// Field orientation:
/// - Length along Z axis: 0 to 100 m
/// - Width along X axis: -18.5 to +18.5 m (37 m total)
/// - End zones: z = 0..18 and z = 82..100
/// - Playing field proper: z = 18..82
pub fn spawn_field(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // -----------------------------------------------------------------------
    // Colors
    // -----------------------------------------------------------------------
    let field_green = Color::srgb(0.15, 0.55, 0.15);
    let endzone_green = Color::srgb(0.12, 0.48, 0.12);
    let line_white = Color::WHITE;
    let ground_gray = Color::srgb(0.25, 0.25, 0.25);
    let cone_yellow = Color::srgb(1.0, 0.85, 0.1);

    let field_mat = materials.add(StandardMaterial {
        base_color: field_green,
        ..default()
    });
    let endzone_mat = materials.add(StandardMaterial {
        base_color: endzone_green,
        ..default()
    });
    let line_mat = materials.add(StandardMaterial {
        base_color: line_white,
        unlit: true,
        ..default()
    });
    let ground_mat = materials.add(StandardMaterial {
        base_color: ground_gray,
        ..default()
    });
    let cone_mat = materials.add(StandardMaterial {
        base_color: cone_yellow,
        ..default()
    });

    // Spawn the Field component so other systems can query field dimensions
    commands.spawn(Field::default());

    // -----------------------------------------------------------------------
    // Main playing field surface (between end zones: z=18 to z=82, 64m long)
    // -----------------------------------------------------------------------
    let main_field_mesh = meshes.add(Cuboid::new(37.0, 0.1, 64.0));
    commands.spawn((
        Mesh3d(main_field_mesh),
        MeshMaterial3d(field_mat),
        Transform::from_xyz(0.0, -0.05, 50.0),
    ));

    // -----------------------------------------------------------------------
    // End zone surfaces
    // -----------------------------------------------------------------------
    let endzone_mesh = meshes.add(Cuboid::new(37.0, 0.1, 18.0));

    // Home end zone: z = 0..18, center at z = 9
    commands.spawn((
        Mesh3d(endzone_mesh.clone()),
        MeshMaterial3d(endzone_mat.clone()),
        Transform::from_xyz(0.0, -0.05, 9.0),
    ));

    // Away end zone: z = 82..100, center at z = 91
    commands.spawn((
        Mesh3d(endzone_mesh),
        MeshMaterial3d(endzone_mat),
        Transform::from_xyz(0.0, -0.05, 91.0),
    ));

    // -----------------------------------------------------------------------
    // Ground plane beyond the field
    // -----------------------------------------------------------------------
    let ground_mesh = meshes.add(Cuboid::new(200.0, 0.1, 200.0));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::from_xyz(0.0, -0.15, 50.0),
    ));

    // -----------------------------------------------------------------------
    // Line markings
    // All lines are raised slightly above the field: y = 0.01, height = 0.02
    // -----------------------------------------------------------------------
    let line_height = 0.02;
    let line_y = 0.01;

    // --- Sidelines (along the length, at x = +/-18.5) ---
    // Each sideline runs the full 100m length
    let sideline_mesh = meshes.add(Cuboid::new(0.05, line_height, 100.0));
    // Left sideline
    commands.spawn((
        Mesh3d(sideline_mesh.clone()),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(-18.5, line_y, 50.0),
    ));
    // Right sideline
    commands.spawn((
        Mesh3d(sideline_mesh),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(18.5, line_y, 50.0),
    ));

    // --- End lines (at z = 0 and z = 100) ---
    let endline_mesh = meshes.add(Cuboid::new(37.0, line_height, 0.05));
    // Back end line (z = 0)
    commands.spawn((
        Mesh3d(endline_mesh.clone()),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(0.0, line_y, 0.0),
    ));
    // Front end line (z = 100)
    commands.spawn((
        Mesh3d(endline_mesh),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(0.0, line_y, 100.0),
    ));

    // --- End zone lines (at z = 18 and z = 82) ---
    let zone_line_mesh = meshes.add(Cuboid::new(37.0, line_height, 0.05));
    // Home end zone line (z = 18)
    commands.spawn((
        Mesh3d(zone_line_mesh.clone()),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(0.0, line_y, 18.0),
    ));
    // Away end zone line (z = 82)
    commands.spawn((
        Mesh3d(zone_line_mesh),
        MeshMaterial3d(line_mat.clone()),
        Transform::from_xyz(0.0, line_y, 82.0),
    ));

    // --- Center line (at z = 50) — slightly dimmer ---
    let center_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        unlit: true,
        ..default()
    });
    let center_line_mesh = meshes.add(Cuboid::new(37.0, line_height, 0.05));
    commands.spawn((
        Mesh3d(center_line_mesh),
        MeshMaterial3d(center_line_mat),
        Transform::from_xyz(0.0, line_y, 50.0),
    ));

    // -----------------------------------------------------------------------
    // Cone markers at end zone corners
    // Small cylinders (radius 0.15, height 0.3)
    // -----------------------------------------------------------------------
    let cone_mesh = meshes.add(Cylinder::new(0.15, 0.3));
    let cone_positions = [
        // Home end zone corners (z = 0 and z = 18, x = +/-18.5)
        Vec3::new(-18.5, 0.15, 0.0),
        Vec3::new(18.5, 0.15, 0.0),
        Vec3::new(-18.5, 0.15, 18.0),
        Vec3::new(18.5, 0.15, 18.0),
        // Away end zone corners (z = 82 and z = 100, x = +/-18.5)
        Vec3::new(-18.5, 0.15, 82.0),
        Vec3::new(18.5, 0.15, 82.0),
        Vec3::new(-18.5, 0.15, 100.0),
        Vec3::new(18.5, 0.15, 100.0),
    ];

    for pos in &cone_positions {
        commands.spawn((
            Mesh3d(cone_mesh.clone()),
            MeshMaterial3d(cone_mat.clone()),
            Transform::from_translation(*pos),
        ));
    }

    // -----------------------------------------------------------------------
    // Lighting
    // -----------------------------------------------------------------------

    // Directional light (sun) — angled to cast shadows across the field
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,  // 45 deg down
            std::f32::consts::FRAC_PI_6,   // slight yaw
            0.0,
        )),
    ));
}
