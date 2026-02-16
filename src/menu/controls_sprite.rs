use bevy::prelude::*;

use crate::anim::AnimMan;
use crate::animations::ControlsAnim;
use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::menu::navigation::{ControlScheme, MenuNavigation, MenuScreen};
use crate::menu::AppState;

#[derive(Component)]
struct ControlsSpriteMarker;

fn spawn_controls_sprite(mut commands: Commands) {
    commands.spawn((
        ControlsSpriteMarker,
        AnimMan::new(ControlsAnim::Arrow),
        Transform::from_xyz(0.0, 0.0, 50.0),
        Visibility::Hidden,
        PIXEL_PERFECT_LAYERS,
    ));
}

fn update_controls_sprite(
    nav: Res<MenuNavigation>,
    controls: Res<ControlScheme>,
    mut query: Query<(&mut AnimMan<ControlsAnim>, &mut Visibility), With<ControlsSpriteMarker>>,
) {
    for (mut anim, mut visibility) in &mut query {
        let should_show = nav.screen == MenuScreen::Controls;
        *visibility = if should_show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        let target_anim = match *controls {
            ControlScheme::Arrow => ControlsAnim::Arrow,
            ControlScheme::Wasd => ControlsAnim::Wasd,
        };

        if anim.get() != target_anim {
            anim.set(target_anim);
        }
    }
}

fn despawn_controls_sprite(mut commands: Commands, query: Query<Entity, With<ControlsSpriteMarker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn controls_sprite_plugin_fn(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), spawn_controls_sprite)
        .add_systems(OnExit(AppState::Menu), despawn_controls_sprite)
        .add_systems(
            Update,
            update_controls_sprite.run_if(in_state(AppState::Menu)),
        );
}
