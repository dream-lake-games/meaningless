use bevy::prelude::*;
use rand::RngExt;

const VOLUME_BACKWARD: f32 = 0.4;
const VOLUME_FORWARD: f32 = 0.4;
const VOLUME_DEATH: f32 = 0.6;
const VOLUME_FLAG_GET: f32 = 0.8;
const VOLUME_FOOTSTEP: f32 = 0.3;
const VOLUME_JUMP: f32 = 0.6;
const VOLUME_LANDING: f32 = 0.5;
const VOLUME_MENU_MOVE: f32 = 0.4;
const VOLUME_MENU_SELECT: f32 = 0.5;

const PITCH_VARIANCE: f32 = 0.1;

#[derive(Resource)]
pub(crate) struct Sfx {
    pub(crate) backward: Handle<AudioSource>,
    pub(crate) forward: Handle<AudioSource>,
    pub(crate) death: Handle<AudioSource>,
    pub(crate) flag_get: Handle<AudioSource>,
    pub(crate) footsteps_a: Handle<AudioSource>,
    pub(crate) footsteps_b: Handle<AudioSource>,
    pub(crate) jump: Handle<AudioSource>,
    pub(crate) landing: Handle<AudioSource>,
    pub(crate) menu_move: Handle<AudioSource>,
    pub(crate) menu_select: Handle<AudioSource>,
}

fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sfx {
        backward: asset_server.load("sound/backward.ogg"),
        forward: asset_server.load("sound/forward.ogg"),
        death: asset_server.load("sound/death.ogg"),
        flag_get: asset_server.load("sound/flag_get.ogg"),
        footsteps_a: asset_server.load("sound/footsteps_a.ogg"),
        footsteps_b: asset_server.load("sound/footsteps_b.ogg"),
        jump: asset_server.load("sound/jump.ogg"),
        landing: asset_server.load("sound/landing.ogg"),
        menu_move: asset_server.load("sound/menu_move.ogg"),
        menu_select: asset_server.load("sound/menu_select.ogg"),
    });
}

fn random_speed() -> f32 {
    1.0 + rand::rng().random_range(-PITCH_VARIANCE..PITCH_VARIANCE)
}

pub(crate) fn play_backward(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.backward.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_BACKWARD),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_forward(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.forward.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_FORWARD),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_death(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.death.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_DEATH),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_flag_get(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.flag_get.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_FLAG_GET),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_footstep(commands: &mut Commands, sfx: &Sfx) {
    let handle = if rand::rng().random_bool(0.5) {
        sfx.footsteps_a.clone()
    } else {
        sfx.footsteps_b.clone()
    };
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_FOOTSTEP),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_jump(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.jump.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_JUMP),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_landing(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.landing.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_LANDING),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_menu_move(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.menu_move.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_MENU_MOVE),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn play_menu_select(commands: &mut Commands, sfx: &Sfx) {
    commands.spawn((
        AudioPlayer::new(sfx.menu_select.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(VOLUME_MENU_SELECT),
            speed: random_speed(),
            ..default()
        },
    ));
}

pub(crate) fn sfx_plugin_fn(app: &mut App) {
    app.add_systems(Startup, load_sfx);
}
