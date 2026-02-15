use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::camera::HIGH_RES_LAYERS;
use crate::menu::AppState;
use crate::palette;

const MENU_LEVEL_SIZE: f32 = 320.0;
const FONT_SIZE: f32 = 24.0;
const TEXT_Z: f32 = 100.0;

#[derive(Component, Default)]
struct MenuText;

#[derive(Component)]
struct SpawnedMenuText;

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
    prev_level: Option<usize>,
}

fn setup_font(mut state: ResMut<MenuTextState>, asset_server: Res<AssetServer>) {
    state.font = Some(asset_server.load("fonts/Tiny5-Regular.ttf"));
}

fn handle_level_change(
    mut commands: Commands,
    mut state: ResMut<MenuTextState>,
    level_selection: Res<LevelSelection>,
    text_query: Query<Entity, With<SpawnedMenuText>>,
) {
    let current_level = match &*level_selection {
        LevelSelection::Indices(indices) => Some(indices.level),
        _ => None,
    };

    if state.prev_level == current_level {
        return;
    }

    info!(
        "Text: level changed {:?} -> {:?}, despawning {} text entities",
        state.prev_level,
        current_level,
        text_query.iter().count()
    );

    for entity in &text_query {
        commands.entity(entity).despawn();
    }

    state.prev_level = current_level;
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
            SpawnedMenuText,
            Text2d::new(&content.text),
            TextFont {
                font: font.clone(),
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(palette::BLACK),
            Transform::from_xyz(high_res_pos.x, high_res_pos.y, TEXT_Z),
            HIGH_RES_LAYERS,
        ));
    }
}

fn ldtk_to_highres(ldtk_pos: Vec2) -> Vec2 {
    let half = MENU_LEVEL_SIZE / 2.0;
    let centered_x = ldtk_pos.x - half;
    let centered_y = ldtk_pos.y - half;
    Vec2::new(centered_x * 2.0, centered_y * 2.0)
}

fn cleanup_text(
    mut commands: Commands,
    mut state: ResMut<MenuTextState>,
    text_query: Query<Entity, With<SpawnedMenuText>>,
) {
    for entity in &text_query {
        commands.entity(entity).despawn();
    }
    state.prev_level = None;
}

pub(crate) fn text_plugin_fn(app: &mut App) {
    app.init_resource::<MenuTextState>()
        .register_type::<TextContent>()
        .register_ldtk_entity::<MenuTextBundle>("Text")
        .add_systems(OnEnter(AppState::Menu), setup_font)
        .add_systems(OnExit(AppState::Menu), cleanup_text)
        .add_systems(
            Update,
            (handle_level_change, spawn_text_entities)
                .chain()
                .run_if(in_state(AppState::Menu)),
        );
}
