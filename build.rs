use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::{env, fs};

const ASEPRITE_BIN: &str = "/Applications/Aseprite.app/Contents/MacOS/aseprite";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let anim_defs = scan_anim_defs("anim_defs");
    if anim_defs.is_empty() {
        let stub = "pub fn register_all_anims(_app: &mut bevy::prelude::App) {}\n";
        fs::write(Path::new(&out_dir).join("animations.rs"), stub).unwrap();
        println!("cargo:rerun-if-changed=anim_defs");
        return;
    }

    let processed = process_animations(&anim_defs);
    let code = generate_code(&processed);

    fs::write(Path::new(&out_dir).join("animations.rs"), &code).unwrap();

    println!("cargo:rerun-if-changed=anim_defs");
    println!("cargo:rerun-if-changed=assets");
}

#[derive(Debug, Clone)]
struct AnimDef {
    name: String,
    file: String,
    default: String,
    exclude: Option<String>,
    fps: Option<u32>,
    variants: HashMap<String, VariantOverride>,
    source_file: String,
    toml_content: String,
}

#[derive(Debug, Clone, Default)]
struct VariantOverride {
    fps: Option<u32>,
    next: Option<String>,
}

#[derive(Debug)]
struct ProcessedAnim {
    def: AnimDef,
    variants: Vec<ProcessedVariant>,
}

#[derive(Debug)]
struct ProcessedVariant {
    name: String,
    tag: String,
    fps: Option<u32>,
    next: AnimNext,
    frame_count: usize,
    frame_width: u32,
    frame_height: u32,
    asset_path: String,
}

#[derive(Debug, Clone)]
enum AnimNext {
    Loop,
    State(String),
    Remove,
    Despawn,
}

fn scan_anim_defs(dir: &str) -> Vec<AnimDef> {
    let mut defs = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return defs;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            let content = fs::read_to_string(&path).expect("Failed to read TOML file");
            defs.extend(parse_toml(&content, &path));
        }
    }
    defs
}

fn find_line_number(content: &str, search_key: &str, search_value: &str) -> Option<usize> {
    for (i, line) in content.lines().enumerate() {
        if line.contains(search_key) && line.contains(search_value) {
            return Some(i + 1);
        }
    }
    None
}

fn parse_toml(content: &str, path: &Path) -> Vec<AnimDef> {
    let table: toml::Table = content.parse().unwrap_or_else(|e| {
        panic!("{}:1: Failed to parse TOML: {}", path.display(), e);
    });

    let mut defs = Vec::new();
    let Some(anims) = table.get("anim").and_then(|v| v.as_array()) else {
        return defs;
    };

    for anim in anims {
        let anim = anim.as_table().expect("anim entry must be a table");

        let name = anim
            .get("name")
            .and_then(|v| v.as_str())
            .expect("anim.name required")
            .to_string();
        let file = anim
            .get("file")
            .and_then(|v| v.as_str())
            .expect("anim.file required")
            .to_string();
        let default = anim
            .get("default")
            .and_then(|v| v.as_str())
            .expect("anim.default required")
            .to_string();
        let exclude = Some(
            anim.get("exclude")
                .and_then(|v| v.as_str())
                .unwrap_or("_")
                .to_string(),
        );
        let fps = anim
            .get("fps")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32);

        let mut variants = HashMap::new();

        if let Some(next_table) = anim.get("next").and_then(|v| v.as_table()) {
            for (var_name, var_val) in next_table {
                let next_value = var_val.as_str().map(String::from);
                variants.insert(
                    var_name.clone(),
                    VariantOverride {
                        fps: None,
                        next: next_value,
                    },
                );
            }
        }

        if let Some(vars) = anim.get("variants").and_then(|v| v.as_table()) {
            for (var_name, var_val) in vars {
                let var_table = var_val.as_table().expect("variant must be a table");
                let var_fps = var_table
                    .get("fps")
                    .and_then(|v| v.as_integer())
                    .map(|v| v as u32);
                let var_next = var_table
                    .get("next")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let existing = variants.get(var_name);
                variants.insert(
                    var_name.clone(),
                    VariantOverride {
                        fps: var_fps.or(existing.and_then(|e| e.fps)),
                        next: var_next.or(existing.and_then(|e| e.next.clone())),
                    },
                );
            }
        }

        let source_file = fs::canonicalize(path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.display().to_string());

        defs.push(AnimDef {
            name,
            file,
            default,
            exclude,
            fps,
            variants,
            source_file,
            toml_content: content.to_string(),
        });
    }
    defs
}

fn process_animations(defs: &[AnimDef]) -> Vec<ProcessedAnim> {
    defs.iter()
        .map(|def| process_single_animation(def, &def.toml_content))
        .collect()
}

fn process_single_animation(def: &AnimDef, toml_content: &str) -> ProcessedAnim {
    let absolute_file = fs::canonicalize(&def.file).unwrap_or_else(|_| {
        let line = find_line_number(toml_content, "file", &def.file).unwrap_or(1);
        panic!("{}:{}: File not found: {}", def.source_file, line, def.file)
    });
    let absolute_file_str = absolute_file.to_string_lossy().to_string();

    let tags = list_tags(&absolute_file_str);
    let filtered_tags: Vec<String> = tags
        .into_iter()
        .filter(|tag| {
            if let Some(ref prefix) = def.exclude {
                !tag.starts_with(prefix)
            } else {
                true
            }
        })
        .collect();

    let struct_snake = to_snake_case(&def.name);
    let output_dir = format!("assets/_generated/anim/{}", struct_snake);
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let exports =
        batch_export_sprite_sheets(&absolute_file_str, &filtered_tags, &output_dir, &def.exclude);

    let variant_names: Vec<String> = filtered_tags.iter().map(|t| to_pascal_case(t)).collect();

    let mut variants: Vec<ProcessedVariant> = Vec::new();
    for tag in &filtered_tags {
        let variant_name = to_pascal_case(tag);
        let export = exports.iter().find(|e| &e.tag == tag).unwrap_or_else(|| {
            panic!(
                "{}:1: Missing export for tag '{}' in {}",
                def.source_file, tag, def.file
            )
        });

        let override_info = def.variants.get(&variant_name);
        let fps = override_info.and_then(|o| o.fps).or(def.fps);

        let next = if let Some(next_str) = override_info.and_then(|o| o.next.as_ref()) {
            match next_str.as_str() {
                "DESPAWN" | "Despawn" => AnimNext::Despawn,
                "REMOVE" | "Remove" => AnimNext::Remove,
                other => {
                    if !variant_names.contains(&other.to_string()) {
                        let line = find_line_number(toml_content, &variant_name, next_str)
                            .or_else(|| find_line_number(toml_content, "next", next_str))
                            .unwrap_or(1);
                        panic!(
                            "{}:{}: Unknown next variant '{}' for {}. Available: {:?}",
                            def.source_file, line, other, variant_name, variant_names
                        );
                    }
                    AnimNext::State(other.to_string())
                }
            }
        } else {
            AnimNext::Loop
        };

        variants.push(ProcessedVariant {
            name: variant_name,
            tag: tag.clone(),
            fps,
            next,
            frame_count: export.frame_count,
            frame_width: export.frame_width,
            frame_height: export.frame_height,
            asset_path: export.asset_path.clone(),
        });
    }

    let default_exists = variants.iter().any(|v| v.name == def.default);
    if !default_exists {
        let line = find_line_number(toml_content, "default", &def.default).unwrap_or(1);
        panic!(
            "{}:{}: Default variant '{}' not found for {}. Available: {:?}",
            def.source_file,
            line,
            def.default,
            def.name,
            variants.iter().map(|v| &v.name).collect::<Vec<_>>()
        );
    }

    ProcessedAnim {
        def: def.clone(),
        variants,
    }
}

fn generate_code(anims: &[ProcessedAnim]) -> String {
    let mut code = String::new();

    for anim in anims {
        code.push_str(&generate_single_anim(anim));
        code.push_str("\n\n");
    }

    code.push_str("pub fn register_all_anims(app: &mut bevy::prelude::App) {\n");
    for anim in anims {
        code.push_str(&format!(
            "    crate::anim::register_anim::<{}>(app);\n",
            anim.def.name
        ));
    }
    code.push_str("}\n");

    code
}

fn generate_single_anim(anim: &ProcessedAnim) -> String {
    let name = &anim.def.name;
    let table_name = format!("{}_ANIM_TABLE", name.to_uppercase());

    let variant_names: Vec<&str> = anim.variants.iter().map(|v| v.name.as_str()).collect();
    let variant_indices: HashMap<&str, usize> = variant_names
        .iter()
        .enumerate()
        .map(|(i, n)| (*n, i))
        .collect();

    let enum_variants = variant_names.join(",\n    ");

    let table_entries: Vec<String> = anim
        .variants
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let fps_str = match v.fps {
                Some(f) => format!("Some({}.0)", f),
                None => "None".to_string(),
            };
            let next_str = match &v.next {
                AnimNext::Loop => format!("crate::anim::AnimNextIndex::Index({})", idx),
                AnimNext::State(s) => {
                    let target_idx = variant_indices.get(s.as_str()).unwrap_or_else(|| {
                        panic!("Unknown next variant '{}' in {}", s, name);
                    });
                    format!("crate::anim::AnimNextIndex::Index({})", target_idx)
                }
                AnimNext::Remove => "crate::anim::AnimNextIndex::Remove".to_string(),
                AnimNext::Despawn => "crate::anim::AnimNextIndex::Despawn".to_string(),
            };
            format!(
                r#"    crate::anim::AnimVariant {{
        tag: "{}",
        fps: {},
        frame_count: {},
        frame_size: ({}, {}),
        asset_path: "{}",
        next: {},
    }}"#,
                v.tag, fps_str, v.frame_count, v.frame_width, v.frame_height, v.asset_path, next_str
            )
        })
        .collect();

    let match_index_arms: Vec<String> = anim
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| format!("            Self::{} => {}", v.name, i))
        .collect();

    let match_from_index_arms: Vec<String> = anim
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| format!("            {} => Self::{}", i, v.name))
        .collect();

    format!(
        r#"#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum {name} {{
    {variants}
}}

impl Default for {name} {{
    fn default() -> Self {{
        Self::{default}
    }}
}}

static {table_name}: &[crate::anim::AnimVariant] = &[
{table_entries}
];

impl crate::anim::Anim for {name} {{
    fn table() -> &'static [crate::anim::AnimVariant] {{
        {table_name}
    }}

    fn index(&self) -> usize {{
        match self {{
{match_index},
        }}
    }}

    fn from_index(index: usize) -> Self {{
        match index {{
{match_from_index},
            _ => Self::default()
        }}
    }}
}}"#,
        name = name,
        variants = enum_variants,
        default = anim.def.default,
        table_name = table_name,
        table_entries = table_entries.join(",\n"),
        match_index = match_index_arms.join(",\n"),
        match_from_index = match_from_index_arms.join(",\n"),
    )
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

struct ExportInfo {
    tag: String,
    frame_count: usize,
    frame_width: u32,
    frame_height: u32,
    asset_path: String,
}

fn run_aseprite_cmd(args: &[&str]) -> String {
    if !Path::new(ASEPRITE_BIN).exists() {
        panic!("Aseprite not found at: {}", ASEPRITE_BIN);
    }

    for attempt in 0..5 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let output = Command::new(ASEPRITE_BIN)
            .args(args)
            .output()
            .expect("Failed to execute Aseprite");

        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).to_string();
        }

        if output.status.code() == Some(255) {
            continue;
        }

        panic!(
            "Aseprite command failed: {:?}\n{}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    panic!("Aseprite command failed after 5 retries: {:?}", args);
}

fn get_mtime(path: &str) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

fn is_cache_valid(source_file: &str, output_files: &[String]) -> bool {
    let Some(source_mtime) = get_mtime(source_file) else {
        return false;
    };
    for output in output_files {
        let Some(output_mtime) = get_mtime(output) else {
            return false;
        };
        if source_mtime > output_mtime {
            return false;
        }
    }
    true
}

fn list_tags(file: &str) -> Vec<String> {
    let cache_path = env::temp_dir().join(format!(
        "aseprite_tags_{}.txt",
        file.replace(['/', '\\', '.', ' '], "_")
    ));

    if let (Some(src_mtime), Ok(cache_content)) = (get_mtime(file), fs::read_to_string(&cache_path))
    {
        if let Some(cache_mtime) = get_mtime(cache_path.to_str().unwrap()) {
            if cache_mtime > src_mtime {
                return cache_content.lines().map(|s| s.to_string()).collect();
            }
        }
    }

    let output = run_aseprite_cmd(&["-b", "--list-tags", file]);
    let tags: Vec<String> = output.split_whitespace().map(|s| s.to_string()).collect();
    let _ = fs::write(&cache_path, tags.join("\n"));
    tags
}

fn list_layers(file: &str) -> Vec<String> {
    let cache_path = env::temp_dir().join(format!(
        "aseprite_layers_{}.txt",
        file.replace(['/', '\\', '.', ' '], "_")
    ));

    if let (Some(src_mtime), Ok(cache_content)) = (get_mtime(file), fs::read_to_string(&cache_path))
    {
        if let Some(cache_mtime) = get_mtime(cache_path.to_str().unwrap()) {
            if cache_mtime > src_mtime {
                return cache_content.lines().map(|s| s.to_string()).collect();
            }
        }
    }

    let output = run_aseprite_cmd(&["-b", "--list-layers", file]);
    let layers: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    let _ = fs::write(&cache_path, layers.join("\n"));
    layers
}

fn batch_export_sprite_sheets(
    file: &str,
    tags: &[String],
    output_dir: &str,
    exclude_prefix: &Option<String>,
) -> Vec<ExportInfo> {
    let output_files: Vec<String> = tags
        .iter()
        .map(|tag| format!("{}/{}.png", output_dir, tag))
        .collect();

    let cache_path = format!("{}/_cache.txt", output_dir);

    if is_cache_valid(file, &output_files) && Path::new(&cache_path).exists() {
        if let Ok(cache_content) = fs::read_to_string(&cache_path) {
            let cached = parse_cache(&cache_content, tags, output_dir);
            if cached.len() == tags.len() {
                return cached;
            }
        }
    }

    let layers_to_ignore: Vec<String> = if let Some(prefix) = exclude_prefix {
        list_layers(file)
            .into_iter()
            .filter(|layer| layer.starts_with(prefix))
            .collect()
    } else {
        vec![]
    };

    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join("batch_export.lua");
    let json_path = temp_dir.join("batch_export_meta.json");

    let tags_lua: String = tags
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");

    let ignore_layers: String = layers_to_ignore
        .iter()
        .map(|l| format!("[\"{}\"] = true", l))
        .collect::<Vec<_>>()
        .join(", ");

    let lua_script = format!(
        r#"
local spr = app.sprite
local ignoreLayers = {{ {ignore_layers} }}
local tags = {{ {tags} }}
local outputDir = "{output_dir}"
local results = {{}}

local function setLayerVisibility(layers, visible)
    for _, layer in ipairs(layers) do
        if ignoreLayers[layer.name] then
            layer.isVisible = visible
        end
        if layer.layers then
            setLayerVisibility(layer.layers, visible)
        end
    end
end

setLayerVisibility(spr.layers, false)

for _, tagName in ipairs(tags) do
    local tag = nil
    for _, t in ipairs(spr.tags) do
        if t.name == tagName then
            tag = t
            break
        end
    end
    if tag then
        local outputPath = outputDir .. "/" .. tagName .. ".png"
        app.command.ExportSpriteSheet {{
            ui = false,
            type = SpriteSheetType.HORIZONTAL,
            textureFilename = outputPath,
            tag = tagName,
        }}
        local frameCount = tag.frames
        local w = spr.width
        local h = spr.height
        table.insert(results, string.format("%s,%d,%d,%d", tagName, frameCount, w, h))
    end
end

local f = io.open("{json_path}", "w")
f:write(table.concat(results, "\n"))
f:close()
"#,
        ignore_layers = ignore_layers,
        tags = tags_lua,
        output_dir = output_dir.replace('\\', "/"),
        json_path = json_path.to_str().unwrap().replace('\\', "/"),
    );

    fs::write(&script_path, &lua_script).expect("Failed to write Lua script");
    run_aseprite_cmd(&["-b", file, "--script", script_path.to_str().unwrap()]);

    let meta_content =
        fs::read_to_string(&json_path).expect("Failed to read batch export metadata");

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&json_path);

    let mut results = Vec::new();
    for line in meta_content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let tag = parts[0].to_string();
            let frame_count: usize = parts[1].parse().unwrap_or(1);
            let frame_width: u32 = parts[2].parse().unwrap_or(0);
            let frame_height: u32 = parts[3].parse().unwrap_or(0);
            let output_path = format!("{}/{}.png", output_dir, tag);
            let asset_path = output_path
                .strip_prefix("assets/")
                .unwrap_or(&output_path)
                .to_string();
            results.push(ExportInfo {
                tag,
                frame_count,
                frame_width,
                frame_height,
                asset_path,
            });
        }
    }

    let cache_content = results
        .iter()
        .map(|r| {
            format!(
                "{},{},{},{}",
                r.tag, r.frame_count, r.frame_width, r.frame_height
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(&cache_path, cache_content);

    results
}

fn parse_cache(content: &str, tags: &[String], output_dir: &str) -> Vec<ExportInfo> {
    let mut results = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let tag = parts[0].to_string();
            if tags.contains(&tag) {
                let frame_count: usize = parts[1].parse().unwrap_or(1);
                let frame_width: u32 = parts[2].parse().unwrap_or(0);
                let frame_height: u32 = parts[3].parse().unwrap_or(0);
                let output_path = format!("{}/{}.png", output_dir, tag);
                let asset_path = output_path
                    .strip_prefix("assets/")
                    .unwrap_or(&output_path)
                    .to_string();
                results.push(ExportInfo {
                    tag,
                    frame_count,
                    frame_width,
                    frame_height,
                    asset_path,
                });
            }
        }
    }
    results
}
