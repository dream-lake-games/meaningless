#![allow(dead_code)]

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use std::marker::PhantomData;

#[derive(Component)]
pub struct AnimSprite<A: Anim> {
    _phantom: PhantomData<A>,
}

impl<A: Anim> Default for AnimSprite<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Component)]
pub struct AnimCache<A: Anim> {
    handles: Vec<Handle<Image>>,
    layouts: Vec<Handle<TextureAtlasLayout>>,
    _phantom: PhantomData<A>,
}

#[derive(Clone, Copy)]
pub struct AnimVariant {
    pub tag: &'static str,
    pub fps: Option<f32>,
    pub frame_count: usize,
    pub frame_size: (u32, u32),
    pub asset_path: &'static str,
    pub next: AnimNextIndex,
}

#[derive(Clone, Copy)]
pub enum AnimNextIndex {
    Index(usize),
    Remove,
    Despawn,
}

pub trait Anim: Clone + Copy + Default + Send + Sync + 'static {
    fn table() -> &'static [AnimVariant];
    fn index(&self) -> usize;
    fn from_index(index: usize) -> Self;
}

pub struct AnimPlugin {
    pub default_fps: f32,
}

impl Default for AnimPlugin {
    fn default() -> Self {
        Self { default_fps: 8.0 }
    }
}

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<AnimConfig>() {
            app.insert_resource(AnimConfig {
                default_fps: self.default_fps,
            });
        }
        crate::animations::register_all_anims(app);
    }
}

#[derive(Resource)]
pub struct AnimConfig {
    pub default_fps: f32,
}

#[derive(Component)]
#[component(on_add = on_add_anim_man::<A>)]
pub struct AnimMan<A: Anim> {
    table: &'static [AnimVariant],
    variant_index: usize,
    loaded_variant_index: usize,
    frame: usize,
    timer: f32,
    fps_override: Option<f32>,
    pub paused: bool,
    pub visible: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    _phantom: PhantomData<A>,
}

impl<A: Anim> AnimMan<A> {
    pub fn new(initial: A) -> Self {
        let index = initial.index();
        Self {
            table: A::table(),
            variant_index: index,
            loaded_variant_index: index,
            frame: 0,
            timer: 0.0,
            fps_override: None,
            paused: false,
            visible: true,
            flip_x: false,
            flip_y: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_flip_x(mut self, flip_x: bool) -> Self {
        self.flip_x = flip_x;
        self
    }

    pub fn with_fps(mut self, fps: f32) -> Self {
        self.fps_override = Some(fps);
        self
    }

    pub fn set(&mut self, value: A) {
        let new_index = value.index();
        if new_index != self.variant_index {
            self.variant_index = new_index;
            self.frame = 0;
            self.timer = 0.0;
        }
    }

    pub fn set_flip_x(&mut self, flip_x: bool) {
        self.flip_x = flip_x;
    }

    pub fn get(&self) -> A {
        A::from_index(self.variant_index)
    }

    fn current(&self) -> &AnimVariant {
        &self.table[self.variant_index]
    }
}

fn on_add_anim_man<A: Anim>(mut world: DeferredWorld, ctx: HookContext) {
    let state = world.get::<AnimMan<A>>(ctx.entity).unwrap();
    let variant_index = state.variant_index;
    let frame = state.frame;
    let visible = state.visible;
    let flip_x = state.flip_x;
    let flip_y = state.flip_y;

    let asset_server = world.resource::<AssetServer>();
    let table = A::table();

    let mut handles: Vec<Handle<Image>> = Vec::with_capacity(table.len());
    for variant in table {
        handles.push(asset_server.load(variant.asset_path));
    }

    let mut layouts_asset = world.resource_mut::<Assets<TextureAtlasLayout>>();
    let mut layouts: Vec<Handle<TextureAtlasLayout>> = Vec::with_capacity(table.len());
    for variant in table {
        let (frame_width, frame_height) = variant.frame_size;
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(frame_width, frame_height),
            variant.frame_count as u32,
            1,
            None,
            None,
        );
        layouts.push(layouts_asset.add(layout));
    }

    let image_handle = handles[variant_index].clone();
    let layout_handle = layouts[variant_index].clone();

    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    let cache = AnimCache::<A> {
        handles,
        layouts,
        _phantom: PhantomData,
    };

    let parent = ctx.entity;
    world.commands().entity(parent).insert(cache);
    world.commands().entity(parent).with_children(|children| {
        children.spawn((
            AnimSprite::<A>::default(),
            Sprite {
                image: image_handle,
                flip_x,
                flip_y,
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle,
                    index: frame,
                }),
                ..default()
            },
            Transform::default(),
            visibility,
        ));
    });
}

fn load_variant_sprite<A: Anim>(state: &mut AnimMan<A>, sprite: &mut Sprite, cache: &AnimCache<A>) {
    let idx = state.variant_index;
    sprite.image = cache.handles[idx].clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: cache.layouts[idx].clone(),
        index: state.frame,
    });
    state.loaded_variant_index = state.variant_index;
}

fn tick_anim<A: Anim>(
    time: Res<Time>,
    config: Res<AnimConfig>,
    mut commands: Commands,
    mut anim_query: Query<(Entity, &mut AnimMan<A>, &AnimCache<A>, &Children)>,
    mut sprite_query: Query<(Entity, &mut Sprite, &mut Visibility), With<AnimSprite<A>>>,
) {
    for (entity, mut state, cache, children) in anim_query.iter_mut() {
        let mut sprite_entity_opt = None;
        for child in children.iter() {
            if sprite_query.contains(child) {
                sprite_entity_opt = Some(child);
                break;
            }
        }
        let Some(sprite_entity) = sprite_entity_opt else {
            continue;
        };
        let Ok((_, mut sprite, mut visibility)) = sprite_query.get_mut(sprite_entity) else {
            continue;
        };

        *visibility = if state.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        sprite.flip_x = state.flip_x;
        sprite.flip_y = state.flip_y;

        if state.variant_index != state.loaded_variant_index {
            load_variant_sprite(&mut state, &mut sprite, cache);
        }

        if state.paused {
            continue;
        }

        let variant = state.current();
        let fps = state
            .fps_override
            .or(variant.fps)
            .unwrap_or(config.default_fps);
        let frame_duration = 1.0 / fps;

        state.timer += time.delta_secs();

        while state.timer >= frame_duration {
            state.timer -= frame_duration;
            state.frame += 1;

            let variant = state.current();
            if state.frame >= variant.frame_count {
                match variant.next {
                    AnimNextIndex::Index(next_index) => {
                        state.variant_index = next_index;
                        state.frame = 0;
                        load_variant_sprite(&mut state, &mut sprite, cache);
                    }
                    AnimNextIndex::Remove => {
                        commands.entity(entity).remove::<AnimMan<A>>();
                        commands.entity(sprite_entity).despawn();
                        return;
                    }
                    AnimNextIndex::Despawn => {
                        commands.entity(entity).despawn();
                        return;
                    }
                }
            }
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = state.frame;
        }
    }
}

pub fn register_anim<A: Anim>(app: &mut App) {
    app.add_systems(Update, tick_anim::<A>);
}
