use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::anim::AnimMan;
use crate::animations::{FlagAnim, FlagIndicatorAnim, PlayerAnim};
use crate::camera::{InGameCamera, HIGH_RES_LAYERS, PIXEL_PERFECT_LAYERS};
use crate::gol::{DeathPhase, DeathRewind};
use crate::level_progress::LevelProgress;
use crate::menu::AppState;
use crate::player::{Player, PlayerState};
use crate::transition::{TransitionState, TransitionTarget};

const COLLECT_WIDTH: f32 = 8.0;
const COLLECT_HEIGHT: f32 = 32.0;
const HALF_VIEW: f32 = 160.0;
const INDICATOR_INSET: f32 = 12.0;
const INDICATOR_Z: f32 = 100.0;

#[derive(Component, Default)]
pub(crate) struct Flag;

#[derive(Component)]
struct FlagCollected;

#[derive(Component)]
struct FlagIndicator {
    flag_entity: Entity,
}

#[derive(Bundle, LdtkEntity, Default)]
struct FlagBundle {
    marker: Flag,
}

#[derive(Resource, Default)]
struct FlagCounter {
    collected: usize,
    total: usize,
    complete_timer: Option<f32>,
}

const COMPLETE_DELAY: f32 = 0.3;

#[derive(Component)]
struct FlagCounterText;

#[derive(Component)]
struct LevelNameText;

fn setup_flag_visuals(mut commands: Commands, query: Query<Entity, Added<Flag>>) {
    for entity in &query {
        commands
            .entity(entity)
            .insert((AnimMan::new(FlagAnim::Wave), Visibility::Inherited));

        commands.spawn((
            FlagIndicator { flag_entity: entity },
            AnimMan::new(FlagIndicatorAnim::Indicator),
            Transform::from_xyz(0.0, 0.0, INDICATOR_Z),
            Visibility::Hidden,
            PIXEL_PERFECT_LAYERS,
        ));
    }
}

fn count_flags(mut counter: ResMut<FlagCounter>, query: Query<(), With<Flag>>) {
    let total = query.iter().count();
    if counter.total != total {
        counter.total = total;
    }
}

fn collect_flags(
    mut commands: Commands,
    mut counter: ResMut<FlagCounter>,
    mut player_query: Query<
        (&Transform, &mut PlayerState, &mut AnimMan<PlayerAnim>),
        With<Player>,
    >,
    mut flag_query: Query<
        (Entity, &Transform, &mut AnimMan<FlagAnim>),
        (With<Flag>, Without<FlagCollected>),
    >,
) {
    let Ok((player_tf, mut player_state, mut player_anim)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();

    for (entity, flag_tf, mut anim) in &mut flag_query {
        let flag_pos = flag_tf.translation.truncate();
        let diff = player_pos - flag_pos;

        if diff.x.abs() < COLLECT_WIDTH / 2.0 && diff.y.abs() < COLLECT_HEIGHT / 2.0 {
            commands.entity(entity).insert(FlagCollected);
            anim.set(FlagAnim::Bare);
            counter.collected += 1;

            player_state.vx = 0.0;
            player_state.vy = 0.0;
            player_anim.set(PlayerAnim::Idle);

            if counter.collected == counter.total {
                counter.complete_timer = Some(COMPLETE_DELAY);
            }
        }
    }
}

fn check_level_complete(
    time: Res<Time>,
    mut counter: ResMut<FlagCounter>,
    death_rewind: Res<DeathRewind>,
    mut transition: ResMut<TransitionState>,
    mut progress: ResMut<LevelProgress>,
) {
    if death_rewind.phase != DeathPhase::None {
        return;
    }

    if transition.is_active() {
        return;
    }

    let Some(ref mut timer) = counter.complete_timer else {
        return;
    };

    *timer -= time.delta_secs();
    
    if *timer <= 0.0 {
        counter.complete_timer = None;
        if let Some(level) = progress.current_playing {
            progress.complete_level(level);
            progress.selected = level;
        }
        transition.request(TransitionTarget::ReturnToMenu);
    }
}

fn reset_flags_on_death(
    mut commands: Commands,
    death_rewind: Res<DeathRewind>,
    mut counter: ResMut<FlagCounter>,
    mut flag_query: Query<(Entity, &mut AnimMan<FlagAnim>), With<FlagCollected>>,
) {
    if death_rewind.phase != DeathPhase::Rewinding {
        return;
    }

    if counter.collected == 0 {
        return;
    }

    for (entity, mut anim) in &mut flag_query {
        commands.entity(entity).remove::<FlagCollected>();
        anim.set(FlagAnim::Wave);
    }
    counter.collected = 0;
    counter.complete_timer = None;
}

fn spawn_counter_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    progress: Res<LevelProgress>,
) {
    let font = asset_server.load("fonts/Tiny5-Regular.ttf");

    let level_name = progress
        .current_playing
        .map(|l| progress.get_level_name(l))
        .unwrap_or("???");

    commands.spawn((
        LevelNameText,
        Text2d::new(level_name),
        TextFont {
            font: font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-310.0, 310.0, 995.0),
        HIGH_RES_LAYERS,
    ));

    commands.spawn((
        FlagCounterText,
        Text2d::new("0/0 flags"),
        TextFont {
            font,
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        bevy::sprite::Anchor::TOP_RIGHT,
        Transform::from_xyz(310.0, 310.0, 995.0),
        HIGH_RES_LAYERS,
    ));
}

fn update_counter_ui(
    counter: Res<FlagCounter>,
    mut text_query: Query<&mut Text2d, With<FlagCounterText>>,
) {
    for mut text in &mut text_query {
        let new_text = format!("{}/{} flags", counter.collected, counter.total);
        if text.0 != new_text {
            text.0 = new_text;
        }
    }
}

fn despawn_counter_ui(
    mut commands: Commands,
    counter_query: Query<Entity, With<FlagCounterText>>,
    name_query: Query<Entity, With<LevelNameText>>,
) {
    for entity in &counter_query {
        commands.entity(entity).despawn();
    }
    for entity in &name_query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_flags(mut counter: ResMut<FlagCounter>) {
    counter.collected = 0;
    counter.total = 0;
    counter.complete_timer = None;
}

fn update_flag_indicators(
    camera_query: Query<&Transform, With<InGameCamera>>,
    flag_query: Query<(Entity, &Transform), (With<Flag>, Without<FlagCollected>)>,
    collected_flags: Query<Entity, (With<Flag>, With<FlagCollected>)>,
    mut indicator_query: Query<(&FlagIndicator, &mut Transform, &mut Visibility), (Without<InGameCamera>, Without<Flag>)>,
) {
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_tf.translation.truncate();

    let collected_set: std::collections::HashSet<Entity> = collected_flags.iter().collect();
    let flag_positions: std::collections::HashMap<Entity, Vec2> = flag_query
        .iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .collect();

    for (indicator, mut transform, mut visibility) in &mut indicator_query {
        let flag_entity = indicator.flag_entity;

        if collected_set.contains(&flag_entity) {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some(&flag_pos) = flag_positions.get(&flag_entity) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = flag_pos - camera_pos;
        let bound = HALF_VIEW - INDICATOR_INSET;

        let is_offscreen = relative.x.abs() > HALF_VIEW || relative.y.abs() > HALF_VIEW;

        if !is_offscreen {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        let clamped = Vec2::new(
            relative.x.clamp(-bound, bound),
            relative.y.clamp(-bound, bound),
        );

        transform.translation.x = (camera_pos.x + clamped.x).round();
        transform.translation.y = (camera_pos.y + clamped.y).round();
    }
}

fn despawn_indicators(
    mut commands: Commands,
    query: Query<Entity, With<FlagIndicator>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn flag_plugin_fn(app: &mut App) {
    app.init_resource::<FlagCounter>()
        .register_ldtk_entity::<FlagBundle>("Flag")
        .add_systems(OnEnter(AppState::Playing), spawn_counter_ui)
        .add_systems(OnExit(AppState::Playing), (despawn_counter_ui, despawn_indicators, cleanup_flags))
        .add_systems(
            Update,
            (
                setup_flag_visuals,
                count_flags,
                collect_flags,
                reset_flags_on_death,
                check_level_complete,
                update_counter_ui,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            PostUpdate,
            update_flag_indicators
                .run_if(in_state(AppState::Playing)),
        );
}
