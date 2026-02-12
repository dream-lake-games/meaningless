use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::*;

mod camera;
mod gol;
mod level;
mod player;

pub const WINDOW_SIZE: u32 = 640;
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
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(TnuaControllerPlugin::<player::PlayerScheme>::new(
            FixedPostUpdate,
        ))
        .add_plugins(TnuaAvian2dPlugin::new(FixedPostUpdate))
        .insert_resource(Gravity(Vec2::new(0.0, -800.0)))
        .add_plugins((
            camera::camera_plugin_fn,
            player::player_plugin_fn,
            level::level_plugin_fn,
            gol::gol_plugin_fn,
        ))
        .run();
}
