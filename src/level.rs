use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::anim::AnimMan;
use crate::animations::CellAnim;
use crate::gol::{SpawnPosition, GHOST_ALPHA};
use crate::player::{spawn_player, Player};
use crate::TILE_SIZE;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LevelSystemSet;

#[derive(Component, Default)]
pub struct PermanentCell;

#[derive(Component, Default)]
pub struct DynamicCell;

#[derive(Component, Default)]
pub struct NeverCell;

#[derive(Component, Default)]
pub struct PlayerSpawn;

#[derive(Bundle, LdtkIntCell, Default)]
pub struct PermanentCellBundle {
    marker: PermanentCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
pub struct DynamicCellBundle {
    marker: DynamicCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
pub struct NeverCellBundle {
    marker: NeverCell,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct PlayerSpawnBundle {
    marker: PlayerSpawn,
}

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/play.ldtk").into(),
        ..default()
    });
}

fn add_permanent_cell_visuals(
    mut commands: Commands,
    query: Query<Entity, (Added<PermanentCell>, Without<RigidBody>)>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            AnimMan::new(CellAnim::Locked),
            Visibility::Inherited,
            RigidBody::Static,
            Collider::rectangle(TILE_SIZE, TILE_SIZE),
        ));
    }
}

fn add_never_cell_visuals(
    mut commands: Commands,
    query: Query<Entity, Added<NeverCell>>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            AnimMan::new(CellAnim::Never),
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
        info!("Player spawned at ({}, {})", pos.x, pos.y);
    }
}

pub fn level_plugin_fn(app: &mut App) {
    app.add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            level_background: LevelBackground::Nonexistent,
            int_grid_rendering: IntGridRendering::Invisible,
            ..default()
        })
        .register_ldtk_int_cell::<PermanentCellBundle>(1)
        .register_ldtk_int_cell::<DynamicCellBundle>(2)
        .register_ldtk_int_cell::<NeverCellBundle>(3)
        .register_ldtk_entity::<PlayerSpawnBundle>("PlayerSpawn")
        .add_systems(Startup, setup_level)
        .add_systems(
            Update,
            (
                add_permanent_cell_visuals,
                add_never_cell_visuals,
                spawn_player_at_spawn_point,
                sync_never_cell_opacity,
            )
                .chain()
                .in_set(LevelSystemSet),
        );
}
