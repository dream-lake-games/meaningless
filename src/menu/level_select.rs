use bevy::prelude::*;

use crate::anim::AnimMan;
use crate::animations::LevelSquareAnim;
use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::level_progress::{LevelProgress, LevelState};
use crate::menu::navigation::{ControlScheme, MenuNavigation, MenuScreen};
use crate::menu::AppState;
use crate::spiral::level_to_pos;
use crate::transition::{TransitionState, TransitionTarget};

const GRID_SPACING: f32 = 24.0; // 16px square + 8px gap
const LEVEL_SELECT_Z: f32 = 60.0;

#[derive(Component)]
struct LevelSquare {
    level: usize,
}

#[derive(Component)]
struct LevelSelectMarker;

#[derive(Resource, Default)]
struct LevelSelectState {
    spawned: bool,
    cooldown: f32,
}

fn spawn_level_squares(
    mut commands: Commands,
    progress: Res<LevelProgress>,
    nav: Res<MenuNavigation>,
    mut state: ResMut<LevelSelectState>,
) {
    if nav.screen != MenuScreen::LevelSelect || state.spawned || progress.total_levels == 0 {
        return;
    }

    for level in 0..progress.total_levels {
        let pos = level_to_pos(level);
        let world_pos = Vec3::new(
            pos.x as f32 * GRID_SPACING,
            pos.y as f32 * GRID_SPACING,
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
    mut query: Query<(&LevelSquare, &mut AnimMan<LevelSquareAnim>)>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        return;
    }

    for (square, mut anim) in &mut query {
        let is_selected = progress.selected == square.level;
        let target = get_anim_for_level(&progress, square.level, is_selected);

        if anim.get() != target {
            anim.set(target);
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
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlScheme>,
    nav: Res<MenuNavigation>,
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

    let dir = if keyboard.just_pressed(up) {
        Some(IVec2::Y)
    } else if keyboard.just_pressed(down) {
        Some(IVec2::NEG_Y)
    } else if keyboard.just_pressed(left) {
        Some(IVec2::NEG_X)
    } else if keyboard.just_pressed(right) {
        Some(IVec2::X)
    } else {
        None
    };

    if let Some(dir) = dir {
        let current_pos = level_to_pos(progress.selected);
        let target_pos = current_pos + dir;

        if let Some(target_level) = crate::spiral::pos_to_level(target_pos) {
            if progress.is_unlocked(target_level) {
                progress.selected = target_level;
                state.cooldown = 0.15;
            }
        }
    }
}

fn start_level(
    keyboard: Res<ButtonInput<KeyCode>>,
    nav: Res<MenuNavigation>,
    progress: Res<LevelProgress>,
    mut transition: ResMut<TransitionState>,
) {
    if nav.screen != MenuScreen::LevelSelect {
        return;
    }

    if transition.is_active() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        let level = progress.selected;
        if progress.is_unlocked(level) {
            transition.request(TransitionTarget::StartLevel(level));
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
            spawn_level_squares,
            despawn_level_squares,
            update_level_squares,
            navigate_level_select,
            start_level,
        )
            .run_if(in_state(AppState::Menu)),
    )
    .add_systems(OnExit(AppState::Menu), cleanup_on_menu_exit);
}
