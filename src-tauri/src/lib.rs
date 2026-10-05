mod archive;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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
    pub theme: ThemeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_game_path: "G:\\SteamLibrary\\steamapps\\common\\Cyberpunk 2077".to_string(),
            active_tab: "home".to_string(),
            show_conflict_summary: true,
            window_width: Some(1600.0),
            window_height: Some(1000.0),
            window_x: None,
            window_y: None,
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CetPluginItem {
    pub name: String,
    pub path: String,
    pub has_init: bool,
    pub size_bytes: u64,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Red4extPluginItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedScriptItem {
    pub name: String,
    pub path: String,
    pub reds_count: usize,
    pub size_bytes: u64,
    pub enabled: bool,
    #[serde(default)]
    pub is_dir: bool,
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
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn scan_game_directory(game_path: String) -> Result<ScanResult, String> {
    let root = PathBuf::from(&game_path);
    let exe_path = root.join("bin").join("x64").join("Cyberpunk2077.exe");
    let is_valid = exe_path.exists();

    let archive_dir = root.join("archive").join("pc").join("mod");
    let mut archive_active = 0;
    let mut archive_total = 0;
    if archive_dir.exists() {
        if let Ok(entries) = fs::read_dir(&archive_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("[CAT] ") { continue; }
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

    Ok(ScanResult {
        is_valid_game_path: is_valid,
        game_version: "2.31".to_string(),
        archive_active,
        archive_total,
        cet_active,
        cet_total,
        red4ext_active,
        red4ext_total,
        redscript_active,
        redscript_total,
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
fn create_physical_category(game_path: String, category_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::create_physical_category(&game_path, &category_name)
}

#[tauri::command]
fn delete_physical_category(game_path: String, category_file_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::delete_physical_category(&game_path, &category_file_name)
}

#[tauri::command]
fn rename_physical_category(game_path: String, old_file_name: String, new_category_name: String) -> Result<archive::ArchiveScanReport, String> {
    archive::rename_physical_category(&game_path, &old_file_name, &new_category_name)
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
                let size_bytes = if is_dir { calculate_dir_size(&p) } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
                list.push(CetPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    has_init,
                    size_bytes,
                    enabled: true,
                    is_dir,
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
                let size_bytes = if is_dir { calculate_dir_size(&p) } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
                list.push(CetPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    has_init,
                    size_bytes,
                    enabled: false,
                    is_dir,
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
                let size_bytes = if is_dir {
                    calculate_dir_size(&p)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                list.push(Red4extPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    size_bytes,
                    enabled: true,
                    is_dir,
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
                let size_bytes = if is_dir {
                    calculate_dir_size(&p)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                list.push(Red4extPluginItem {
                    name,
                    path: p.to_string_lossy().to_string(),
                    size_bytes,
                    enabled: false,
                    is_dir,
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
                let (reds_count, size_bytes) = if is_dir {
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
                    is_dir,
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
                let (reds_count, size_bytes) = if is_dir {
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
                    is_dir,
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

#[tauri::command]
fn open_log_folder(app: tauri::AppHandle) -> Result<(), String> {
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let config = load_app_config().unwrap_or_default();
                let width = config.window_width.unwrap_or(1600.0);
                let height = config.window_height.unwrap_or(1000.0);
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
                let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize { width: 1200.0, height: 800.0 })));
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
                if let (Ok(size), Ok(pos)) = (window.inner_size(), window.outer_position()) {
                    let scale = window.scale_factor().unwrap_or(1.0);
                    let logical_size = size.to_logical::<f64>(scale);
                    let logical_pos = pos.to_logical::<f64>(scale);
                    
                    // Defensive window guard: ignore coordinates < -1000
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
        })
        .invoke_handler(tauri::generate_handler![
            load_app_config,
            save_app_config,
            scan_game_directory,
            get_archive_details,
            toggle_mod_state,
            save_load_order,
            save_categories_config,
            load_categories_config,
            create_physical_category,
            delete_physical_category,
            rename_physical_category,
            link_xl_to_archive,
            unlink_xl_from_archive,
            start_directory_watcher,
            get_cet_details,
            get_red4ext_details,
            get_redscript_details,
            toggle_plugin_state,
            open_log_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
