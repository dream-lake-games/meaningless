use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::menu::AppState;

const BG_Z: f32 = -100.0;
const SCROLL_SPEED: f32 = 20.0;

#[derive(Component)]
struct MenuBgLayer {
    tile_size: Vec2,
    offset: Vec2,
}

#[derive(Component)]
struct MenuBgTile;

fn spawn_menu_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("bgs/scifi.png");
    let tile_size = Vec2::new(640.0, 640.0);

    let layer_entity = commands
        .spawn((
            MenuBgLayer {
                tile_size,
                offset: Vec2::ZERO,
            },
            Transform::from_xyz(0.0, 0.0, BG_Z),
            Visibility::Inherited,
            DespawnOnExit::<AppState>::default(),
        ))
        .id();

    for y in -1..=1 {
        for x in -1..=1 {
            commands.spawn((
                MenuBgTile,
                Sprite::from_image(texture.clone()),
                Transform::from_xyz(x as f32 * tile_size.x, y as f32 * tile_size.y, 0.0),
                PIXEL_PERFECT_LAYERS,
                ChildOf(layer_entity),
            ));
        }
    }
}

fn update_menu_background(
    time: Res<Time>,
    mut layer_query: Query<(&mut MenuBgLayer, &Children)>,
    mut tile_query: Query<&mut Transform, With<MenuBgTile>>,
) {
    for (mut layer, children) in &mut layer_query {
        layer.offset.x += SCROLL_SPEED * time.delta_secs();
        layer.offset.y += SCROLL_SPEED * time.delta_secs();

        layer.offset.x %= layer.tile_size.x;
        layer.offset.y %= layer.tile_size.y;

        let base_x = -layer.offset.x;
        let base_y = -layer.offset.y;

        let mut idx = 0;
        for y in -1..=1 {
            for x in -1..=1 {
                if let Some(child) = children.get(idx) {
                    if let Ok(mut tile_tf) = tile_query.get_mut(*child) {
                        tile_tf.translation.x = (base_x + x as f32 * layer.tile_size.x).round();
                        tile_tf.translation.y = (base_y + y as f32 * layer.tile_size.y).round();
                    }
                }
                idx += 1;
            }
        }
    }
}

pub(crate) fn background_plugin_fn(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), spawn_menu_background)
        .add_systems(Update, update_menu_background.run_if(in_state(AppState::Menu)));
}
