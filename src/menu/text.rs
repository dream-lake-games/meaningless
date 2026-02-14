use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use bevy_ecs_ldtk::prelude::*;

use crate::camera::HIGH_RES_LAYERS;
use crate::menu::AppState;
use crate::palette;

const MENU_LEVEL_SIZE: f32 = 320.0;
const FONT_SIZE: f32 = 24.0;
const TEXT_Z: f32 = 100.0;

#[derive(Component, Default)]
struct MenuText;

#[derive(Component, Default, Reflect)]
struct TextContent {
    #[allow(dead_code)]
    text: String,
}

impl From<&EntityInstance> for TextContent {
    fn from(entity: &EntityInstance) -> Self {
        let text = entity
            .get_string_field("text")
            .map(|s| s.to_string())
            .unwrap_or_default();
        Self { text }
    }
}

#[derive(Bundle, LdtkEntity, Default)]
struct MenuTextBundle {
    marker: MenuText,
    #[from_entity_instance]
    content: TextContent,
}

#[derive(Resource, Default)]
struct MenuTextState {
    font: Option<Handle<Font>>,
}

fn setup_font(mut state: ResMut<MenuTextState>, asset_server: Res<AssetServer>) {
    state.font = Some(asset_server.load("fonts/Tiny5-Regular.ttf"));
}

fn spawn_text_entities(
    mut commands: Commands,
    state: Res<MenuTextState>,
    query: Query<(Entity, &TextContent, &Transform), Added<MenuText>>,
) {
    let Some(font) = state.font.clone() else {
        return;
    };

    for (entity, content, transform) in &query {
        let ldtk_pos = transform.translation.truncate();
        let high_res_pos = ldtk_to_highres(ldtk_pos);

        commands.entity(entity).despawn();

        commands.spawn((
            Text2d::new(&content.text),
            TextFont {
                font: font.clone(),
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(palette::BLACK),
            Transform::from_xyz(high_res_pos.x, high_res_pos.y, TEXT_Z),
            HIGH_RES_LAYERS,
            DespawnOnExit::<AppState>::default(),
        ));
    }
}

fn ldtk_to_highres(ldtk_pos: Vec2) -> Vec2 {
    let half = MENU_LEVEL_SIZE / 2.0;
    let centered_x = ldtk_pos.x - half;
    let centered_y = ldtk_pos.y - half;
    Vec2::new(centered_x * 2.0, centered_y * 2.0)
}

pub(crate) fn text_plugin_fn(app: &mut App) {
    app.init_resource::<MenuTextState>()
        .register_type::<TextContent>()
        .register_ldtk_entity::<MenuTextBundle>("Text")
        .add_systems(OnEnter(AppState::Menu), setup_font)
        .add_systems(Update, spawn_text_entities.run_if(in_state(AppState::Menu)));
}
