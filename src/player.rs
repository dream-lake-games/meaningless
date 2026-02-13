use avian2d::prelude::*;
use bevy::prelude::*;

use crate::anim::AnimMan;
use crate::animations::PlayerAnim;

const HITBOX_WIDTH: f32 = 12.0;
const HITBOX_HEIGHT: f32 = 14.0;
const HITBOX_OFFSET_Y: f32 = -1.0;

const MOVE_SPEED: f32 = 150.0;
const GROUND_ACCEL: f32 = 4000.0;
const GROUND_DECEL: f32 = 8000.0;
const AIR_ACCEL: f32 = 1600.0;
const AIR_DECEL: f32 = 1200.0;

const JUMP_VELOCITY: f32 = 240.0;
const GRAVITY: f32 = -640.0;
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
    pub was_grounded: bool,
    pub facing_right: bool,
    pub pushing_wall: bool,
    coyote_timer: f32,
    jump_buffer_timer: f32,
    jump_held: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            grounded: true,
            was_grounded: true,
            facing_right: true,
            pushing_wall: false,
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
        AnimMan::new(PlayerAnim::Idle),
        Transform::from_translation(pos.extend(10.0)),
        Visibility::Inherited,
        PlayerState::default(),
    ));

    info!("Player spawned at ({}, {})", pos.x, pos.y);
}

fn player_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    spatial: SpatialQuery,
    mut query: Query<(&mut Transform, &mut PlayerState, &mut AnimMan<PlayerAnim>), With<Player>>,
) {
    let dt = time.delta_secs();
    let shape = Collider::rectangle(HITBOX_WIDTH - SKIN * 2.0, HITBOX_HEIGHT - SKIN * 2.0);
    let filter = SpatialQueryFilter::default();

    for (mut tf, mut state, mut anim) in &mut query {
        let mut input_dir = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            input_dir -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input_dir += 1.0;
        }

        if input_dir > 0.0 {
            state.facing_right = true;
        } else if input_dir < 0.0 {
            state.facing_right = false;
        }

        state.jump_held = keyboard.pressed(KeyCode::KeyJ);
        if keyboard.just_pressed(KeyCode::KeyJ) {
            state.jump_buffer_timer = JUMP_BUFFER_TIME;
        }
        state.jump_buffer_timer -= dt;

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

        let mut just_jumped = false;
        if state.jump_buffer_timer > 0.0 && state.coyote_timer > 0.0 {
            state.vy = JUMP_VELOCITY;
            state.coyote_timer = 0.0;
            state.jump_buffer_timer = 0.0;
            just_jumped = true;
        }

        let mut grav = GRAVITY;
        if state.vy > 0.0 && !state.jump_held {
            grav *= JUMP_CUT_GRAVITY_MULT;
        }
        state.vy += grav * dt;
        state.vy = state.vy.max(MAX_FALL_SPEED);

        let dx = state.vx * dt;
        let dy = state.vy * dt;
        let mut pos = tf.translation.truncate();
        let hitbox_pos = pos + Vec2::new(0.0, HITBOX_OFFSET_Y);

        if dx != 0.0 {
            let move_dir = if dx > 0.0 { Dir2::X } else { Dir2::NEG_X };
            let config = ShapeCastConfig::from_max_distance(dx.abs());

            if let Some(hit) =
                spatial.cast_shape(&shape, hitbox_pos, 0.0, move_dir, &config, &filter)
            {
                pos.x += move_dir.as_vec2().x * (hit.distance - SKIN).max(0.0);
                state.vx = 0.0;
            } else {
                pos.x += dx;
            }
        }

        let hitbox_pos = pos + Vec2::new(0.0, HITBOX_OFFSET_Y);

        if dy < 0.0 {
            let config = ShapeCastConfig::from_max_distance(dy.abs());
            if let Some(hit) =
                spatial.cast_shape(&shape, hitbox_pos, 0.0, Dir2::NEG_Y, &config, &filter)
            {
                pos.y -= (hit.distance - SKIN).max(0.0);
                state.vy = 0.0;
            } else {
                pos.y += dy;
            }
        } else if dy > 0.0 {
            let config = ShapeCastConfig::from_max_distance(dy.abs());
            if let Some(hit) =
                spatial.cast_shape(&shape, hitbox_pos, 0.0, Dir2::Y, &config, &filter)
            {
                pos.y += (hit.distance - SKIN).max(0.0);
                state.vy = 0.0;
            } else {
                pos.y += dy;
            }
        }

        let hitbox_pos = pos + Vec2::new(0.0, HITBOX_OFFSET_Y);
        let ground_check = ShapeCastConfig::from_max_distance(2.0);
        let on_ground = spatial
            .cast_shape(&shape, hitbox_pos, 0.0, Dir2::NEG_Y, &ground_check, &filter)
            .is_some();

        if on_ground {
            state.grounded = true;
            state.coyote_timer = COYOTE_TIME;
        } else {
            state.grounded = false;
            state.coyote_timer -= dt;
        }

        state.pushing_wall = false;
        if input_dir != 0.0 && state.grounded {
            let wall_dir = if input_dir > 0.0 {
                Dir2::X
            } else {
                Dir2::NEG_X
            };
            let wall_check = ShapeCastConfig::from_max_distance(2.0);
            state.pushing_wall = spatial
                .cast_shape(&shape, hitbox_pos, 0.0, wall_dir, &wall_check, &filter)
                .is_some();
        }

        pos.x = pos.x.round();
        pos.y = pos.y.round();

        tf.translation.x = pos.x;
        tf.translation.y = pos.y;

        anim.set_flip_x(!state.facing_right);

        let just_landed = state.grounded && !state.was_grounded;
        let current = anim.get();
        let holding_down = keyboard.pressed(KeyCode::KeyS);
        let holding_up = keyboard.pressed(KeyCode::KeyW);
        let is_idle = state.vx.abs() < 5.0;

        let new_anim = if just_jumped {
            PlayerAnim::Jump
        } else if just_landed {
            PlayerAnim::Land
        } else if !state.grounded {
            match current {
                PlayerAnim::Jump => current,
                _ => {
                    if state.vx.abs() > 20.0 {
                        PlayerAnim::AirMove
                    } else {
                        PlayerAnim::AirStill
                    }
                }
            }
        } else if current == PlayerAnim::Land {
            current
        } else if state.pushing_wall {
            PlayerAnim::Push
        } else if state.vx.abs() > 20.0 {
            PlayerAnim::Run
        } else if is_idle && holding_down {
            PlayerAnim::Duck
        } else if is_idle && holding_up {
            PlayerAnim::Up
        } else if is_idle {
            PlayerAnim::Idle
        } else {
            current
        };

        anim.set(new_anim);

        state.was_grounded = state.grounded;
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

pub fn player_plugin_fn(app: &mut App) {
    app.add_systems(Update, player_system);
}
