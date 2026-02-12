use avian2d::prelude::*;
use bevy::prelude::*;

use crate::TILE_SIZE;

const GROUND_WIDTH: f32 = TILE_SIZE * 20.0;
const GROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
const PLATFORM_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

fn spawn_hardcoded_level(mut commands: Commands) {
    // Ground
    commands.spawn((
        Name::new("Ground"),
        Sprite {
            color: GROUND_COLOR,
            custom_size: Some(Vec2::new(GROUND_WIDTH, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, -TILE_SIZE * 4.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(GROUND_WIDTH, TILE_SIZE),
    ));

    let platforms = [
        Vec2::new(-TILE_SIZE * 3.0, -TILE_SIZE * 2.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(TILE_SIZE * 3.0, TILE_SIZE * 2.0),
        Vec2::new(TILE_SIZE * 6.0, TILE_SIZE * 2.0),
        Vec2::new(-TILE_SIZE * 6.0, TILE_SIZE * 1.0),
    ];

    for (i, pos) in platforms.iter().enumerate() {
        let width = TILE_SIZE * 3.0;
        commands.spawn((
            Name::new(format!("Platform {}", i)),
            Sprite {
                color: PLATFORM_COLOR,
                custom_size: Some(Vec2::new(width, TILE_SIZE)),
                ..default()
            },
            Transform::from_translation(pos.extend(0.0)),
            RigidBody::Static,
            Collider::rectangle(width, TILE_SIZE),
        ));
    }

    info!(
        "Hardcoded level spawned: ground + {} platforms",
        platforms.len()
    );
}

pub fn level_plugin_fn(app: &mut App) {
    app.add_systems(Startup, spawn_hardcoded_level);
}
