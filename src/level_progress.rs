use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::spiral::{level_to_pos, pos_to_level};

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
}

#[derive(Resource, Default)]
struct PlayLdtkHandle(Option<Handle<LdtkProject>>);

impl LevelProgress {
    pub(crate) fn is_unlocked(&self, level: usize) -> bool {
        if level >= self.total_levels {
            return false;
        }
        if level == 0 {
            return true;
        }
        if self.completed.contains(&level) {
            return true;
        }

        let pos = level_to_pos(level);
        for dir in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            if let Some(neighbor) = pos_to_level(pos + dir) {
                if self.completed.contains(&neighbor) {
                    return true;
                }
            }
        }
        false
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

    #[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
fn save_to_disk(data: &SaveData) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(data) {
                let _ = storage.set_item("meaningless_save", &json);
            }
        }
    }
}

fn load_progress(mut progress: ResMut<LevelProgress>) {
    if let Some(data) = load_from_disk() {
        info!("Loaded {} completed levels from save", data.completed.len());
        progress.completed = data.completed.into_iter().collect();
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

    let count = project.json_data().levels.len();
    progress.total_levels = count;
    info!("Detected {} levels from play.ldtk", count);
}

pub(crate) fn level_progress_plugin_fn(app: &mut App) {
    app.init_resource::<LevelProgress>()
        .init_resource::<PlayLdtkHandle>()
        .add_systems(Startup, (load_progress, load_play_ldtk))
        .add_systems(Update, detect_level_count);
}
