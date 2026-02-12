use bevy::prelude::*;

use crate::player::Player;

#[derive(Component)]
struct GameCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        GameCamera,
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
    ));
}

fn camera_follow_system(
    player_q: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera_q: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut camera_tf) = camera_q.single_mut() else {
        return;
    };

    camera_tf.translation.x = player_tf.translation.x;
    camera_tf.translation.y = player_tf.translation.y;
}

pub fn camera_plugin_fn(app: &mut App) {
    app.add_systems(Startup, spawn_camera).add_systems(
        PostUpdate,
        camera_follow_system.before(TransformSystems::Propagate),
    );
}
