use bevy::prelude::*;
use bevy::window::WindowResolution;

mod camera;
mod gol;
mod level;
mod player;

pub const INTERNAL_SIZE: u32 = 640;
pub const WINDOW_SIZE: u32 = INTERNAL_SIZE;
pub const TILE_SIZE: f32 = 32.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Meaningless".to_string(),
                        resolution: WindowResolution::new(WINDOW_SIZE, WINDOW_SIZE),
                        fit_canvas_to_parent: false,
                        prevent_default_event_handling: true,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(avian2d::prelude::PhysicsPlugins::default())
        .add_plugins((
            camera::camera_plugin_fn,
            player::player_plugin_fn,
            level::level_plugin_fn,
            gol::gol_plugin_fn,
        ))
        .run();
}
