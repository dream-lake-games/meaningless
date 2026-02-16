use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::menu::AppState;
use crate::sfx::{self, Sfx};

const TRANSITION_COOLDOWN: f32 = 0.3;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Resource)]
pub(crate) enum ControlScheme {
    #[default]
    Arrow,
    Wasd,
}

impl ControlScheme {
    fn toggle(self) -> Self {
        match self {
            ControlScheme::Arrow => ControlScheme::Wasd,
            ControlScheme::Wasd => ControlScheme::Arrow,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum MenuScreen {
    #[default]
    Splash,
    Controls,
    LevelSelect,
}

impl MenuScreen {
    fn level_index(self) -> usize {
        match self {
            MenuScreen::Splash => 0,
            MenuScreen::Controls => 1,
            MenuScreen::LevelSelect => 2,
        }
    }

    fn next(self) -> Option<MenuScreen> {
        match self {
            MenuScreen::Splash => Some(MenuScreen::Controls),
            MenuScreen::Controls => Some(MenuScreen::LevelSelect),
            MenuScreen::LevelSelect => None,
        }
    }

    fn prev(self) -> Option<MenuScreen> {
        match self {
            MenuScreen::Splash => None,
            MenuScreen::Controls => Some(MenuScreen::Splash),
            MenuScreen::LevelSelect => Some(MenuScreen::Controls),
        }
    }
}

#[derive(Resource)]
pub(crate) struct MenuNavigation {
    pub(crate) screen: MenuScreen,
    cooldown_remaining: f32,
    waiting_for_release: bool,
}

impl Default for MenuNavigation {
    fn default() -> Self {
        Self {
            screen: MenuScreen::default(),
            cooldown_remaining: 0.0,
            waiting_for_release: true,
        }
    }
}

fn navigate_menu(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    sfx: Res<Sfx>,
    mut nav: ResMut<MenuNavigation>,
    mut level_selection: ResMut<LevelSelection>,
    mut controls: ResMut<ControlScheme>,
) {
    if nav.cooldown_remaining > 0.0 {
        nav.cooldown_remaining -= time.delta_secs();
    }

    let any_key_pressed = keyboard.get_just_pressed().next().is_some();
    let any_key_held = keyboard.get_pressed().next().is_some();

    if nav.waiting_for_release {
        if !any_key_held {
            nav.waiting_for_release = false;
        }
        return;
    }

    if nav.cooldown_remaining > 0.0 {
        return;
    }

    if !any_key_pressed {
        return;
    }

    let escape_pressed = keyboard.just_pressed(KeyCode::Escape);
    let enter_pressed = keyboard.just_pressed(KeyCode::Enter);
    let tab_pressed = keyboard.just_pressed(KeyCode::Tab);
    let right_pressed = keyboard.just_pressed(KeyCode::ArrowRight);
    let a_pressed = keyboard.just_pressed(KeyCode::KeyA);
    let any_non_escape_pressed = keyboard.get_just_pressed().any(|k| *k != KeyCode::Escape);

    if nav.screen == MenuScreen::Controls {
        let should_toggle = tab_pressed
            || (right_pressed && *controls == ControlScheme::Arrow)
            || (a_pressed && *controls == ControlScheme::Wasd);

        if should_toggle {
            *controls = controls.toggle();
            nav.cooldown_remaining = TRANSITION_COOLDOWN;
            nav.waiting_for_release = true;
            sfx::play_menu_move(&mut commands, &sfx);
            return;
        }
    }

    let new_screen = match nav.screen {
        MenuScreen::Splash => {
            if any_non_escape_pressed {
                nav.screen.next()
            } else {
                None
            }
        }
        MenuScreen::Controls | MenuScreen::LevelSelect => {
            if escape_pressed {
                nav.screen.prev()
            } else if enter_pressed {
                nav.screen.next()
            } else {
                None
            }
        }
    };

    if let Some(screen) = new_screen {
        nav.screen = screen;
        nav.cooldown_remaining = TRANSITION_COOLDOWN;
        nav.waiting_for_release = true;
        *level_selection = LevelSelection::index(screen.level_index());
        sfx::play_menu_select(&mut commands, &sfx);
    }
}

fn reset_navigation(mut nav: ResMut<MenuNavigation>, mut level_selection: ResMut<LevelSelection>) {
    nav.cooldown_remaining = 0.0;
    nav.waiting_for_release = true;
    *level_selection = LevelSelection::index(nav.screen.level_index());
}

pub(crate) fn navigation_plugin_fn(app: &mut App) {
    app.init_resource::<ControlScheme>()
        .init_resource::<MenuNavigation>()
        .add_systems(OnEnter(AppState::Menu), reset_navigation)
        .add_systems(Update, navigate_menu.run_if(in_state(AppState::Menu)));
}
