use bevy::prelude::*;
use fk_core::components::*;
use fk_core::types::*;
use rand::Rng;

/// Marker for the visual root of a stickman.
#[derive(Component)]
pub struct StickmanVisual {
    pub player_id: PlayerId,
}

/*ROBOTODO: SelectionRing is a child of the initially controlled player entity.
  player_switch_system exists in fk-game but the ring visual doesn't re-parent.
  Need a sync system that despawns the old ring and spawns a new one on the
  newly Controlled player, or use a non-parented approach (world-space ring). */
/// Marker for the controlled player's selection ring.
#[derive(Component)]
pub struct SelectionRing;

/// Spawn both teams (7v7) in their respective end zones.
pub fn spawn_teams(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    // Home team (blue) -- 7 players in the home end zone (z = 0..18, center z = 9)
    // Spread across x from -12 to +12
    for i in 0..7u32 {
        let x = -12.0 + (i as f32) * 4.0;
        let z = 9.0;
        spawn_player(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut rng,
            PlayerId(i),
            Team::Home,
            Vec3::new(x, 0.0, z),
            i == 0, // first home player is controlled
        );
    }

    // Away team (red) -- 7 players in the away end zone (z = 82..100, center z = 91)
    for i in 0..7u32 {
        let x = -12.0 + (i as f32) * 4.0;
        let z = 91.0;
        spawn_player(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut rng,
            PlayerId(7 + i),
            Team::Away,
            Vec3::new(x, 0.0, z),
            false,
        );
    }
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    rng: &mut impl Rng,
    id: PlayerId,
    team: Team,
    position: Vec3,
    is_controlled: bool,
) {
    let team_color = match team {
        Team::Home => Color::srgb(0.2, 0.3, 0.9), // Blue
        Team::Away => Color::srgb(0.9, 0.2, 0.2), // Red
    };
    let skin_color = Color::srgb(0.85, 0.7, 0.55);

    let role = if id.0 % 7 < 3 {
        PlayerRole::Handler
    } else {
        PlayerRole::Cutter
    };
    let stats = match role {
        PlayerRole::Handler => PlayerStats::random_handler(rng),
        PlayerRole::Cutter | PlayerRole::Hybrid => PlayerStats::random_cutter(rng),
    };

    // Materials
    let team_mat = materials.add(StandardMaterial {
        base_color: team_color,
        ..default()
    });
    let skin_mat = materials.add(StandardMaterial {
        base_color: skin_color,
        ..default()
    });

    // Meshes
    let head_mesh = meshes.add(Sphere::new(0.12));
    let torso_mesh = meshes.add(Capsule3d::new(0.06, 0.3));
    let arm_mesh = meshes.add(Capsule3d::new(0.03, 0.35));
    let leg_mesh = meshes.add(Capsule3d::new(0.04, 0.50));

    let mut player_entity = commands.spawn((
        Player {
            id,
            name: format!("Player {}", id.0),
            /*ROBOTODO: randomize handedness (e.g. 85% right, 15% left).
          Affects throw selection and disc grip side. */
        handedness: Handedness::Right,
        },
        TeamMember { team },
        Position(position),
        Velocity(Vec3::ZERO),
        stats,
        Stamina::new(0.5 + rng.gen::<f32>() * 0.4),
        SwagMeter::default(),
        role,
        FacingDirection(0.0),
        AiMoveTarget::default(),
        StickmanVisual { player_id: id },
        Transform::from_translation(position),
        Visibility::default(),
    ));

    // Initial offense/defense is arbitrary here — point_reset_system overrides
    // these markers immediately when PrePoint fires before the first pull.
    if team == Team::Home {
        player_entity.insert(OnOffense);
    } else {
        player_entity.insert(OnDefense);
    }

    // Control marker
    if is_controlled {
        player_entity.insert(Controlled);
    } else {
        player_entity.insert(AiControlled);
    }

    // -----------------------------------------------------------------------
    // Child body parts (local transforms relative to the player root)
    // -----------------------------------------------------------------------
    player_entity.with_children(|parent| {
        // Head (sphere at top of stickman)
        parent.spawn((
            Mesh3d(head_mesh.clone()),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.0, 1.7, 0.0),
        ));

        // Torso (capsule centered on chest area)
        parent.spawn((
            Mesh3d(torso_mesh),
            MeshMaterial3d(team_mat.clone()),
            Transform::from_xyz(0.0, 1.25, 0.0),
        ));

        // Left arm
        parent.spawn((
            Mesh3d(arm_mesh.clone()),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(-0.2, 1.15, 0.0)
                .with_rotation(Quat::from_rotation_z(0.15)),
        ));

        // Right arm
        parent.spawn((
            Mesh3d(arm_mesh),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.2, 1.15, 0.0)
                .with_rotation(Quat::from_rotation_z(-0.15)),
        ));

        // Left leg
        parent.spawn((
            Mesh3d(leg_mesh.clone()),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(-0.08, 0.45, 0.0),
        ));

        // Right leg
        parent.spawn((
            Mesh3d(leg_mesh),
            MeshMaterial3d(skin_mat),
            Transform::from_xyz(0.08, 0.45, 0.0),
        ));

        // Selection ring (only for the controlled player)
        if is_controlled {
            let ring_mesh = meshes.add(Cylinder::new(0.5, 0.02));
            let ring_mat = materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 0.0, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            parent.spawn((
                SelectionRing,
                Mesh3d(ring_mesh),
                MeshMaterial3d(ring_mat),
                Transform::from_xyz(0.0, 0.02, 0.0),
            ));
        }
    });
}

/// Synchronize visual transforms from the ECS `Position` component so the
/// rendered stickman follows the logical game position.
pub fn sync_player_visuals(
    mut query: Query<(&Position, &FacingDirection, &mut Transform), With<StickmanVisual>>,
) {
    for (pos, facing, mut transform) in &mut query {
        transform.translation = pos.0;
        // FacingDirection convention: 0 = +Z, PI/2 = +X (heading from +Z axis).
        // Bevy Y rotation: 0 = looking toward -Z. So rotate by PI - facing to flip.
        transform.rotation = Quat::from_rotation_y(-facing.0);
    }
}
