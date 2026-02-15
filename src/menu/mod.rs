mod background;
mod cells;
mod controls_sprite;
mod level_select;
pub(crate) mod navigation;
mod text;

use bevy::prelude::*;

#[allow(unused)]
pub(crate) use navigation::ControlScheme;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum AppState {
    #[default]
    Menu,
    Playing,
}

pub(crate) fn menu_plugin_fn(app: &mut App) {
    app.init_state::<AppState>()
        .add_plugins(background::background_plugin_fn)
        .add_plugins(cells::cells_plugin_fn)
        .add_plugins(controls_sprite::controls_sprite_plugin_fn)
        .add_plugins(level_select::level_select_plugin_fn)
        .add_plugins(navigation::navigation_plugin_fn)
        .add_plugins(text::text_plugin_fn);
}
