use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LevelState {
    Locked,
    Unlocked,
    Done,
}

#[derive(Resource, Default)]
pub(crate) struct LevelProgress {
    pub(crate) completed: HashSet<usize>,
    pub(crate) selected: usize,
    pub(crate) current_playing: Option<usize>,
    pub(crate) total_levels: usize,
    pub(crate) level_names: Vec<String>,
}

impl LevelProgress {
    pub(crate) fn get_level_name(&self, level: usize) -> &str {
        self.level_names
            .get(level)
            .map(|s| s.as_str())
            .unwrap_or("???")
    }
}

#[derive(Resource, Default)]
pub(crate) struct PlayLdtkHandle(pub(crate) Option<Handle<LdtkProject>>);

impl LevelProgress {
    pub(crate) fn is_unlocked(&self, level: usize) -> bool {
        if level >= self.total_levels {
            return false;
        }
        if level == 0 {
            return true;
        }
        self.completed.contains(&(level - 1))
    }

    pub(crate) fn level_state(&self, level: usize) -> LevelState {
        if !self.is_unlocked(level) {
            return LevelState::Locked;
        }
        if self.completed.contains(&level) {
            return LevelState::Done;
        }
        LevelState::Unlocked
    }

    pub(crate) fn complete_level(&mut self, level: usize) {
        self.completed.insert(level);
        self.selected = level;
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    completed: Vec<usize>,
}

#[cfg(not(target_family = "wasm"))]
fn get_save_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|p| p.join("meaningless").join("save.json"))
}

#[cfg(not(target_family = "wasm"))]
fn load_from_disk() -> Option<SaveData> {
    let path = get_save_path()?;
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

#[cfg(not(target_family = "wasm"))]
fn save_to_disk(data: &SaveData) {
    if let Some(path) = get_save_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(data) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(target_family = "wasm")]
fn load_from_disk() -> Option<SaveData> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let json = storage.get_item("meaningless_save").ok()??;
    serde_json::from_str(&json).ok()
}

#[cfg(target_family = "wasm")]
fn save_to_disk(data: &SaveData) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(data) {
                let _ = storage.set_item("meaningless_save", &json);
            }
        }
    }
}

fn load_progress(mut progress: ResMut<LevelProgress>, mut last_saved: ResMut<LastSavedCount>) {
    if let Some(data) = load_from_disk() {
        let count = data.completed.len();
        progress.completed = data.completed.into_iter().collect();
        last_saved.0 = count;
    }
}

fn load_play_ldtk(asset_server: Res<AssetServer>, mut handle: ResMut<PlayLdtkHandle>) {
    if handle.0.is_none() {
        handle.0 = Some(asset_server.load("levels/play.ldtk"));
    }
}

fn detect_level_count(
    mut progress: ResMut<LevelProgress>,
    handle: Res<PlayLdtkHandle>,
    projects: Res<Assets<LdtkProject>>,
) {
    if progress.total_levels > 0 {
        return;
    }

    let Some(h) = &handle.0 else { return };
    let Some(project) = projects.get(h) else { return };

    let levels = &project.json_data().levels;
    let count = levels.len();
    progress.total_levels = count;
    
    progress.level_names = levels
        .iter()
        .map(|level| {
            for field in &level.field_instances {
                if field.identifier == "Name" {
                    if let bevy_ecs_ldtk::ldtk::FieldValue::String(Some(name)) = &field.value {
                        return name.clone();
                    }
                }
            }
            level.identifier.clone()
        })
        .collect();
}

#[derive(Resource, Default)]
struct LastSavedCount(usize);

fn save_progress_on_change(progress: Res<LevelProgress>, mut last_saved: ResMut<LastSavedCount>) {
    let current_count = progress.completed.len();
    if current_count != last_saved.0 && current_count > 0 {
        let data = SaveData {
            completed: progress.completed.iter().copied().collect(),
        };
        save_to_disk(&data);
        last_saved.0 = current_count;
    }
}

pub(crate) fn level_progress_plugin_fn(app: &mut App) {
    app.init_resource::<LevelProgress>()
        .init_resource::<PlayLdtkHandle>()
        .init_resource::<LastSavedCount>()
        .add_systems(Startup, (load_progress, load_play_ldtk))
        .add_systems(Update, (detect_level_count, save_progress_on_change));
}
