mod background;
mod cells;
mod text;

use bevy::prelude::*;

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
        .add_plugins(text::text_plugin_fn);
}
