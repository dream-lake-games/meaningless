use bevy::{
    camera::visibility::RenderLayers,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use crate::menu::AppState;
use crate::player::Player;
use crate::transition::TransitionState;
use crate::{INTERNAL_SIZE, WINDOW_SIZE};

pub(crate) const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
pub(crate) const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

const DEADZONE_X: f32 = 16.0;
const DEADZONE_Y: f32 = 24.0;
const CATCH_UP_SPEED: f32 = 8.0;

#[derive(Component)]
pub(crate) struct InGameCamera;

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Canvas;

fn spawn_cameras(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: INTERNAL_SIZE,
        height: INTERNAL_SIZE,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    commands.spawn((
        Name::new("InGameCamera"),
        InGameCamera,
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        RenderTarget::Image(image_handle.clone().into()),
        Msaa::Off,
        PIXEL_PERFECT_LAYERS,
    ));

    let scale = WINDOW_SIZE as f32 / INTERNAL_SIZE as f32;
    commands.spawn((
        Name::new("Canvas"),
        Canvas,
        Sprite::from_image(image_handle),
        Transform::from_scale(Vec3::splat(scale)),
        HIGH_RES_LAYERS,
    ));

    commands.spawn((
        Name::new("OuterCamera"),
        OuterCamera,
        Camera2d,
        Msaa::Off,
        HIGH_RES_LAYERS,
    ));
}

fn camera_follow_system(
    time: Res<Time>,
    transition: Res<TransitionState>,
    player_q: Query<&Transform, (With<Player>, Without<InGameCamera>)>,
    mut camera_q: Query<&mut Transform, (With<InGameCamera>, Without<Player>)>,
) {
    if transition.is_active() {
        return;
    }

    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut camera_tf) = camera_q.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let player_pos = player_tf.translation.truncate();
    let camera_pos = camera_tf.translation.truncate();
    let diff = player_pos - camera_pos;

    let mut target = camera_pos;

    if diff.x > DEADZONE_X {
        target.x = player_pos.x - DEADZONE_X;
    } else if diff.x < -DEADZONE_X {
        target.x = player_pos.x + DEADZONE_X;
    }

    if diff.y > DEADZONE_Y {
        target.y = player_pos.y - DEADZONE_Y;
    } else if diff.y < -DEADZONE_Y {
        target.y = player_pos.y + DEADZONE_Y;
    }

    let new_pos = camera_pos.lerp(target, (dt * CATCH_UP_SPEED).min(1.0));

    camera_tf.translation.x = new_pos.x.round();
    camera_tf.translation.y = new_pos.y.round();
}

pub(crate) fn camera_plugin_fn(app: &mut App) {
    app.add_systems(Startup, spawn_cameras).add_systems(
        PostUpdate,
        camera_follow_system
            .before(TransformSystems::Propagate)
            .run_if(in_state(AppState::Playing)),
    );
}
