use avian2d::prelude::*;
use bevy::prelude::*;

use crate::TILE_SIZE;

const PLAYER_SIZE: f32 = 20.0;
const PLAYER_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);

const MOVE_SPEED: f32 = 200.0;
const GROUND_ACCEL: f32 = 8000.0;
const GROUND_DECEL: f32 = 8000.0;
const AIR_ACCEL: f32 = 5000.0;
const AIR_DECEL: f32 = 2400.0;

const JUMP_VELOCITY: f32 = 365.0;
const GRAVITY: f32 = -900.0;
const JUMP_CUT_GRAVITY_MULT: f32 = 2.2;
const MAX_FALL_SPEED: f32 = -400.0;

const COYOTE_TIME: f32 = 0.16;
const JUMP_BUFFER_TIME: f32 = 0.16;
const SKIN: f32 = 0.5;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerState {
    pub vx: f32,
    pub vy: f32,
    pub grounded: bool,
    coyote_timer: f32,
    jump_buffer_timer: f32,
    jump_held: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            grounded: false,
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
            jump_held: false,
        }
    }
}

pub fn spawn_player(commands: &mut Commands, pos: Vec2) {
    commands.spawn((
        Name::new("Player"),
        Player,
        Sprite {
            color: PLAYER_COLOR,
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_translation(pos.extend(10.0)),
        PlayerState::default(),
    ));

    info!(
        "Player spawned at ({}, {}), size={}",
        pos.x, pos.y, PLAYER_SIZE
    );
}

fn player_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    spatial: SpatialQuery,
    mut query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
) {
    let dt = time.delta_secs();
    let shape = Collider::rectangle(PLAYER_SIZE - SKIN * 2.0, PLAYER_SIZE - SKIN * 2.0);
    let filter = SpatialQueryFilter::default();

    for (mut tf, mut state) in &mut query {
        // --- Input ---
        let mut input_dir = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            input_dir -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input_dir += 1.0;
        }

        state.jump_held = keyboard.pressed(KeyCode::KeyJ);
        if keyboard.just_pressed(KeyCode::KeyJ) {
            state.jump_buffer_timer = JUMP_BUFFER_TIME;
        }
        state.jump_buffer_timer -= dt;

        // --- Horizontal velocity with acceleration ---
        let target_vx = input_dir * MOVE_SPEED;
        let (accel, decel) = if state.grounded {
            (GROUND_ACCEL, GROUND_DECEL)
        } else {
            (AIR_ACCEL, AIR_DECEL)
        };

        if input_dir != 0.0 {
            state.vx = move_toward(state.vx, target_vx, accel * dt);
        } else {
            state.vx = move_toward(state.vx, 0.0, decel * dt);
        }

        // --- Jump (with buffer + coyote) ---
        if state.jump_buffer_timer > 0.0 && state.coyote_timer > 0.0 {
            state.vy = JUMP_VELOCITY;
            state.coyote_timer = 0.0;
            state.jump_buffer_timer = 0.0;
        }

        // --- Gravity (variable: heavier when falling or jump released) ---
        let mut grav = GRAVITY;
        if state.vy > 0.0 && !state.jump_held {
            grav *= JUMP_CUT_GRAVITY_MULT;
        }
        state.vy += grav * dt;
        state.vy = state.vy.max(MAX_FALL_SPEED);

        let dx = state.vx * dt;
        let dy = state.vy * dt;
        let mut pos = tf.translation.truncate();

        // --- Horizontal movement ---
        if dx != 0.0 {
            let move_dir = if dx > 0.0 { Dir2::X } else { Dir2::NEG_X };
            let config = ShapeCastConfig::from_max_distance(dx.abs());

            if let Some(hit) = spatial.cast_shape(&shape, pos, 0.0, move_dir, &config, &filter) {
                pos.x += move_dir.as_vec2().x * (hit.distance - SKIN).max(0.0);
                state.vx = 0.0;
            } else {
                pos.x += dx;
            }
        }

        // --- Vertical movement ---
        if dy < 0.0 {
            let config = ShapeCastConfig::from_max_distance(dy.abs());
            if let Some(hit) = spatial.cast_shape(&shape, pos, 0.0, Dir2::NEG_Y, &config, &filter) {
                pos.y -= (hit.distance - SKIN).max(0.0);
                state.vy = 0.0;
                state.grounded = true;
                state.coyote_timer = COYOTE_TIME;
            } else {
                pos.y += dy;
                state.grounded = false;
                state.coyote_timer -= dt;
            }
        } else if dy > 0.0 {
            let config = ShapeCastConfig::from_max_distance(dy.abs());
            if let Some(hit) = spatial.cast_shape(&shape, pos, 0.0, Dir2::Y, &config, &filter) {
                pos.y += (hit.distance - SKIN).max(0.0);
                state.vy = 0.0;
            } else {
                pos.y += dy;
            }
            state.grounded = false;
            state.coyote_timer -= dt;
        } else {
            let config = ShapeCastConfig::from_max_distance(2.0);
            if spatial
                .cast_shape(&shape, pos, 0.0, Dir2::NEG_Y, &config, &filter)
                .is_some()
            {
                state.grounded = true;
                state.coyote_timer = COYOTE_TIME;
            } else {
                state.grounded = false;
                state.coyote_timer -= dt;
            }
        }

        pos.x = pos.x.round();
        pos.y = pos.y.round();

        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
    }
}

fn move_toward(current: f32, target: f32, max_step: f32) -> f32 {
    if (target - current).abs() <= max_step {
        target
    } else if target > current {
        current + max_step
    } else {
        current - max_step
    }
}

fn startup_system(mut commands: Commands) {
    spawn_player(&mut commands, Vec2::new(0.0, TILE_SIZE * 2.0));
}

pub fn player_plugin_fn(app: &mut App) {
    app.add_systems(Startup, startup_system)
        .add_systems(Update, player_system);
}
