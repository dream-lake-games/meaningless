use bevy::prelude::*;
use bevy_ecs_ldtk::ldtk::FieldValue;
use bevy_ecs_ldtk::prelude::*;

use crate::anim::AnimMan;
use crate::animations::{PlayerAnim, SignAnim};
use crate::camera::HIGH_RES_LAYERS;
use crate::level::HelpText;
use crate::menu::AppState;
use crate::player::Player;

const INTERACT_DISTANCE: f32 = 32.0;
const DIALOGUE_BOX_WIDTH: f32 = 500.0;
const DIALOGUE_BOX_HEIGHT: f32 = 80.0;
const DIALOGUE_Y_POS: f32 = 260.0;
const CHARS_PER_SECOND: f32 = 30.0;

#[derive(Component, Default)]
pub(crate) struct Sign;

#[derive(Component, Default, Reflect)]
struct SignText {
    lines: Vec<String>,
}

impl From<&EntityInstance> for SignText {
    fn from(entity: &EntityInstance) -> Self {
        let mut lines = Vec::new();
        for field in &entity.field_instances {
            if field.identifier == "text" {
                if let FieldValue::Strings(strings) = &field.value {
                    lines = strings.iter().filter_map(|s| s.clone()).collect();
                }
            }
        }
        Self { lines }
    }
}

#[derive(Bundle, LdtkEntity, Default)]
struct SignBundle {
    marker: Sign,
    #[from_entity_instance]
    text: SignText,
}

#[derive(Resource, Default)]
pub(crate) struct DialogueState {
    pub(crate) active: bool,
    lines: Vec<String>,
    current_line: usize,
    char_index: usize,
    char_timer: f32,
    sign_entity: Option<Entity>,
}

impl DialogueState {
    fn current_display_text(&self) -> String {
        if self.current_line >= self.lines.len() {
            return String::new();
        }
        let line = &self.lines[self.current_line];
        line.chars().take(self.char_index).collect()
    }

    fn is_line_complete(&self) -> bool {
        if self.current_line >= self.lines.len() {
            return true;
        }
        self.char_index >= self.lines[self.current_line].len()
    }

    fn advance(&mut self) {
        if !self.is_line_complete() {
            self.char_index = self.lines[self.current_line].len();
        } else {
            self.current_line += 1;
            self.char_index = 0;
            if self.current_line >= self.lines.len() {
                self.active = false;
            }
        }
    }
}

#[derive(Component)]
struct DialogueBox;

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct SignMarker;

fn setup_sign_visuals(
    mut commands: Commands,
    query: Query<(Entity, &SignText), Added<Sign>>,
) {
    for (entity, text) in &query {
        info!("Sign spawned with {} lines of text: {:?}", text.lines.len(), text.lines);
        commands.entity(entity).insert((
            SignMarker,
            AnimMan::new(SignAnim::Idle),
            Visibility::Inherited,
        ));
    }
}

fn update_sign_interaction(
    mut help_text: ResMut<HelpText>,
    mut dialogue: ResMut<DialogueState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &AnimMan<PlayerAnim>), With<Player>>,
    sign_query: Query<(Entity, &Transform, &SignText), With<Sign>>,
    mut anim_query: Query<&mut AnimMan<SignAnim>>,
) {
    if dialogue.active {
        for mut anim in &mut anim_query {
            if anim.get() != SignAnim::Active {
                anim.set(SignAnim::Active);
            }
        }
        return;
    }

    let Ok((player_tf, player_anim)) = player_query.single() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();
    let player_idle = player_anim.get() == PlayerAnim::Idle;

    let mut near_sign: Option<(Entity, &SignText)> = None;

    for (entity, sign_tf, sign_text) in &sign_query {
        let sign_pos = sign_tf.translation.truncate();
        let distance = player_pos.distance(sign_pos);

        if distance < INTERACT_DISTANCE {
            if player_idle {
                if let Ok(mut anim) = anim_query.get_mut(entity) {
                    if anim.get() != SignAnim::Active {
                        anim.set(SignAnim::Active);
                    }
                }
                near_sign = Some((entity, sign_text));
            }
        } else {
            if let Ok(mut anim) = anim_query.get_mut(entity) {
                if anim.get() != SignAnim::Idle {
                    anim.set(SignAnim::Idle);
                }
            }
        }
    }

    if let Some((entity, sign_text)) = near_sign {
        help_text.override_text = Some("ENTER to read sign".to_string());

        if keyboard.just_pressed(KeyCode::Enter) {
            info!("Starting dialogue with sign, {} lines", sign_text.lines.len());
            dialogue.active = true;
            dialogue.lines = sign_text.lines.clone();
            dialogue.current_line = 0;
            dialogue.char_index = 0;
            dialogue.char_timer = 0.0;
            dialogue.sign_entity = Some(entity);
        }
    } else {
        if help_text.override_text.as_deref() == Some("ENTER to read sign") {
            help_text.override_text = None;
        }
    }
}

fn update_dialogue(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialogue: ResMut<DialogueState>,
    mut help_text: ResMut<HelpText>,
) {
    if !dialogue.active {
        return;
    }

    help_text.override_text = Some("ENTER to continue".to_string());

    dialogue.char_timer += time.delta_secs();
    let chars_to_add = (dialogue.char_timer * CHARS_PER_SECOND) as usize;
    
    if chars_to_add > 0 {
        dialogue.char_index += chars_to_add;
        if dialogue.current_line < dialogue.lines.len() {
            dialogue.char_index = dialogue.char_index.min(dialogue.lines[dialogue.current_line].len());
        }
        dialogue.char_timer = 0.0;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        dialogue.advance();
        if !dialogue.active {
            help_text.override_text = None;
        }
    }
}

fn spawn_dialogue_box(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dialogue: Res<DialogueState>,
    existing: Query<Entity, With<DialogueBox>>,
) {
    let should_exist = dialogue.active;
    let exists = !existing.is_empty();

    if should_exist && !exists {
        let font = asset_server.load("fonts/Tiny5-Regular.ttf");
        let bg_color = Color::srgb(0.863, 0.863, 0.863);
        let border_color = Color::srgb(0.0, 0.0, 0.0);
        let text_color = Color::srgb(0.0, 0.0, 0.0);

        // Border (slightly larger)
        commands.spawn((
            DialogueBox,
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(DIALOGUE_BOX_WIDTH + 4.0, DIALOGUE_BOX_HEIGHT + 4.0)),
                ..default()
            },
            Transform::from_xyz(0.0, DIALOGUE_Y_POS, 996.0),
            HIGH_RES_LAYERS,
        ));

        // Background
        commands.spawn((
            DialogueBox,
            Sprite {
                color: bg_color,
                custom_size: Some(Vec2::new(DIALOGUE_BOX_WIDTH, DIALOGUE_BOX_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(0.0, DIALOGUE_Y_POS, 997.0),
            HIGH_RES_LAYERS,
        ));

        // Text
        commands.spawn((
            DialogueBox,
            DialogueText,
            Text2d::new(""),
            TextFont {
                font,
                font_size: 20.0,
                ..default()
            },
            TextColor(text_color),
            Transform::from_xyz(0.0, DIALOGUE_Y_POS, 998.0),
            HIGH_RES_LAYERS,
        ));
    } else if !should_exist && exists {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

fn update_dialogue_text(
    dialogue: Res<DialogueState>,
    mut text_query: Query<&mut Text2d, With<DialogueText>>,
) {
    if !dialogue.active {
        return;
    }

    for mut text in &mut text_query {
        text.0 = dialogue.current_display_text();
    }
}

fn cleanup_dialogue(
    mut commands: Commands,
    mut dialogue: ResMut<DialogueState>,
    query: Query<Entity, With<DialogueBox>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    *dialogue = DialogueState::default();
}

pub(crate) fn sign_plugin_fn(app: &mut App) {
    app.init_resource::<DialogueState>()
        .register_type::<SignText>()
        .register_ldtk_entity::<SignBundle>("Sign")
        .add_systems(
            Update,
            (
                setup_sign_visuals,
                update_sign_interaction,
                update_dialogue,
                spawn_dialogue_box,
                update_dialogue_text,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_dialogue);
}
