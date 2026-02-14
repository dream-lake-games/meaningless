use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::TILE_SIZE;
use crate::anim::AnimMan;
use crate::animations::CellAnim;
use crate::level::{DynamicCell, LevelSystemSet, NeverCell, PermanentCell};
use crate::menu::AppState;
use crate::player::{Player, PlayerState};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GolSystemSet;

const PLAYER_SIZE: f32 = 18.0;
const CRUSH_THRESHOLD: f32 = 0.25;
const HISTORY_CAP: usize = 256;

#[derive(Resource, Default)]
pub(crate) struct TickState {
    pub(crate) previous: Option<HashSet<IVec2>>,
    pub(crate) current: HashSet<IVec2>,
    pub(crate) next: HashSet<IVec2>,
    pub(crate) permanent: HashSet<IVec2>,
    pub(crate) never: HashSet<IVec2>,
    pub(crate) history: Vec<HashSet<IVec2>>,
    pub(crate) initialized: bool,
}

impl TickState {
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
        let mut positions_to_check: HashSet<IVec2> = HashSet::new();
        for &pos in self.current.iter().chain(self.permanent.iter()) {
            positions_to_check.insert(pos);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    positions_to_check.insert(pos + IVec2::new(dx, dy));
                }
            }
        }

        let mut next: HashSet<IVec2> = HashSet::new();
        for pos in positions_to_check {
            if self.permanent.contains(&pos) || self.never.contains(&pos) {
                continue;
            }
            let neighbors = self.count_neighbors(pos);
            let currently_alive = self.current.contains(&pos);
            let will_live = if currently_alive {
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

    fn relevant_positions(&self) -> HashSet<IVec2> {
        let mut positions = self.current.clone();
        positions.extend(&self.next);
        if let Some(prev) = &self.previous {
            for &pos in prev {
                if !self.current.contains(&pos) {
                    positions.insert(pos);
                }
            }
        }
        positions
    }

    fn cell_state(&self, pos: IVec2) -> Option<CellAnim> {
        let in_current = self.current.contains(&pos);
        let in_next = self.next.contains(&pos);
        let in_previous = self.previous.as_ref().is_some_and(|p| p.contains(&pos));

        if in_current && in_next {
            Some(CellAnim::Stable)
        } else if in_current && !in_next {
            Some(CellAnim::Scared)
        } else if !in_current && in_next {
            Some(CellAnim::Pending)
        } else if in_previous && !in_current {
            Some(CellAnim::Slain)
        } else {
            None
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct CellEntities {
    pub(crate) map: HashMap<IVec2, Entity>,
}

#[derive(Resource, Default)]
pub(crate) struct SpawnPosition(pub(crate) Option<Vec2>);

#[derive(Resource, Default)]
pub(crate) struct RespawnTimer(pub(crate) Option<Timer>);

#[derive(Resource, Default)]
pub(crate) struct LevelEntity(pub(crate) Option<Entity>);

#[derive(Component)]
pub(crate) struct GolCell;

fn init_from_ldtk(
    mut commands: Commands,
    mut tick_state: ResMut<TickState>,
    mut cell_entities: ResMut<CellEntities>,
    mut level_entity: ResMut<LevelEntity>,
    permanent_query: Query<&GridCoords, With<PermanentCell>>,
    dynamic_query: Query<(Entity, &GridCoords), (With<DynamicCell>, Without<GolCell>)>,
    never_query: Query<&GridCoords, With<NeverCell>>,
    new_permanent: Query<(), Added<PermanentCell>>,
    new_dynamic: Query<(), (Added<DynamicCell>, Without<GolCell>)>,
    new_never: Query<(), Added<NeverCell>>,
    level_query: Query<Entity, With<LevelIid>>,
) {
    let has_new_cells = !new_permanent.is_empty() || !new_dynamic.is_empty() || !new_never.is_empty();
    if !has_new_cells {
        return;
    }

    if tick_state.initialized {
        tick_state.previous = None;
        tick_state.current.clear();
        tick_state.next.clear();
        tick_state.permanent.clear();
        tick_state.never.clear();
        tick_state.history.clear();
        tick_state.initialized = false;
        cell_entities.map.clear();
    }

    if let Ok(entity) = level_query.single() {
        level_entity.0 = Some(entity);
    }

    for coords in &permanent_query {
        tick_state.permanent.insert(IVec2::new(coords.x, coords.y));
    }

    for coords in &never_query {
        tick_state.never.insert(IVec2::new(coords.x, coords.y));
    }

    for (entity, coords) in &dynamic_query {
        let pos = IVec2::new(coords.x, coords.y);
        tick_state.current.insert(pos);
        cell_entities.map.insert(pos, entity);
        commands.entity(entity).insert((
            GolCell,
            AnimMan::new(CellAnim::Stable),
            Visibility::Inherited,
            RigidBody::Static,
            Collider::rectangle(TILE_SIZE, TILE_SIZE),
        ));
    }

    tick_state.initialized = true;
    tick_state.next = tick_state.compute_next();

    info!(
        "GoL initialized: {} permanent, {} dynamic, {} pending",
        tick_state.permanent.len(),
        tick_state.current.len(),
        tick_state.next.len()
    );
}

fn process_tick_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tick_state: ResMut<TickState>,
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
    mut respawn_timer: ResMut<RespawnTimer>,
) {
    if !tick_state.initialized {
        return;
    }

    let forward = keyboard.just_pressed(KeyCode::KeyK);
    let backward = keyboard.just_pressed(KeyCode::KeyL);

    if !forward && !backward {
        return;
    }

    let old_current = tick_state.current.clone();

    if forward {
        if tick_state.history.len() >= HISTORY_CAP {
            tick_state.history.remove(0);
        }
        tick_state.history.push(old_current.clone());
        tick_state.previous = Some(old_current.clone());
        tick_state.current = std::mem::take(&mut tick_state.next);
        tick_state.next = tick_state.compute_next();
    } else if backward {
        if tick_state.history.is_empty() {
            return;
        }

        let restored = tick_state.history.pop().unwrap();
        tick_state.previous = tick_state.history.last().cloned();
        tick_state.current = restored;
        tick_state.next = tick_state.compute_next();
    }

    let newly_alive: Vec<IVec2> = tick_state
        .current
        .difference(&old_current)
        .copied()
        .collect();

    if !newly_alive.is_empty() {
        handle_player_collision(&newly_alive, &mut player_query, &mut respawn_timer);
    }
}

fn update_existing_cells(
    mut commands: Commands,
    tick_state: Res<TickState>,
    mut cell_entities: ResMut<CellEntities>,
    mut cell_query: Query<
        (
            Entity,
            &GridCoords,
            &mut AnimMan<CellAnim>,
            Option<&RigidBody>,
        ),
        With<GolCell>,
    >,
) {
    if !tick_state.initialized {
        return;
    }

    let mut to_remove: Vec<IVec2> = Vec::new();

    for (entity, coords, mut anim, has_collider) in &mut cell_query {
        let pos = IVec2::new(coords.x, coords.y);

        if let Some(state) = tick_state.cell_state(pos) {
            let should_be_solid = tick_state.current.contains(&pos);
            let is_solid = has_collider.is_some();

            anim.set(state);

            if should_be_solid && !is_solid {
                commands
                    .entity(entity)
                    .insert((RigidBody::Static, Collider::rectangle(TILE_SIZE, TILE_SIZE)));
            } else if !should_be_solid && is_solid {
                commands
                    .entity(entity)
                    .remove::<RigidBody>()
                    .remove::<Collider>();
            }
        } else {
            to_remove.push(pos);
            commands.entity(entity).despawn();
        }
    }

    for pos in to_remove {
        cell_entities.map.remove(&pos);
    }
}

fn spawn_missing_cells(
    mut commands: Commands,
    tick_state: Res<TickState>,
    mut cell_entities: ResMut<CellEntities>,
    level_entity: Res<LevelEntity>,
) {
    if !tick_state.initialized {
        return;
    }

    let Some(level_ent) = level_entity.0 else {
        return;
    };

    let relevant = tick_state.relevant_positions();

    for pos in relevant {
        if cell_entities.map.contains_key(&pos) {
            continue;
        }

        let Some(state) = tick_state.cell_state(pos) else {
            continue;
        };

        let world_pos = grid_to_world(pos);
        let should_be_solid = tick_state.current.contains(&pos);

        let mut entity_commands = commands.spawn((
            GolCell,
            GridCoords::new(pos.x, pos.y),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            Visibility::Inherited,
            AnimMan::new(state),
            ChildOf(level_ent),
        ));

        if should_be_solid {
            entity_commands.insert((RigidBody::Static, Collider::rectangle(TILE_SIZE, TILE_SIZE)));
        }

        cell_entities.map.insert(pos, entity_commands.id());
    }
}

fn handle_player_collision(
    appeared: &[IVec2],
    player_query: &mut Query<(&mut Transform, &mut PlayerState), With<Player>>,
    respawn_timer: &mut ResMut<RespawnTimer>,
) {
    if appeared.is_empty() {
        return;
    }

    let Ok((mut player_tf, mut player_state)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();
    let player_half = PLAYER_SIZE / 2.0;
    let player_area = PLAYER_SIZE * PLAYER_SIZE;
    let tile_half = TILE_SIZE / 2.0;

    for &pos in appeared {
        let cell_world_pos = grid_to_world(pos);
        let overlap = calculate_overlap(player_pos, player_half, cell_world_pos, tile_half);

        if overlap <= 0.0 {
            continue;
        }

        let overlap_ratio = overlap / player_area;

        if overlap_ratio > CRUSH_THRESHOLD {
            respawn_timer.0 = Some(Timer::from_seconds(0.5, TimerMode::Once));
            return;
        }

        if let Some((dir, amount)) =
            find_push_direction(player_pos, player_half, cell_world_pos, tile_half)
        {
            let new_pos = player_pos + dir * amount;
            player_tf.translation.x = new_pos.x;
            player_tf.translation.y = new_pos.y;

            if dir.x != 0.0 {
                player_state.vx = 0.0;
            }
            if dir.y != 0.0 {
                player_state.vy = 0.0;
            }
        }
    }
}

fn grid_to_world(grid_pos: IVec2) -> Vec2 {
    Vec2::new(
        grid_pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        grid_pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
    )
}

fn calculate_overlap(player_pos: Vec2, player_half: f32, cell_pos: Vec2, cell_half: f32) -> f32 {
    let player_min = player_pos - Vec2::splat(player_half);
    let player_max = player_pos + Vec2::splat(player_half);
    let cell_min = cell_pos - Vec2::splat(cell_half);
    let cell_max = cell_pos + Vec2::splat(cell_half);

    let overlap_x = (player_max.x.min(cell_max.x) - player_min.x.max(cell_min.x)).max(0.0);
    let overlap_y = (player_max.y.min(cell_max.y) - player_min.y.max(cell_min.y)).max(0.0);

    overlap_x * overlap_y
}

fn find_push_direction(
    player_pos: Vec2,
    player_half: f32,
    cell_pos: Vec2,
    cell_half: f32,
) -> Option<(Vec2, f32)> {
    let player_min = player_pos - Vec2::splat(player_half);
    let player_max = player_pos + Vec2::splat(player_half);
    let cell_min = cell_pos - Vec2::splat(cell_half);
    let cell_max = cell_pos + Vec2::splat(cell_half);

    let push_left = player_max.x - cell_min.x;
    let push_right = cell_max.x - player_min.x;
    let push_down = player_max.y - cell_min.y;
    let push_up = cell_max.y - player_min.y;

    let directions = [
        (Vec2::NEG_X, push_left),
        (Vec2::X, push_right),
        (Vec2::NEG_Y, push_down),
        (Vec2::Y, push_up),
    ];

    directions
        .into_iter()
        .filter(|(_, dist)| *dist > 0.0)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(dir, dist)| (dir, dist + 1.0))
}

fn handle_respawn(
    mut commands: Commands,
    mut respawn_timer: ResMut<RespawnTimer>,
    spawn_pos: Res<SpawnPosition>,
    time: Res<Time>,
    player_query: Query<Entity, With<Player>>,
) {
    let Some(ref mut timer) = respawn_timer.0 else {
        return;
    };

    timer.tick(time.delta());

    if timer.just_finished() {
        for entity in &player_query {
            commands.entity(entity).despawn();
        }

        if let Some(pos) = spawn_pos.0 {
            crate::player::spawn_player(&mut commands, pos);
        }
        respawn_timer.0 = None;
    }
}

pub(crate) const GHOST_ALPHA: f32 = 0.5;

fn sync_cell_opacity(
    cell_query: Query<(Option<&RigidBody>, &Children), With<GolCell>>,
    mut sprite_query: Query<&mut Sprite>,
) {
    for (has_collider, children) in &cell_query {
        let alpha = if has_collider.is_some() { 1.0 } else { GHOST_ALPHA };
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_query.get_mut(child) {
                sprite.color = sprite.color.with_alpha(alpha);
            }
        }
    }
}

pub(crate) fn gol_plugin_fn(app: &mut App) {
    app.init_resource::<TickState>()
        .init_resource::<CellEntities>()
        .init_resource::<SpawnPosition>()
        .init_resource::<RespawnTimer>()
        .init_resource::<LevelEntity>()
        .add_systems(
            Update,
            (
                init_from_ldtk,
                process_tick_input,
                update_existing_cells,
                spawn_missing_cells,
                handle_respawn,
                sync_cell_opacity,
            )
                .chain()
                .in_set(GolSystemSet)
                .after(LevelSystemSet)
                .run_if(in_state(AppState::Playing)),
        );
}
