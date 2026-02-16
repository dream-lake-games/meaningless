use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

mod anim;
mod animations;
mod bg;
mod camera;
mod flag;
mod gol;
mod level;
mod level_progress;
mod menu;
pub(crate) mod palette;
mod player;
mod sign;
mod spiral;
mod transition;

pub(crate) const INTERNAL_SIZE: u32 = 320;
pub(crate) const WINDOW_SIZE: u32 = INTERNAL_SIZE * 2;
pub(crate) const TILE_SIZE: f32 = 32.0;

// ============================================================================
// DEV HACK: Uncomment the line below to skip to level select with WASD
// ============================================================================
const DEV_SKIP_TO_LEVEL_SELECT: bool = true;
// const DEV_SKIP_TO_LEVEL_SELECT: bool = false;

fn dev_skip_to_level_select(
    mut controls: ResMut<menu::navigation::ControlScheme>,
    mut nav: ResMut<menu::navigation::MenuNavigation>,
    mut level_selection: ResMut<bevy_ecs_ldtk::prelude::LevelSelection>,
) {
    if !DEV_SKIP_TO_LEVEL_SELECT {
        return;
    }
    *controls = menu::navigation::ControlScheme::Wasd;
    nav.screen = menu::navigation::MenuScreen::LevelSelect;
    *level_selection = bevy_ecs_ldtk::prelude::LevelSelection::index(2);
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins(
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
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(avian2d::prelude::PhysicsPlugins::default())
    .add_plugins(anim::AnimPlugin::default())
    .add_plugins((
        level_progress::level_progress_plugin_fn,
        menu::menu_plugin_fn,
        bg::bg_plugin_fn,
        camera::camera_plugin_fn,
        player::player_plugin_fn,
        level::level_plugin_fn,
        gol::gol_plugin_fn,
        sign::sign_plugin_fn,
        flag::flag_plugin_fn,
        transition::transition_plugin_fn,
    ));

    if DEV_SKIP_TO_LEVEL_SELECT {
        app.add_systems(Startup, dev_skip_to_level_select);
    }

    app.run();
}
