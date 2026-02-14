use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use bevy_ecs_ldtk::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::menu::AppState;
use crate::palette;

const MENU_TILE_SIZE: f32 = 8.0;
const MENU_LEVEL_SIZE: f32 = 320.0;
const CELL_Z: f32 = 10.0;
const TICK_INTERVAL: f32 = 0.3;

#[derive(Component, Default)]
pub(crate) struct MenuPermanentCell;

#[derive(Component, Default)]
pub(crate) struct MenuDynamicCell;

#[derive(Bundle, LdtkIntCell, Default)]
pub(crate) struct MenuPermanentCellBundle {
    marker: MenuPermanentCell,
}

#[derive(Bundle, LdtkIntCell, Default)]
pub(crate) struct MenuDynamicCellBundle {
    marker: MenuDynamicCell,
}

#[derive(Component)]
struct MenuCellSprite;

#[derive(Resource, Default)]
struct MenuCellState {
    permanent: HashSet<IVec2>,
    current: HashSet<IVec2>,
    entities: HashMap<IVec2, Entity>,
    level_entity: Option<Entity>,
    dynamic_texture: Option<Handle<Image>>,
    initialized: bool,
}

#[derive(Resource)]
struct MenuTickTimer(Timer);

impl Default for MenuTickTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(TICK_INTERVAL, TimerMode::Repeating))
    }
}

impl MenuCellState {
    fn is_alive(&self, pos: IVec2) -> bool {
        self.permanent.contains(&pos) || self.current.contains(&pos)
    }

    fn count_neighbors(&self, pos: IVec2) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if self.is_alive(pos + IVec2::new(dx, dy)) {
                    count += 1;
                }
            }
        }
        count
    }

    fn compute_next(&self) -> HashSet<IVec2> {
        let mut to_check: HashSet<IVec2> = HashSet::new();
        for &pos in self.current.iter().chain(self.permanent.iter()) {
            to_check.insert(pos);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    to_check.insert(pos + IVec2::new(dx, dy));
                }
            }
        }

        let mut next = HashSet::new();
        for pos in to_check {
            if self.permanent.contains(&pos) {
                continue;
            }
            let neighbors = self.count_neighbors(pos);
            let alive = self.current.contains(&pos);
            let will_live = if alive {
                neighbors == 2 || neighbors == 3
            } else {
                neighbors == 3
            };
            if will_live {
                next.insert(pos);
            }
        }
        next
    }
}

fn setup_menu_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<MenuCellState>,
) {
    state.dynamic_texture = Some(asset_server.load("menu/dynamic.png"));

    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("levels/menu.ldtk").into(),
            ..default()
        },
        DespawnOnExit::<AppState>::default(),
    ));
}

fn init_permanent_cells(
    mut commands: Commands,
    mut state: ResMut<MenuCellState>,
    query: Query<(Entity, &GridCoords, &ChildOf), Added<MenuPermanentCell>>,
) {
    for (entity, coords, child_of) in &query {
        let pos = IVec2::new(coords.x, coords.y);
        state.permanent.insert(pos);
        state.level_entity = Some(child_of.0);

        let world_pos = grid_to_world(pos);
        commands.entity(entity).insert((
            MenuCellSprite,
            Sprite {
                color: palette::BLACK,
                custom_size: Some(Vec2::splat(MENU_TILE_SIZE)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, CELL_Z),
            Visibility::Inherited,
            PIXEL_PERFECT_LAYERS,
        ));
    }
}

fn init_dynamic_cells(
    mut commands: Commands,
    mut state: ResMut<MenuCellState>,
    query: Query<(Entity, &GridCoords), Added<MenuDynamicCell>>,
) {
    let Some(texture) = state.dynamic_texture.clone() else {
        return;
    };

    for (entity, coords) in &query {
        let pos = IVec2::new(coords.x, coords.y);
        state.current.insert(pos);
        state.entities.insert(pos, entity);

        let world_pos = grid_to_world(pos);
        commands.entity(entity).insert((
            MenuCellSprite,
            Sprite::from_image(texture.clone()),
            Transform::from_xyz(world_pos.x, world_pos.y, CELL_Z),
            Visibility::Inherited,
            PIXEL_PERFECT_LAYERS,
        ));
    }

    if !state.initialized && !state.permanent.is_empty() {
        state.initialized = true;
        info!(
            "Menu initialized: {} permanent, {} dynamic",
            state.permanent.len(),
            state.current.len()
        );
    }
}

fn tick_simulation(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MenuTickTimer>,
    mut state: ResMut<MenuCellState>,
) {
    if !state.initialized {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let next = state.compute_next();

    let to_remove: Vec<IVec2> = state.current.difference(&next).copied().collect();

    let to_spawn: Vec<IVec2> = next.difference(&state.current).copied().collect();

    for pos in to_remove {
        if let Some(entity) = state.entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }

    let (Some(level_entity), Some(texture)) = (state.level_entity, state.dynamic_texture.clone())
    else {
        return;
    };

    for pos in to_spawn {
        let world_pos = grid_to_world(pos);
        let entity = commands
            .spawn((
                MenuCellSprite,
                MenuDynamicCell,
                Sprite::from_image(texture.clone()),
                Transform::from_xyz(world_pos.x, world_pos.y, CELL_Z),
                Visibility::Inherited,
                PIXEL_PERFECT_LAYERS,
                ChildOf(level_entity),
            ))
            .id();
        state.entities.insert(pos, entity);
    }

    state.current = next;
}

fn grid_to_world(grid_pos: IVec2) -> Vec2 {
    let half_level = MENU_LEVEL_SIZE / 2.0;
    Vec2::new(
        grid_pos.x as f32 * MENU_TILE_SIZE - half_level,
        grid_pos.y as f32 * MENU_TILE_SIZE - half_level,
    )
}

fn cleanup_menu_state(mut state: ResMut<MenuCellState>, mut timer: ResMut<MenuTickTimer>) {
    state.permanent.clear();
    state.current.clear();
    state.entities.clear();
    state.initialized = false;
    timer.0.reset();
}

pub(crate) fn cells_plugin_fn(app: &mut App) {
    app.init_resource::<MenuCellState>()
        .init_resource::<MenuTickTimer>()
        .register_ldtk_int_cell_for_layer::<MenuPermanentCellBundle>("Cells", 1)
        .register_ldtk_int_cell_for_layer::<MenuDynamicCellBundle>("Cells", 2)
        .add_systems(OnEnter(AppState::Menu), setup_menu_level)
        .add_systems(OnExit(AppState::Menu), cleanup_menu_state)
        .add_systems(
            Update,
            (init_permanent_cells, init_dynamic_cells, tick_simulation)
                .chain()
                .run_if(in_state(AppState::Menu)),
        );
}
