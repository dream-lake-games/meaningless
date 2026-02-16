use bevy::prelude::*;

use crate::camera::{InGameCamera, HIGH_RES_LAYERS};
use crate::level_progress::LevelProgress;
use crate::menu::navigation::{MenuNavigation, MenuScreen};
use crate::menu::AppState;
use crate::palette;
use crate::player::Player;
use crate::WINDOW_SIZE;

const STEP_DURATION: f32 = 0.08;
const DARK_FRAMES_MIN: u32 = 3;

const FADE_COLORS: [Color; 4] = [
    Color::NONE,
    palette::LIGHT,
    palette::DARK,
    palette::BLACK,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum TransitionPhase {
    #[default]
    None,
    FadingOut { step: usize },
    Dark,
    FadingIn { step: usize },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TransitionTarget {
    StartLevel(usize),
    ReturnToMenu,
}

#[derive(Resource, Default)]
pub(crate) struct TransitionState {
    phase: TransitionPhase,
    step_timer: f32,
    dark_frames: u32,
    target: Option<TransitionTarget>,
    camera_snapped: bool,
}

impl TransitionState {
    pub(crate) fn is_active(&self) -> bool {
        self.phase != TransitionPhase::None
    }

    pub(crate) fn request(&mut self, target: TransitionTarget) {
        if self.phase != TransitionPhase::None {
            return;
        }
        self.phase = TransitionPhase::FadingOut { step: 0 };
        self.step_timer = 0.0;
        self.dark_frames = 0;
        self.target = Some(target);
        self.camera_snapped = false;
    }
}

#[derive(Component)]
struct TransitionOverlay;

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        TransitionOverlay,
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::splat(WINDOW_SIZE as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0),
        HIGH_RES_LAYERS,
    ));
}

fn update_transition(
    time: Res<Time>,
    mut state: ResMut<TransitionState>,
    mut overlay_q: Query<&mut Sprite, With<TransitionOverlay>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut progress: ResMut<LevelProgress>,
    mut nav: ResMut<MenuNavigation>,
    mut camera_q: Query<&mut Transform, With<InGameCamera>>,
    player_q: Query<&Transform, (With<Player>, Without<InGameCamera>)>,
) {
    if state.phase == TransitionPhase::None {
        return;
    }

    let Ok(mut overlay) = overlay_q.single_mut() else {
        return;
    };

    match state.phase {
        TransitionPhase::FadingOut { step } => {
            overlay.color = FADE_COLORS[step];

            state.step_timer += time.delta_secs();
            if state.step_timer >= STEP_DURATION {
                state.step_timer = 0.0;
                let next_step = step + 1;

                if next_step >= FADE_COLORS.len() {
                    state.phase = TransitionPhase::Dark;
                    state.dark_frames = 0;
                    overlay.color = palette::BLACK;

                    if let Some(target) = state.target {
                        match target {
                            TransitionTarget::StartLevel(level) => {
                                progress.current_playing = Some(level);
                                app_state.set(AppState::Playing);
                            }
                            TransitionTarget::ReturnToMenu => {
                                progress.current_playing = None;
                                nav.screen = MenuScreen::LevelSelect;
                                app_state.set(AppState::Menu);
                            }
                        }
                    }
                } else {
                    state.phase = TransitionPhase::FadingOut { step: next_step };
                }
            }
        }
        TransitionPhase::Dark => {
            overlay.color = palette::BLACK;
            state.dark_frames += 1;

            if !state.camera_snapped {
                if let Ok(player_tf) = player_q.single() {
                    if let Ok(mut camera_tf) = camera_q.single_mut() {
                        let target_pos = player_tf.translation.truncate();
                        camera_tf.translation.x = target_pos.x.round();
                        camera_tf.translation.y = target_pos.y.round();
                        state.camera_snapped = true;
                    }
                }
            }

            let ready_to_fade_in = state.dark_frames >= DARK_FRAMES_MIN && state.camera_snapped;
            let timeout = state.dark_frames >= 30;

            if ready_to_fade_in || timeout {
                if timeout && !state.camera_snapped {
                    if let Ok(mut camera_tf) = camera_q.single_mut() {
                        camera_tf.translation.x = 0.0;
                        camera_tf.translation.y = 0.0;
                    }
                }
                state.phase = TransitionPhase::FadingIn { step: FADE_COLORS.len() - 1 };
                state.step_timer = 0.0;
            }
        }
        TransitionPhase::FadingIn { step } => {
            overlay.color = FADE_COLORS[step];

            state.step_timer += time.delta_secs();
            if state.step_timer >= STEP_DURATION {
                state.step_timer = 0.0;

                if step == 0 {
                    state.phase = TransitionPhase::None;
                    state.target = None;
                    overlay.color = Color::NONE;
                } else {
                    state.phase = TransitionPhase::FadingIn { step: step - 1 };
                }
            }
        }
        TransitionPhase::None => {}
    }
}

fn handle_escape_during_play(
    keyboard: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut transition: ResMut<TransitionState>,
) {
    if *app_state.get() != AppState::Playing {
        return;
    }

    if transition.is_active() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        transition.request(TransitionTarget::ReturnToMenu);
    }
}

pub(crate) fn transition_plugin_fn(app: &mut App) {
    app.init_resource::<TransitionState>()
        .add_systems(Startup, spawn_overlay)
        .add_systems(
            Update,
            (
                handle_escape_during_play,
                update_transition,
            )
                .chain(),
        );
}
