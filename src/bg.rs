use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::{InGameCamera, PIXEL_PERFECT_LAYERS};
use crate::menu::AppState;

const BG_Z: f32 = -100.0;

#[derive(Component)]
pub(crate) struct BgLayer {
    pub(crate) parallax: f32,
    pub(crate) tile_size: Vec2,
}

#[derive(Component)]
struct BgTile;

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("bgs/scifi.png");
    let tile_size = Vec2::new(640.0, 640.0);
    let parallax = 0.5;

    let layer_entity = commands
        .spawn((
            BgLayer { parallax, tile_size },
            Transform::from_xyz(0.0, 0.0, BG_Z),
            Visibility::Inherited,
            DespawnOnExit::<AppState>::default(),
        ))
        .id();

    for y in -1..=1 {
        for x in -1..=1 {
            commands.spawn((
                BgTile,
                Sprite::from_image(texture.clone()),
                Transform::from_xyz(x as f32 * tile_size.x, y as f32 * tile_size.y, 0.0),
                PIXEL_PERFECT_LAYERS,
                ChildOf(layer_entity),
            ));
        }
    }
}

fn update_background(
    camera_query: Query<&Transform, With<InGameCamera>>,
    mut layer_query: Query<(&BgLayer, &mut Transform, &Children), Without<InGameCamera>>,
    mut tile_query: Query<&mut Transform, (With<BgTile>, Without<InGameCamera>, Without<BgLayer>)>,
) {
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_tf.translation.truncate();

    for (layer, mut layer_tf, children) in &mut layer_query {
        let layer_offset = camera_pos * (1.0 - layer.parallax);
        layer_tf.translation.x = layer_offset.x.round();
        layer_tf.translation.y = layer_offset.y.round();

        let camera_local = camera_pos * layer.parallax;
        let base_x = (camera_local.x / layer.tile_size.x).floor() * layer.tile_size.x;
        let base_y = (camera_local.y / layer.tile_size.y).floor() * layer.tile_size.y;

        let mut idx = 0;
        for y in -1..=1 {
            for x in -1..=1 {
                if let Some(child) = children.get(idx) {
                    if let Ok(mut tile_tf) = tile_query.get_mut(*child) {
                        tile_tf.translation.x = base_x + x as f32 * layer.tile_size.x;
                        tile_tf.translation.y = base_y + y as f32 * layer.tile_size.y;
                    }
                }
                idx += 1;
            }
        }
    }
}

pub(crate) fn bg_plugin_fn(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), spawn_background)
        .add_systems(
            PostUpdate,
            update_background.run_if(in_state(AppState::Playing)),
        );
}
