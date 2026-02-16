use bevy::prelude::*;

use crate::anim::AnimMan;
use crate::animations::LevelSquareAnim;
use crate::camera::{HIGH_RES_LAYERS, PIXEL_PERFECT_LAYERS};
use crate::level_progress::{LevelProgress, LevelState, PlayLdtkHandle};
use crate::menu::navigation::{ControlScheme, MenuNavigation, MenuScreen};
use crate::menu::AppState;
use crate::palette;
use crate::sfx::{self, Sfx};
use crate::transition::{TransitionState, TransitionTarget};

const GRID_COLS: usize = 4;
const SQUARE_SIZE: f32 = 16.0;
const SQUARE_GAP: f32 = 8.0;
const GRID_SPACING: f32 = SQUARE_SIZE + SQUARE_GAP;
const LEVEL_SELECT_Z: f32 = 60.0;
const NAME_OFFSET_Y: f32 = 140.0;
const NAME_FONT_SIZE: f32 = 24.0;

fn level_to_grid(level: usize) -> (usize, usize) {
    (level % GRID_COLS, level / GRID_COLS)
}

fn grid_to_level(col: usize, row: usize) -> usize {
    row * GRID_COLS + col
}

#[derive(Component)]
struct LevelSquare {
    level: usize,
}

#[derive(Component)]
struct LevelSelectMarker;

#[derive(Component)]
struct LevelNameText;

#[derive(Resource, Default)]
struct LevelSelectState {
    spawned: bool,
    refreshed: bool,
    cooldown: f32,
}

fn refresh_level_data(
    asset_server: Res<AssetServer>,
    nav: Res<MenuNavigation>,
    mut state: ResMut<LevelSelectState>,
    mut progress: ResMut<LevelProgress>,
    mut handle: ResMut<PlayLdtkHandle>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        state.refreshed = false;
        return;
    }
    if state.refreshed {
        return;
    }
    state.refreshed = true;
    state.cooldown = 0.2;
    progress.total_levels = 0;
    handle.0 = Some(asset_server.load("levels/play.ldtk"));
}

fn spawn_level_squares(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    progress: Res<LevelProgress>,
    nav: Res<MenuNavigation>,
    mut state: ResMut<LevelSelectState>,
) {
    if nav.screen != MenuScreen::LevelSelect || state.spawned || progress.total_levels == 0 {
        return;
    }

    let grid_width = GRID_COLS as f32 * GRID_SPACING - SQUARE_GAP;
    let num_rows = (progress.total_levels + GRID_COLS - 1) / GRID_COLS;
    let grid_height = num_rows as f32 * GRID_SPACING - SQUARE_GAP;

    let offset_x = -grid_width / 2.0 + SQUARE_SIZE / 2.0;
    let offset_y = grid_height / 2.0 - SQUARE_SIZE / 2.0;

    for level in 0..progress.total_levels.min(20) {
        let (col, row) = level_to_grid(level);
        let world_pos = Vec3::new(
            offset_x + col as f32 * GRID_SPACING,
            offset_y - row as f32 * GRID_SPACING,
            LEVEL_SELECT_Z,
        );

        let initial_anim = get_anim_for_level(&progress, level, level == progress.selected);

        commands.spawn((
            LevelSelectMarker,
            LevelSquare { level },
            AnimMan::new(initial_anim),
            Transform::from_translation(world_pos),
            Visibility::Inherited,
            PIXEL_PERFECT_LAYERS,
        ));
    }

    let font = asset_server.load("fonts/Tiny5-Regular.ttf");
    let name = format!("\"{}\"", progress.get_level_name(progress.selected));
    commands.spawn((
        LevelSelectMarker,
        LevelNameText,
        Text2d::new(name),
        TextFont {
            font,
            font_size: NAME_FONT_SIZE,
            ..default()
        },
        TextColor(palette::DARK),
        Transform::from_xyz(0.0, NAME_OFFSET_Y, LEVEL_SELECT_Z),
        HIGH_RES_LAYERS,
    ));

    state.spawned = true;
}

fn despawn_level_squares(
    mut commands: Commands,
    nav: Res<MenuNavigation>,
    query: Query<Entity, With<LevelSelectMarker>>,
    mut state: ResMut<LevelSelectState>,
) {
    if nav.screen == MenuScreen::LevelSelect {
        return;
    }

    if !state.spawned {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }

    state.spawned = false;
}

fn update_level_squares(
    progress: Res<LevelProgress>,
    nav: Res<MenuNavigation>,
    mut square_query: Query<(&LevelSquare, &mut AnimMan<LevelSquareAnim>)>,
    mut name_query: Query<&mut Text2d, With<LevelNameText>>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        return;
    }

    for (square, mut anim) in &mut square_query {
        let is_selected = progress.selected == square.level;
        let target = get_anim_for_level(&progress, square.level, is_selected);

        if anim.get() != target {
            anim.set(target);
        }
    }

    for mut text in &mut name_query {
        let name = format!("\"{}\"", progress.get_level_name(progress.selected));
        if text.0 != name {
            text.0 = name;
        }
    }
}

fn get_anim_for_level(progress: &LevelProgress, level: usize, selected: bool) -> LevelSquareAnim {
    let state = progress.level_state(level);
    match (state, selected) {
        (LevelState::Locked, _) => LevelSquareAnim::Locked,
        (LevelState::Unlocked, false) => LevelSquareAnim::Unlocked,
        (LevelState::Unlocked, true) => LevelSquareAnim::UnlockedSelected,
        (LevelState::Done, false) => LevelSquareAnim::Done,
        (LevelState::Done, true) => LevelSquareAnim::DoneSelected,
    }
}

fn navigate_level_select(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlScheme>,
    nav: Res<MenuNavigation>,
    sfx: Res<Sfx>,
    mut progress: ResMut<LevelProgress>,
    mut state: ResMut<LevelSelectState>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        return;
    }

    state.cooldown -= time.delta_secs();
    if state.cooldown > 0.0 {
        return;
    }

    let (up, down, left, right) = match *controls {
        ControlScheme::Arrow => (
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ),
        ControlScheme::Wasd => (KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD),
    };

    let (col, row) = level_to_grid(progress.selected);

    let target = if keyboard.just_pressed(up) && row > 0 {
        Some(grid_to_level(col, row - 1))
    } else if keyboard.just_pressed(down) {
        Some(grid_to_level(col, row + 1))
    } else if keyboard.just_pressed(left) && col > 0 {
        Some(grid_to_level(col - 1, row))
    } else if keyboard.just_pressed(right) && col < GRID_COLS - 1 {
        Some(grid_to_level(col + 1, row))
    } else {
        None
    };

    if let Some(target_level) = target {
        if target_level < progress.total_levels && progress.is_unlocked(target_level) {
            progress.selected = target_level;
            state.cooldown = 0.15;
            sfx::play_menu_move(&mut commands, &sfx);
        }
    }
}

fn start_level(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    nav: Res<MenuNavigation>,
    progress: Res<LevelProgress>,
    state: Res<LevelSelectState>,
    sfx: Res<Sfx>,
    mut transition: ResMut<TransitionState>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        return;
    }

    if transition.is_active() || state.cooldown > 0.0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        let level = progress.selected;
        if progress.is_unlocked(level) {
            transition.request(TransitionTarget::StartLevel(level));
            sfx::play_menu_select(&mut commands, &sfx);
        }
    }
}

fn cleanup_on_menu_exit(
    mut commands: Commands,
    mut state: ResMut<LevelSelectState>,
    query: Query<Entity, With<LevelSelectMarker>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state.spawned = false;
}

pub(crate) fn level_select_plugin_fn(app: &mut App) {
    app.init_resource::<LevelSelectState>().add_systems(
        Update,
        (
            refresh_level_data,
            spawn_level_squares,
            despawn_level_squares,
            update_level_squares,
            navigate_level_select,
            start_level,
        )
            .chain()
            .run_if(in_state(AppState::Menu)),
    )
    .add_systems(OnExit(AppState::Menu), cleanup_on_menu_exit);
}
