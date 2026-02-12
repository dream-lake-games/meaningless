use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig};
use bevy_tnua::prelude::*;

use crate::TILE_SIZE;

const PLAYER_SIZE: f32 = 20.0;
const PLAYER_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);
const FLOAT_GAP: f32 = 2.0;
const FLOAT_OFFSET: f32 = FLOAT_GAP;

#[derive(Component)]
pub struct Player;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerScheme {
    Jump(TnuaBuiltinJump),
}

pub fn spawn_player(commands: &mut Commands, configs: &mut Assets<PlayerSchemeConfig>, pos: Vec2) {
    let config = configs.add(PlayerSchemeConfig {
        basis: TnuaBuiltinWalkConfig {
            speed: 180.0,
            float_height: PLAYER_SIZE * 0.5 + FLOAT_GAP,
            cling_distance: FLOAT_GAP + 1.0,
            acceleration: 1600.0,
            air_acceleration: 600.0,
            coyote_time: 0.15,
            free_fall_extra_gravity: 400.0,
            ..default()
        },
        jump: TnuaBuiltinJumpConfig {
            height: TILE_SIZE * 3.0,
            input_buffer_time: 0.15,
            fall_extra_gravity: 300.0,
            shorten_extra_gravity: 600.0,
            ..default()
        },
    });

    commands
        .spawn((
            Name::new("Player"),
            Player,
            Transform::from_translation(pos.extend(10.0)),
            Visibility::Inherited,
            RigidBody::Dynamic,
            Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
            SpeculativeMargin(2.0),
            LockedAxes::ROTATION_LOCKED,
            TransformInterpolation,
            TnuaController::<PlayerScheme>::default(),
            TnuaConfig::<PlayerScheme>(config),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: PLAYER_COLOR,
                    custom_size: Some(Vec2::splat(PLAYER_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, -FLOAT_OFFSET, 0.0),
            ));
        });

    info!(
        "Player spawned at ({}, {}), size={}",
        pos.x, pos.y, PLAYER_SIZE
    );
}

fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController<PlayerScheme>, With<Player>>,
) {
    for mut controller in &mut query {
        controller.initiate_action_feeding();

        let mut direction = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += 1.0;
        }

        controller.basis = TnuaBuiltinWalk {
            desired_motion: Vec3::new(direction, 0.0, 0.0),
            ..default()
        };

        if keyboard.pressed(KeyCode::KeyJ) {
            controller.action(PlayerScheme::Jump(TnuaBuiltinJump::default()));
        }
    }
}

fn startup_system(mut commands: Commands, mut configs: ResMut<Assets<PlayerSchemeConfig>>) {
    spawn_player(&mut commands, &mut configs, Vec2::new(0.0, TILE_SIZE * 2.0));
}

pub fn player_plugin_fn(app: &mut App) {
    app.add_systems(Startup, startup_system)
        .add_systems(Update, player_input_system.in_set(TnuaUserControlsSystems));
}
