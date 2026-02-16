use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::TILE_SIZE;
use crate::anim::AnimMan;
use crate::animations::{LeftCellAnim, RightCellAnim, SpikeAnim};
use crate::camera::HIGH_RES_LAYERS;
use crate::gol::{GHOST_ALPHA, SpawnPosition};
use crate::level_progress::LevelProgress;
use crate::menu::AppState;
use crate::menu::navigation::ControlScheme;
use crate::player::{Player, spawn_player};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LevelSystemSet;

#[derive(Component)]
pub(crate) struct PlayingLevelMarker;

#[derive(Component)]
struct ControlHints;

#[derive(Component)]
struct ControlHintsText;

#[derive(Component)]
struct ControlHintsBg;

#[derive(Resource, Default)]
pub(crate) struct HelpText {
    pub(crate) override_text: Option<String>,
}

#[derive(Component, Default)]
pub(crate) struct PermanentCell;

#[derive(Component, Default)]
pub(crate) struct DynamicCell;

#[derive(Component, Default)]
pub(crate) struct NeverCell;

#[derive(Component, Default)]
pub(crate) struct Spike;

#[derive(Component, Default)]
pub(crate) struct PlayerSpawn;

#[derive(Bundle, LdtkIntCell, Default)]
pub(crate) struct PermanentCellBundle {
    marker: PermanentCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
pub(crate) struct DynamicCellBundle {
    marker: DynamicCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
pub(crate) struct NeverCellBundle {
    marker: NeverCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
struct SpikeBundle {
    marker: Spike,
}

#[derive(Bundle, LdtkEntity, Default)]
pub(crate) struct PlayerSpawnBundle {
    marker: PlayerSpawn,
}

fn select_level_from_progress(
    progress: Res<LevelProgress>,
    mut level_selection: ResMut<LevelSelection>,
) {
    let level = progress.current_playing.unwrap_or(0);
    info!("Setting LevelSelection to level {}", level);
    *level_selection = LevelSelection::index(level);
}

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayingLevelMarker,
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("levels/play.ldtk").into(),
            ..default()
        },
    ));
}

fn add_permanent_cell_visuals(
    mut commands: Commands,
    query: Query<Entity, (Added<PermanentCell>, Without<RigidBody>)>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            AnimMan::new(LeftCellAnim::Locked),
            AnimMan::new(RightCellAnim::None).with_flip_x(true),
            Visibility::Inherited,
            RigidBody::Static,
            Collider::rectangle(TILE_SIZE, TILE_SIZE),
        ));
    }
}

fn add_never_cell_visuals(mut commands: Commands, query: Query<Entity, Added<NeverCell>>) {
    for entity in &query {
        commands.entity(entity).insert((
            AnimMan::new(LeftCellAnim::Never),
            AnimMan::new(RightCellAnim::None).with_flip_x(true),
            Visibility::Inherited,
        ));
    }
}

fn add_spike_visuals(mut commands: Commands, query: Query<Entity, Added<Spike>>) {
    for entity in &query {
        commands.entity(entity).insert((
            AnimMan::new(SpikeAnim::Spikes),
            Visibility::Inherited,
        ));
    }
}

fn sync_never_cell_opacity(
    cell_query: Query<&Children, With<NeverCell>>,
    mut sprite_query: Query<&mut Sprite>,
) {
    for children in &cell_query {
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_query.get_mut(child) {
                sprite.color = sprite.color.with_alpha(GHOST_ALPHA);
            }
        }
    }
}

fn spawn_player_at_spawn_point(
    mut commands: Commands,
    spawn_query: Query<&Transform, Added<PlayerSpawn>>,
    existing_players: Query<Entity, With<Player>>,
    mut spawn_pos: ResMut<SpawnPosition>,
) {
    for transform in &spawn_query {
        for player_entity in &existing_players {
            commands.entity(player_entity).despawn();
        }

        let pos = transform.translation.truncate();
        spawn_pos.0 = Some(pos);
        spawn_player(&mut commands, pos);
    }
}

fn spawn_control_hints(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    controls: Res<ControlScheme>,
    help_text: Res<HelpText>,
) {
    let default_text = match *controls {
        ControlScheme::Arrow => "ARROWS move, F forward, D back, R restart, ESC menu",
        ControlScheme::Wasd => "WASD move, K forward, J back, R restart, ESC menu",
    };

    let text = help_text.override_text.as_deref().unwrap_or(default_text);

    let font = asset_server.load("fonts/Tiny5-Regular.ttf");
    let y_pos = -290.0;
    let padding_x = 10.0;
    let padding_y = 6.0;
    let font_size = 20.0;
    let text_width = text.len() as f32 * (font_size * 0.55);
    let text_height = font_size;

    // Background rectangle - lightest color: rgb(0.863, 0.863, 0.863)
    let bg_color = Color::srgb(0.863, 0.863, 0.863);
    // Text - darkest color: rgb(0, 0, 0)
    let text_color = Color::srgb(0.0, 0.0, 0.0);

    commands.spawn((
        ControlHints,
        ControlHintsBg,
        Sprite {
            color: bg_color,
            custom_size: Some(Vec2::new(
                text_width + padding_x * 2.0,
                text_height + padding_y * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, y_pos, 998.0),
        HIGH_RES_LAYERS,
    ));

    commands.spawn((
        ControlHints,
        ControlHintsText,
        Text2d::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(text_color),
        Transform::from_xyz(0.0, y_pos, 999.0),
        HIGH_RES_LAYERS,
    ));
}

fn update_control_hints(
    controls: Res<ControlScheme>,
    help_text: Res<HelpText>,
    mut text_query: Query<&mut Text2d, With<ControlHintsText>>,
    mut bg_query: Query<&mut Sprite, With<ControlHintsBg>>,
) {
    let default_text = match *controls {
        ControlScheme::Arrow => "ARROWS move, F forward, D back, R restart, ESC menu",
        ControlScheme::Wasd => "WASD move, K forward, J back, R restart, ESC menu",
    };

    let text = help_text.override_text.as_deref().unwrap_or(default_text);

    for mut text2d in &mut text_query {
        if text2d.0 != text {
            text2d.0 = text.to_string();
        }
    }

    let font_size = 20.0;
    let padding_x = 10.0;
    let padding_y = 6.0;
    let text_width = text.len() as f32 * (font_size * 0.55);
    let text_height = font_size;
    let new_size = Vec2::new(text_width + padding_x * 2.0, text_height + padding_y * 2.0);

    for mut sprite in &mut bg_query {
        if sprite.custom_size != Some(new_size) {
            sprite.custom_size = Some(new_size);
        }
    }
}

fn cleanup_level(
    mut commands: Commands,
    level_query: Query<Entity, With<PlayingLevelMarker>>,
    player_query: Query<Entity, With<Player>>,
    hints_query: Query<Entity, With<ControlHints>>,
    mut spawn_pos: ResMut<SpawnPosition>,
) {
    for entity in &level_query {
        commands.entity(entity).despawn();
    }

    for entity in &player_query {
        commands.entity(entity).despawn();
    }

    for entity in &hints_query {
        commands.entity(entity).despawn();
    }

    spawn_pos.0 = None;
}

pub(crate) fn level_plugin_fn(app: &mut App) {
    app.add_plugins(LdtkPlugin)
        .init_resource::<HelpText>()
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            level_background: LevelBackground::Nonexistent,
            int_grid_rendering: IntGridRendering::Invisible,
            ..default()
        })
        .register_ldtk_int_cell::<PermanentCellBundle>(1)
        .register_ldtk_int_cell::<DynamicCellBundle>(2)
        .register_ldtk_int_cell::<NeverCellBundle>(3)
        .register_ldtk_int_cell_for_layer::<SpikeBundle>("Spikes", 1)
        .register_ldtk_entity::<PlayerSpawnBundle>("PlayerSpawn")
        .add_systems(
            OnEnter(AppState::Playing),
            (select_level_from_progress, setup_level, spawn_control_hints).chain(),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_level)
        .add_systems(
            Update,
            (
                add_permanent_cell_visuals,
                add_never_cell_visuals,
                add_spike_visuals,
                spawn_player_at_spawn_point,
                sync_never_cell_opacity,
                update_control_hints,
            )
                .chain()
                .in_set(LevelSystemSet)
                .run_if(in_state(AppState::Playing)),
        );
}
