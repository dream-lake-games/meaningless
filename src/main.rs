use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

mod anim;
mod animations;
mod bg;
mod camera;
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
// DEV HACK: Uncomment the line below to skip menu and go straight to level 0
// ============================================================================
const DEV_SKIP_MENU: bool = true;
// const DEV_SKIP_MENU: bool = false;

fn dev_skip_to_level(
    mut controls: ResMut<menu::navigation::ControlScheme>,
    mut progress: ResMut<level_progress::LevelProgress>,
    mut next_state: ResMut<NextState<menu::AppState>>,
) {
    if !DEV_SKIP_MENU {
        return;
    }
    *controls = menu::navigation::ControlScheme::Wasd;
    progress.current_playing = Some(0);
    next_state.set(menu::AppState::Playing);
    info!("DEV: Skipping menu, starting level 0 with WASD controls");
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
        transition::transition_plugin_fn,
    ));

    // DEV HACK: Skip menu system
    if DEV_SKIP_MENU {
        app.add_systems(Startup, dev_skip_to_level);
    }

    app.run();
}
