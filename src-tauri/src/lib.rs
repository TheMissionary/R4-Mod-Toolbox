mod archive;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanResult {
    pub is_valid_game_path: bool,
    pub game_version: String,
    pub archive_count: usize,
    pub cet_count: usize,
    pub red4ext_count: usize,
    pub redscript_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CetPluginItem {
    pub name: String,
    pub path: String,
    pub has_init: bool,
    pub size_bytes: u64,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Red4extPluginItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedScriptItem {
    pub name: String,
    pub path: String,
    pub reds_count: usize,
    pub size_bytes: u64,
    pub enabled: bool,
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += calculate_dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn count_reds_files(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                count += count_reds_files(&p);
            } else if p.extension().map_or(false, |ext| ext == "reds") {
                count += 1;
            }
        }
    }
    count
}

#[tauri::command]
fn scan_game_directory(game_path: String) -> Result<ScanResult, String> {
    let root = PathBuf::from(&game_path);
    let exe_path = root.join("bin").join("x64").join("Cyberpunk2077.exe");
    let is_valid = exe_path.exists();

    let archive_dir = root.join("archive").join("pc").join("mod");
    let archive_count = if archive_dir.exists() {
        fs::read_dir(&archive_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "archive"))
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    let cet_dir = root.join("bin").join("x64").join("plugins").join("cyber_engine_tweaks").join("mods");
    let cet_count = if cet_dir.exists() {
        fs::read_dir(&cet_dir)
            .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
            .unwrap_or(0)
    } else {
        0
    };

    let red4ext_dir = root.join("red4ext").join("plugins");
    let red4ext_count = if red4ext_dir.exists() {
        fs::read_dir(&red4ext_dir)
            .map(|entries| entries.flatten().count())
            .unwrap_or(0)
    } else {
        0
    };

    let redscript_dir = root.join("r6").join("scripts");
    let redscript_count = if redscript_dir.exists() {
        fs::read_dir(&redscript_dir)
            .map(|entries| entries.flatten().count())
            .unwrap_or(0)
    } else {
        0
    };

    Ok(ScanResult {
        is_valid_game_path: is_valid,
        game_version: "2.31".to_string(),
        archive_count,
        cet_count,
        red4ext_count,
        redscript_count,
    })
}

#[tauri::command]
fn get_archive_details(game_path: String) -> Result<archive::ArchiveScanReport, String> {
    archive::scan_archives(&game_path)
}

#[tauri::command]
fn toggle_mod_state(game_path: String, mod_name: String, enable: bool) -> Result<(), String> {
    archive::toggle_mod(&game_path, &mod_name, enable)
}

#[tauri::command]
fn save_load_order(game_path: String, load_order: Vec<String>) -> Result<archive::ArchiveScanReport, String> {
    archive::save_modlist(&game_path, load_order)
}

#[tauri::command]
fn save_categories_config(game_path: String, config_json: String) -> Result<(), String> {
    archive::save_categories(&game_path, config_json)
}

#[tauri::command]
fn load_categories_config(game_path: String) -> Result<String, String> {
    archive::load_categories(&game_path)
}

#[tauri::command]
fn start_directory_watcher(_game_path: String) -> Result<(), String> {
    // Directory watcher integration stub
    Ok(())
}

#[tauri::command]
fn get_cet_details(game_path: String) -> Result<Vec<CetPluginItem>, String> {
    let root = PathBuf::from(&game_path);
    let active_dir = root.join("bin").join("x64").join("plugins").join("cyber_engine_tweaks").join("mods");
    let disabled_dir = root.join("Disabled_Mods").join("cet");

    let mut list = Vec::new();

    if active_dir.exists() && active_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&active_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let has_init = p.join("init.lua").exists();
                    let size_bytes = calculate_dir_size(&p);
                    list.push(CetPluginItem {
                        name,
                        path: p.to_string_lossy().to_string(),
                        has_init,
                        size_bytes,
                        enabled: true,
                    });
                }
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let has_init = p.join("init.lua").exists();
                    let size_bytes = calculate_dir_size(&p);
                    list.push(CetPluginItem {
                        name,
                        path: p.to_string_lossy().to_string(),
                        has_init,
                        size_bytes,
                        enabled: false,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
fn get_red4ext_details(game_path: String) -> Result<Vec<Red4extPluginItem>, String> {
    let root = PathBuf::from(&game_path);
    let active_dir = root.join("red4ext").join("plugins");
    let disabled_dir = root.join("Disabled_Mods").join("red4ext");

    let mut list = Vec::new();

    if active_dir.exists() && active_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&active_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let size_bytes = if p.is_dir() {
                    calculate_dir_size(&p)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                list.push(Red4extPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    size_bytes,
                    enabled: true,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let size_bytes = if p.is_dir() {
                    calculate_dir_size(&p)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                list.push(Red4extPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    size_bytes,
                    enabled: false,
                });
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
fn get_redscript_details(game_path: String) -> Result<Vec<RedScriptItem>, String> {
    let root = PathBuf::from(&game_path);
    let active_dir = root.join("r6").join("scripts");
    let disabled_dir = root.join("Disabled_Mods").join("redscript");

    let mut list = Vec::new();

    if active_dir.exists() && active_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&active_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let (reds_count, size_bytes) = if p.is_dir() {
                    (count_reds_files(&p), calculate_dir_size(&p))
                } else {
                    let is_reds = p.extension().map_or(false, |ext| ext == "reds");
                    (
                        if is_reds { 1 } else { 0 },
                        entry.metadata().map(|m| m.len()).unwrap_or(0),
                    )
                };
                list.push(RedScriptItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    reds_count,
                    size_bytes,
                    enabled: true,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let (reds_count, size_bytes) = if p.is_dir() {
                    (count_reds_files(&p), calculate_dir_size(&p))
                } else {
                    let is_reds = p.extension().map_or(false, |ext| ext == "reds");
                    (
                        if is_reds { 1 } else { 0 },
                        entry.metadata().map(|m| m.len()).unwrap_or(0),
                    )
                };
                list.push(RedScriptItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    reds_count,
                    size_bytes,
                    enabled: false,
                });
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
fn toggle_plugin_state(
    game_path: String,
    mod_type: String,
    mod_name: String,
    enable: bool,
) -> Result<(), String> {
    let root = PathBuf::from(&game_path);

    let (active_base, disabled_base) = match mod_type.as_str() {
        "cet" => (
            root.join("bin").join("x64").join("plugins").join("cyber_engine_tweaks").join("mods"),
            root.join("Disabled_Mods").join("cet"),
        ),
        "red4ext" => (
            root.join("red4ext").join("plugins"),
            root.join("Disabled_Mods").join("red4ext"),
        ),
        "redscript" => (
            root.join("r6").join("scripts"),
            root.join("Disabled_Mods").join("redscript"),
        ),
        _ => return Err(format!("Unsupported mod type: {}", mod_type)),
    };

    let (source, target) = if enable {
        (disabled_base.join(&mod_name), active_base.join(&mod_name))
    } else {
        (active_base.join(&mod_name), disabled_base.join(&mod_name))
    };

    if !source.exists() {
        return Err(format!("Source path does not exist: {:?}", source));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::rename(&source, &target).map_err(|e| {
        format!("Failed to move {:?} to {:?}: {}", source, target, e)
    })?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_game_directory,
            get_archive_details,
            toggle_mod_state,
            save_load_order,
            save_categories_config,
            load_categories_config,
            start_directory_watcher,
            get_cet_details,
            get_red4ext_details,
            get_redscript_details,
            toggle_plugin_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}