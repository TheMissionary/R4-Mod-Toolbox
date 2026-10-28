mod archive;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppThemeColors {
    pub accent: String,
    pub accent_hover: String,
    pub bg: String,
    pub surface: String,
    pub card: String,
    pub border: String,
    pub text_muted: String,
    pub text_primary: String,
}

impl Default for AppThemeColors {
    fn default() -> Self {
        Self {
            accent: "#76b900".to_string(),
            accent_hover: "#88d600".to_string(),
            bg: "#121517".to_string(),
            surface: "#181c20".to_string(),
            card: "#1e2328".to_string(),
            border: "#2a323d".to_string(),
            text_muted: "#94a3b8".to_string(),
            text_primary: "#ffffff".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub mode: String,
    pub dark_colors: AppThemeColors,
    pub light_colors: AppThemeColors,
    #[serde(default)]
    pub font_family_base: String,
    #[serde(default)]
    pub font_family_mods: String,
    #[serde(default)]
    pub font_family: String,
    #[serde(default)]
    pub is_compact: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: "dark".to_string(),
            dark_colors: AppThemeColors::default(),
            light_colors: AppThemeColors {
                accent: "#5a8f00".to_string(),
                accent_hover: "#4c7a00".to_string(),
                bg: "#f1f5f9".to_string(),
                surface: "#f8fafc".to_string(),
                card: "#ffffff".to_string(),
                border: "#cbd5e1".to_string(),
                text_muted: "#64748b".to_string(),
                text_primary: "#0f172a".to_string(),
            },
            font_family_base: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif".to_string(),
            font_family_mods: "\"Cascadia Code\", \"Consolas\", monospace".to_string(),
            font_family: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif".to_string(),
            is_compact: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub target_game_path: String,
    pub active_tab: String,
    pub show_conflict_summary: bool,
    #[serde(default)]
    pub window_width: Option<f64>,
    #[serde(default)]
    pub window_height: Option<f64>,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default)]
    pub recent_days_threshold: Option<u32>,
    #[serde(default)]
    pub new_mods_hours: Option<u32>,
    #[serde(default)]
    pub show_new_mods: Option<bool>,
    #[serde(default)]
    pub custom_text_editor_path: Option<String>,
    #[serde(default)]
    pub custom_photo_path: Option<String>,
    #[serde(default)]
    pub enable_organizer_view: Option<bool>,
    #[serde(default)]
    pub show_category_labels_in_load_order: Option<bool>,
    pub theme: ThemeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_game_path: "".to_string(),
            active_tab: "archive".to_string(),
            show_conflict_summary: true,
            window_width: Some(1600.0),
            window_height: Some(1000.0),
            window_x: None,
            window_y: None,
            recent_days_threshold: Some(30),
            new_mods_hours: Some(6),
            show_new_mods: Some(false),
            custom_text_editor_path: None,
            custom_photo_path: None,
            enable_organizer_view: Some(false),
            show_category_labels_in_load_order: Some(true),
            theme: ThemeConfig::default(),
        }
    }
}

fn get_app_config_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("r4-mod-toolbox").join("config.json")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join("AppData").join("Roaming").join("r4-mod-toolbox").join("config.json")
    } else {
        PathBuf::from("config.json")
    }
}

fn get_ledger_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("r4-mod-toolbox").join("mod_history.json")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join("AppData").join("Roaming").join("r4-mod-toolbox").join("mod_history.json")
    } else {
        PathBuf::from("mod_history.json")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LedgerEntry {
    pub name: String,
    pub mod_type: String,
    pub path: String,
    pub first_seen: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub extension: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanResult {
    pub is_valid_game_path: bool,
    pub game_version: String,
    pub archive_active: usize,
    pub archive_total: usize,
    pub cet_active: usize,
    pub cet_total: usize,
    pub red4ext_active: usize,
    pub red4ext_total: usize,
    pub redscript_active: usize,
    pub redscript_total: usize,
    pub r6tweaks_active: usize,
    pub r6tweaks_total: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CetPluginItem {
    pub name: String,
    pub path: String,
    pub has_init: bool,
    pub size_bytes: u64,
    pub file_count: usize,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Red4extPluginItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedScriptItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct R6TweaksItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
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

fn count_all_files(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                count += count_all_files(&p);
            } else {
                count += 1;
            }
        }
    }
    count
}

fn prune_logs(app: &tauri::App) {
    if let Ok(log_dir) = app.path().app_log_dir() {
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            let mut logs = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            logs.push((path, modified));
                        }
                    }
                }
            }
            logs.sort_by(|a, b| b.1.cmp(&a.1));
            for (path, _) in logs.into_iter().skip(5) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[tauri::command]
fn load_app_config() -> Result<AppConfig, String> {
    let path = get_app_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return Ok(config);
            }
        }
    }
    Ok(AppConfig::default())
}

#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<(), String> {
    let path = get_app_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let serialized = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

fn resolve_game_root(candidate: &Path) -> Option<PathBuf> {
    if candidate.join("bin").join("x64").join("Cyberpunk2077.exe").exists() {
        return Some(candidate.to_path_buf());
    }
    if candidate.join("Cyberpunk2077.exe").exists() {
        if let Some(parent) = candidate.parent() {
            if let Some(grandparent) = parent.parent() {
                if grandparent.join("bin").join("x64").join("Cyberpunk2077.exe").exists() {
                    return Some(grandparent.to_path_buf());
                }
            }
        }
    }
    if candidate.join("x64").join("Cyberpunk2077.exe").exists() {
        if let Some(parent) = candidate.parent() {
            if parent.join("bin").join("x64").join("Cyberpunk2077.exe").exists() {
                return Some(parent.to_path_buf());
            }
        }
    }
    None
}

#[tauri::command]
fn validate_and_resolve_game_path(candidate_path: String) -> Result<Option<String>, String> {
    let trimmed = candidate_path.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let path = PathBuf::from(trimmed);
    let resolved = resolve_game_root(&path).map(|p| p.to_string_lossy().to_string());
    Ok(resolved)
}

fn update_ledger(game_path: &str) {
    let mut current_mods: Vec<LedgerEntry> = Vec::new();

    if let Ok(report) = archive::scan_archives(game_path) {
        for item in report.archives {
            if !item.is_delimiter {
                current_mods.push(LedgerEntry {
                    name: item.file_name,
                    mod_type: "archive".to_string(),
                    path: item.path,
                    first_seen: 0,
                });
            }
        }
    }

    if let Ok(cet_list) = get_cet_details(game_path.to_string()) {
        for item in cet_list {
            current_mods.push(LedgerEntry {
                name: item.name,
                mod_type: "cet".to_string(),
                path: item.path,
                first_seen: 0,
            });
        }
    }

    if let Ok(r4e_list) = get_red4ext_details(game_path.to_string()) {
        for item in r4e_list {
            current_mods.push(LedgerEntry {
                name: item.name,
                mod_type: "red4ext".to_string(),
                path: item.path,
                first_seen: 0,
            });
        }
    }

    if let Ok(rs_list) = get_redscript_details(game_path.to_string()) {
        for item in rs_list {
            current_mods.push(LedgerEntry {
                name: item.name,
                mod_type: "redscript".to_string(),
                path: item.path,
                first_seen: 0,
            });
        }
    }

    if let Ok(tw_list) = get_r6tweaks_details(game_path.to_string()) {
        for item in tw_list {
            current_mods.push(LedgerEntry {
                name: item.name,
                mod_type: "r6tweaks".to_string(),
                path: item.path,
                first_seen: 0,
            });
        }
    }

    let old_ledger = get_ledger_data();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let mut new_ledger = Vec::new();

    for mut cmod in current_mods {
        if let Some(existing) = old_ledger.iter().find(|e| e.name == cmod.name && e.mod_type == cmod.mod_type) {
            cmod.first_seen = existing.first_seen;
        } else {
            cmod.first_seen = now;
        }
        new_ledger.push(cmod);
    }

    let _ = save_ledger_data(&new_ledger);
}

#[tauri::command]
fn scan_game_directory(game_path: String) -> Result<ScanResult, String> {
    info!("Initiating full game directory scan at: {}", game_path);
    if game_path.trim().is_empty() {
        return Ok(ScanResult {
            is_valid_game_path: false,
            game_version: "2.31".to_string(),
            archive_active: 0, archive_total: 0,
            cet_active: 0, cet_total: 0,
            red4ext_active: 0, red4ext_total: 0,
            redscript_active: 0, redscript_total: 0,
            r6tweaks_active: 0, r6tweaks_total: 0,
        });
    }

    let root = PathBuf::from(&game_path);
    let exe_path = root.join("bin").join("x64").join("Cyberpunk2077.exe");
    let is_valid = exe_path.exists();

    update_ledger(&game_path);

    let archive_dir = root.join("archive").join("pc").join("mod");
    let mut archive_active = 0;
    let mut archive_total = 0;
    if archive_dir.exists() {
        if let Ok(entries) = fs::read_dir(&archive_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("[CAT] ") || name.starts_with("[DIR] ") { continue; }
                if name.ends_with(".archive") {
                    archive_active += 1;
                    archive_total += 1;
                } else if name.ends_with(".archive.disabled") {
                    archive_total += 1;
                }
            }
        }
    }

    let cet_dir = root.join("bin").join("x64").join("plugins").join("cyber_engine_tweaks").join("mods");
    let cet_disabled_dir = root.join("Disabled_Mods").join("cet");
    let cet_active = if cet_dir.exists() { fs::read_dir(&cet_dir).map(|e| e.flatten().filter(|e| e.path().is_dir()).count()).unwrap_or(0) } else { 0 };
    let cet_disabled = if cet_disabled_dir.exists() { fs::read_dir(&cet_disabled_dir).map(|e| e.flatten().filter(|e| e.path().is_dir()).count()).unwrap_or(0) } else { 0 };
    let cet_total = cet_active + cet_disabled;

    let red4ext_dir = root.join("red4ext").join("plugins");
    let red4ext_disabled_dir = root.join("Disabled_Mods").join("red4ext");
    let red4ext_active = if red4ext_dir.exists() { fs::read_dir(&red4ext_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let red4ext_disabled = if red4ext_disabled_dir.exists() { fs::read_dir(&red4ext_disabled_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let red4ext_total = red4ext_active + red4ext_disabled;

    let redscript_dir = root.join("r6").join("scripts");
    let redscript_disabled_dir = root.join("Disabled_Mods").join("redscript");
    let redscript_active = if redscript_dir.exists() { fs::read_dir(&redscript_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let redscript_disabled = if redscript_disabled_dir.exists() { fs::read_dir(&redscript_disabled_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let redscript_total = redscript_active + redscript_disabled;

    let r6tweaks_dir = root.join("r6").join("tweaks");
    let r6tweaks_disabled_dir = root.join("Disabled_Mods").join("r6tweaks");
    let r6tweaks_active = if r6tweaks_dir.exists() { fs::read_dir(&r6tweaks_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let r6tweaks_disabled = if r6tweaks_disabled_dir.exists() { fs::read_dir(&r6tweaks_disabled_dir).map(|e| e.flatten().count()).unwrap_or(0) } else { 0 };
    let r6tweaks_total = r6tweaks_active + r6tweaks_disabled;

    Ok(ScanResult {
        is_valid_game_path: is_valid,
        game_version: "2.31".to_string(),
        archive_active, archive_total,
        cet_active, cet_total,
        red4ext_active, red4ext_total,
        redscript_active, redscript_total,
        r6tweaks_active, r6tweaks_total,
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
fn load_labels_config(game_path: String) -> Result<archive::LabelsConfig, String> {
    archive::load_labels(&game_path)
}

#[tauri::command]
fn save_labels_config(game_path: String, config: archive::LabelsConfig) -> Result<(), String> {
    archive::save_labels(&game_path, config)
}

#[tauri::command]
fn create_physical_folder(game_path: String, folder_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::create_physical_folder(&game_path, &folder_name)
}

#[tauri::command]
fn delete_physical_folder(game_path: String, folder_file_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::delete_physical_folder(&game_path, &folder_file_name)
}

#[tauri::command]
fn rename_physical_folder(game_path: String, old_file_name: String, new_folder_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::rename_physical_folder(&game_path, &old_file_name, &new_folder_name)
}

#[tauri::command]
fn link_xl_to_archive(game_path: String, xl_name: String, archive_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::link_xl_to_archive(&game_path, &xl_name, &archive_name)
}

#[tauri::command]
fn unlink_xl_from_archive(game_path: String, xl_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::unlink_xl_from_archive(&game_path, &xl_name)
}

#[tauri::command]
fn start_directory_watcher(_game_path: String) -> Result<(), String> {
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
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let has_init = p.join("init.lua").exists();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(CetPluginItem {
                    name, path: p.to_string_lossy().to_string(), has_init, size_bytes, file_count, enabled: true, is_dir, created_at_ms,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let has_init = p.join("init.lua").exists();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(CetPluginItem {
                    name, path: p.to_string_lossy().to_string(), has_init, size_bytes, file_count, enabled: false, is_dir, created_at_ms,
                });
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
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(Red4extPluginItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: true, is_dir, created_at_ms,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(Red4extPluginItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: false, is_dir, created_at_ms,
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
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(RedScriptItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: true, is_dir, created_at_ms,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(RedScriptItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: false, is_dir, created_at_ms,
                });
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
fn get_r6tweaks_details(game_path: String) -> Result<Vec<R6TweaksItem>, String> {
    let root = PathBuf::from(&game_path);
    let active_dir = root.join("r6").join("tweaks");
    let disabled_dir = root.join("Disabled_Mods").join("r6tweaks");

    let mut list = Vec::new();

    if active_dir.exists() && active_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&active_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(R6TweaksItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: true, is_dir, created_at_ms,
                });
            }
        }
    }

    if disabled_dir.exists() && disabled_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&disabled_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_dir = p.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let (file_count, size_bytes) = if is_dir {
                    (count_all_files(&p), calculate_dir_size(&p))
                } else {
                    (1, entry.metadata().map(|m| m.len()).unwrap_or(0))
                };
                let created_at_ms = archive::get_path_timestamp_ms(&p);
                list.push(R6TweaksItem {
                    name, path: p.to_string_lossy().to_string(), size_bytes, file_count, enabled: false, is_dir, created_at_ms,
                });
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
fn get_recent_mods(_game_path: String, hours: u32) -> Result<Vec<LedgerEntry>, String> {
    let ledger = get_ledger_data();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let cutoff_ms = hours as u64 * 60 * 60 * 1000;
    
    let mut recent_mods: Vec<LedgerEntry> = ledger.into_iter()
        .filter(|e| now.saturating_sub(e.first_seen) <= cutoff_ms)
        .collect();
    
    recent_mods.sort_by(|a, b| b.first_seen.cmp(&a.first_seen));
    Ok(recent_mods)
}

fn get_ledger_data() -> Vec<LedgerEntry> {
    let path = get_ledger_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(ledger) = serde_json::from_str::<Vec<LedgerEntry>>(&content) {
                return ledger;
            }
        }
    }
    Vec::new()
}

fn save_ledger_data(ledger: &Vec<LedgerEntry>) -> Result<(), String> {
    let path = get_ledger_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let serialized = serde_json::to_string_pretty(ledger).map_err(|e| e.to_string())?;
    fs::write(&path, serialized).map_err(|e| e.to_string())?;
    Ok(())
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
        "r6tweaks" => (
            root.join("r6").join("tweaks"),
            root.join("Disabled_Mods").join("r6tweaks"),
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
        let _ = fs::create_dir_all(parent);
    }

    fs::rename(&source, &target).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn bulk_toggle_mod_type(game_path: String, mod_type: String, enable: bool) -> Result<(), String> {
    info!("Bulk toggling {} to {}", mod_type, enable);
    if mod_type == "archive" {
        if let Ok(report) = archive::scan_archives(&game_path) {
            for item in report.archives {
                if !item.is_delimiter && item.enabled != enable {
                    let _ = archive::toggle_mod(&game_path, &item.file_name, enable);
                }
            }
        }
    } else {
        let items = match mod_type.as_str() {
            "cet" => get_cet_details(game_path.clone()).map(|v| v.into_iter().map(|i| (i.name, i.enabled)).collect::<Vec<_>>())?,
            "red4ext" => get_red4ext_details(game_path.clone()).map(|v| v.into_iter().map(|i| (i.name, i.enabled)).collect::<Vec<_>>())?,
            "redscript" => get_redscript_details(game_path.clone()).map(|v| v.into_iter().map(|i| (i.name, i.enabled)).collect::<Vec<_>>())?,
            "r6tweaks" => get_r6tweaks_details(game_path.clone()).map(|v| v.into_iter().map(|i| (i.name, i.enabled)).collect::<Vec<_>>())?,
            _ => return Err("Unknown mod type".to_string()),
        };
        for (name, is_enabled) in items {
            if is_enabled != enable {
                let _ = toggle_plugin_state(game_path.clone(), mod_type.clone(), name, enable);
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn get_folder_contents(folder_path: String) -> Result<Vec<FileNode>, String> {
    let path = PathBuf::from(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let is_dir = p.is_dir();
            let name = entry.file_name().to_string_lossy().to_string();
            let size_bytes = if is_dir { 0 } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
            let extension = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();

            list.push(FileNode { name, path: p.to_string_lossy().to_string(), size_bytes, is_dir, extension });
        }
    }
    
    list.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(list)
}

#[tauri::command]
fn open_in_text_editor(file_path: String, editor_path: Option<String>) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() || path.is_dir() {
        return Err("Invalid file path".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = if let Some(ep) = editor_path.filter(|s| !s.trim().is_empty()) {
            std::process::Command::new(ep)
        } else {
            std::process::Command::new("notepad.exe")
        };

        cmd.arg(&file_path).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_log_folder(app: tauri::AppHandle) -> Result<(), String> {
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&log_dir).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn clear_user_log(app: tauri::AppHandle) -> Result<(), String> {
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    if log_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                    if std::fs::remove_file(&path).is_err() {
                        let _ = std::fs::File::create(&path);
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn launch_game(game_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let target = PathBuf::from(&game_path).join("bin").join("x64").join("Cyberpunk2077.exe");
        let working_dir = target.parent().unwrap_or(Path::new(""));
        std::process::Command::new(&target)
            .current_dir(working_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_custom_folder(folder_path: String) -> Result<(), String> {
    let path = PathBuf::from(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&path).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("tao", log::LevelFilter::Error)
                .level_for("wry", log::LevelFilter::Error)
                .level_for("tauri", log::LevelFilter::Warn)
                .max_file_size(50 * 1024)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            prune_logs(app);
            if let Some(window) = app.get_webview_window("main") {
                let config = load_app_config().unwrap_or_default();
                let width = config.window_width.unwrap_or(1600.0);
                let height = config.window_height.unwrap_or(1000.0);
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
                let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize { width: 1000.0, height: 650.0 })));
                if let (Some(x), Some(y)) = (config.window_x, config.window_y) {
                    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x: x as f64, y: y as f64 }));
                } else {
                    let _ = window.center();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Ok(is_maximized) = window.is_maximized() {
                    if !is_maximized {
                        if let (Ok(size), Ok(pos)) = (window.inner_size(), window.outer_position()) {
                            let scale = window.scale_factor().unwrap_or(1.0);
                            let logical_size = size.to_logical::<f64>(scale);
                            let logical_pos = pos.to_logical::<f64>(scale);
                            
                            if logical_pos.x >= -1000.0 && logical_pos.y >= -1000.0 {
                                let mut config = load_app_config().unwrap_or_default();
                                config.window_width = Some(logical_size.width);
                                config.window_height = Some(logical_size.height);
                                config.window_x = Some(logical_pos.x as i32);
                                config.window_y = Some(logical_pos.y as i32);
                                let _ = save_app_config(config);
                            }
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_app_config,
            save_app_config,
            validate_and_resolve_game_path,
            scan_game_directory,
            get_archive_details,
            toggle_mod_state,
            save_load_order,
            save_categories_config,
            load_categories_config,
            load_labels_config,
            save_labels_config,
            create_physical_folder,
            delete_physical_folder,
            rename_physical_folder,
            link_xl_to_archive,
            unlink_xl_from_archive,
            start_directory_watcher,
            get_cet_details,
            get_red4ext_details,
            get_redscript_details,
            get_r6tweaks_details,
            get_recent_mods,
            toggle_plugin_state,
            bulk_toggle_mod_type,
            get_folder_contents,
            open_in_text_editor,
            open_log_folder,
            clear_user_log,
            launch_game,
            open_custom_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}