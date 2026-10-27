use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XlItem {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub enabled: bool,
    pub associated_archive: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveItem {
    pub name: String,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub enabled: bool,
    pub has_conflicts: bool,
    pub conflicts_with: Vec<String>,
    pub wins: Vec<String>,
    pub loses: Vec<String>,
    pub associated_xls: Vec<XlItem>,
    pub is_delimiter: bool,
    pub category_name: Option<String>,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryItem {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub collapsed: bool,
}

pub fn get_path_timestamp_ms(path: &Path) -> Option<u64> {
    let mut newest_time = None;
    if let Ok(meta) = path.metadata() {
        let created = meta.created().ok();
        let modified = meta.modified().ok();
        newest_time = match (created, modified) {
            (Some(c), Some(m)) => Some(std::cmp::max(c, m)),
            (Some(c), None) => Some(c),
            (None, Some(m)) => Some(m),
            (None, None) => None,
        };
    }
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    let c = meta.created().ok();
                    let m = meta.modified().ok();
                    let cand = match (c, m) {
                        (Some(c), Some(m)) => Some(std::cmp::max(c, m)),
                        (Some(c), None) => Some(c),
                        (None, Some(m)) => Some(m),
                        (None, None) => None,
                    };
                    if let Some(cand_time) = cand {
                        newest_time = match newest_time {
                            Some(cur) => Some(std::cmp::max(cur, cand_time)),
                            None => Some(cand_time),
                        };
                    }
                }
            }
        }
    }
    newest_time
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveScanReport {
    pub archives: Vec<ArchiveItem>,
    pub total_conflicts: usize,
    pub affected_archives_count: usize,
    pub unassociated_xl: Vec<XlItem>,
}

fn get_categories_config_path(base_game_path: &str) -> PathBuf {
    PathBuf::from(base_game_path).join("archive").join("pc").join("mod").join("modcategories.json")
}

fn get_xl_associations_path(base_game_path: &str) -> PathBuf {
    PathBuf::from(base_game_path).join("archive").join("pc").join("mod").join("modxl_associations.json")
}

fn load_xl_associations_map(base_game_path: &str) -> HashMap<String, String> {
    let path = get_xl_associations_path(base_game_path);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn save_xl_associations_map(base_game_path: &str, map: &HashMap<String, String>) -> Result<(), String> {
    let path = get_xl_associations_path(base_game_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error!("Failed to create directory for XL associations: {}", e);
            e.to_string()
        })?;
    }
    let serialized = serde_json::to_string_pretty(map).map_err(|e| {
        error!("Failed to serialize XL associations: {}", e);
        e.to_string()
    })?;
    fs::write(&path, serialized).map_err(|e| {
        error!("Failed to write XL associations to disk: {}", e);
        e.to_string()
    })?;
    Ok(())
}

fn clean_path_name(p: &Path) -> String {
    let raw = p.file_name().unwrap_or_default().to_string_lossy().to_string();
    if let Some(stripped) = raw.strip_suffix(".disabled") {
        stripped.to_string()
    } else {
        raw
    }
}

fn clean_name_str(s: &str) -> String {
    if let Some(stripped) = s.strip_suffix(".disabled") {
        stripped.to_string()
    } else {
        s.to_string()
    }
}

fn get_archive_stem(clean_name: &str) -> String {
    if let Some(stem) = clean_name.strip_suffix(".archive") {
        stem.to_string()
    } else {
        clean_name.to_string()
    }
}

fn sync_engine_modlist(archive_dir: &Path) {
    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    let modlist_path = archive_dir.join("modlist.txt");
    
    if let Ok(content) = fs::read_to_string(&loadorder_path) {
        let mut engine_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if !trimmed.starts_with("[DIR] ") && trimmed.ends_with(".archive") {
                engine_lines.push(trimmed.to_string());
            }
        }
        if let Err(e) = fs::write(&modlist_path, engine_lines.join("\r\n") + "\r\n") {
            error!("Failed to sync engine modlist.txt: {}", e);
        } else {
            info!("Successfully synchronized engine modlist.txt");
        }
    } else {
        warn!("Could not read r4_loadorder.txt to sync engine modlist.");
    }
}

fn migrate_legacy_categories(archive_dir: &Path) {
    let modlist_path = archive_dir.join("modlist.txt");
    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    let mut needs_sync = false;

    if let Ok(entries) = fs::read_dir(archive_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                
                // 1. Migrate old .archive to .txt
                if name.starts_with("[CAT] ") || name.starts_with("[DIR] ") {
                    if name.ends_with(".archive") {
                        let new_name = name.replace(".archive", ".txt");
                        let _ = fs::rename(&p, archive_dir.join(&new_name));
                    } else if name.ends_with(".archive.disabled") {
                        let new_name = name.replace(".archive.disabled", ".txt.disabled");
                        let _ = fs::rename(&p, archive_dir.join(&new_name));
                    }
                }
                
                // 2. Migrate [CAT] to [DIR]
                let current_name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if current_name.starts_with("[CAT] ") {
                    let new_name = current_name.replacen("[CAT] ", "[DIR] ", 1);
                    info!("Migrating legacy category to folder: {} -> {}", current_name, new_name);
                    let _ = fs::rename(archive_dir.join(&current_name), archive_dir.join(&new_name));
                }
            }
        }
    }

    if loadorder_path.exists() {
        if let Ok(content) = fs::read_to_string(&loadorder_path) {
            if content.contains("[CAT] ") {
                let new_content = content.replace("[CAT] ", "[DIR] ");
                let _ = fs::write(&loadorder_path, new_content);
                needs_sync = true;
            }
        }
    } else if modlist_path.exists() {
        info!("r4_loadorder.txt not found. Migrating from existing modlist.txt...");
        if let Ok(content) = fs::read_to_string(&modlist_path) {
            let mut new_lines = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("[CAT] ") && trimmed.ends_with(".archive") {
                    new_lines.push(trimmed.replace(".archive", ".txt").replace("[CAT] ", "[DIR] "));
                } else if trimmed.starts_with("[CAT] ") {
                    new_lines.push(trimmed.replace("[CAT] ", "[DIR] "));
                } else {
                    new_lines.push(trimmed.to_string());
                }
            }
            if let Err(e) = fs::write(&loadorder_path, new_lines.join("\r\n") + "\r\n") {
                error!("Failed to write migrated r4_loadorder.txt: {}", e);
            } else {
                needs_sync = true;
            }
        }
    }

    if needs_sync {
        sync_engine_modlist(archive_dir);
    }
}

pub fn scan_archives(base_game_path: &str) -> Result<ArchiveScanReport, String> {
    info!("Scanning archives at base path: {}", base_game_path);
    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");

    if !archive_dir.exists() {
        warn!("Archive directory does not exist: {:?}", archive_dir);
        return Ok(ArchiveScanReport {
            archives: Vec::new(),
            total_conflicts: 0,
            affected_archives_count: 0,
            unassociated_xl: Vec::new(),
        });
    }

    migrate_legacy_categories(&archive_dir);

    let mut active_archive_files: Vec<PathBuf> = Vec::new();
    let mut disabled_archive_files: Vec<PathBuf> = Vec::new();
    let mut xl_files: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = fs::read_dir(&archive_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if name.ends_with(".archive") || (name.starts_with("[DIR] ") && name.ends_with(".txt")) {
                    active_archive_files.push(p);
                } else if name.ends_with(".archive.disabled") || (name.starts_with("[DIR] ") && name.ends_with(".txt.disabled")) {
                    disabled_archive_files.push(p);
                } else if name.ends_with(".xl") || name.ends_with(".xl.disabled") {
                    xl_files.push(p);
                }
            }
        }
    }

    // Deduplicate: Prioritize active files over disabled files with the same clean name
    let mut unique_archives = std::collections::HashMap::new();
    for p in active_archive_files {
        unique_archives.insert(clean_path_name(&p), p);
    }
    for p in disabled_archive_files {
        let clean = clean_path_name(&p);
        if !unique_archives.contains_key(&clean) {
            unique_archives.insert(clean, p);
        }
    }
    let mut all_archive_files: Vec<PathBuf> = unique_archives.into_values().collect();

    all_archive_files.sort_by(|a, b| {
        let clean_a = clean_path_name(a);
        let clean_b = clean_path_name(b);
        clean_a.as_bytes().cmp(clean_b.as_bytes())
    });

    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    let mut ordered_paths: Vec<PathBuf> = Vec::new();
    if loadorder_path.exists() {
        if let Ok(file) = File::open(&loadorder_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Some(pos) = all_archive_files.iter().position(|p| {
                    let clean_name = clean_path_name(p);
                    clean_name == trimmed
                }) {
                    ordered_paths.push(all_archive_files.remove(pos));
                }
            }
        }
    } else {
        warn!("r4_loadorder.txt not found during scan. Using alphabetical fallback.");
    }

    let mut final_ordered_paths = all_archive_files;
    final_ordered_paths.extend(ordered_paths);
    let ordered_paths = final_ordered_paths;

    let mut xl_items: Vec<XlItem> = Vec::new();
    for p in xl_files {
        let raw_name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        let is_enabled = !raw_name.ends_with(".disabled");
        let display_name = clean_name_str(&raw_name);
        let size_bytes = p.metadata().map(|m| m.len()).unwrap_or(0);

        xl_items.push(XlItem {
            file_name: display_name,
            path: p.to_string_lossy().to_string(),
            size_bytes,
            enabled: is_enabled,
            associated_archive: None,
        });
    }

    let explicit_associations = load_xl_associations_map(base_game_path);

    let mut raw_items = Vec::new();
    let mut hash_to_archives: HashMap<u64, Vec<String>> = HashMap::new();
    let mut archive_to_hashes: HashMap<String, Vec<u64>> = HashMap::new();

    for path in &ordered_paths {
        let raw_file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let is_enabled = !raw_file_name.ends_with(".disabled");
        let display_name = clean_name_str(&raw_file_name);

        let metadata = path.metadata().ok();
        let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);

        let is_delimiter = display_name.starts_with("[DIR] ") && display_name.ends_with(".txt");
        let category_name = if is_delimiter {
            display_name
                .strip_prefix("[DIR] ")
                .and_then(|s| s.strip_suffix(".txt"))
                .map(|s| s.trim().to_string())
        } else {
            None
        };

        let mut hashes = Vec::new();
        if is_enabled && !is_delimiter {
            if let Ok(archive) = red4lib::archive::open_read(path) {
                hashes = archive.get_entries().clone().into_keys().collect::<Vec<u64>>();
            } else {
                warn!("Failed to parse archive hashes for: {:?}", path);
            }
            for h in &hashes {
                hash_to_archives.entry(*h).or_default().push(display_name.clone());
            }
        }

        archive_to_hashes.insert(display_name.clone(), hashes.clone());

        let mut matched_xls: Vec<XlItem> = Vec::new();
        let base_stem = get_archive_stem(&display_name);

        let mut to_remove = Vec::new();
        for (i, xl) in xl_items.iter().enumerate() {
            let mut matched_target = None;
            for (k, v) in &explicit_associations {
                if k.eq_ignore_ascii_case(&xl.file_name) {
                    matched_target = Some(v.clone());
                    break;
                }
            }
            if let Some(target_arch) = matched_target {
                if target_arch.eq_ignore_ascii_case(&display_name) {
                    to_remove.push(i);
                }
            }
        }
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for i in to_remove {
            let mut item = xl_items.remove(i);
            item.associated_archive = Some(display_name.clone());
            matched_xls.push(item);
        }

        let candidate_archive_xl = format!("{}.xl", display_name);
        if let Some(pos) = xl_items.iter().position(|x| x.file_name.eq_ignore_ascii_case(&candidate_archive_xl)) {
            let actual_name = xl_items[pos].file_name.clone();
            let mut is_unlinked = false;
            for (k, v) in &explicit_associations {
                if k.eq_ignore_ascii_case(&actual_name) && v == "[UNLINKED]" {
                    is_unlinked = true;
                    break;
                }
            }
            if !is_unlinked {
                let mut item = xl_items.remove(pos);
                item.associated_archive = Some(display_name.clone());
                matched_xls.push(item);
            }
        }

        let candidate_stem_xl = format!("{}.xl", base_stem);
        if let Some(pos) = xl_items.iter().position(|x| x.file_name.eq_ignore_ascii_case(&candidate_stem_xl)) {
            let actual_name = xl_items[pos].file_name.clone();
            let mut is_unlinked = false;
            for (k, v) in &explicit_associations {
                if k.eq_ignore_ascii_case(&actual_name) && v == "[UNLINKED]" {
                    is_unlinked = true;
                    break;
                }
            }
            if !is_unlinked {
                let mut item = xl_items.remove(pos);
                item.associated_archive = Some(display_name.clone());
                matched_xls.push(item);
            }
        }

        matched_xls.truncate(3);

        let created_at_ms = if is_delimiter {
            None
        } else {
            get_path_timestamp_ms(path)
        };

        raw_items.push(ArchiveItem {
            name: display_name.clone(),
            file_name: display_name,
            path: path.to_string_lossy().to_string(),
            size_bytes,
            file_count: hashes.len(),
            enabled: is_enabled,
            has_conflicts: false,
            conflicts_with: Vec::new(),
            wins: Vec::new(),
            loses: Vec::new(),
            associated_xls: matched_xls,
            is_delimiter,
            category_name,
            created_at_ms,
        });
    }

    let mut total_contested_hashes = 0;
    for (_hash, sources) in &hash_to_archives {
        if sources.len() > 1 {
            total_contested_hashes += 1;
            let winner = &sources[0];
            let losers = &sources[1..];

            if let Some(winner_item) = raw_items.iter_mut().find(|i| &i.file_name == winner) {
                for loser in losers {
                    if !winner_item.wins.contains(loser) {
                        winner_item.wins.push(loser.clone());
                    }
                }
            }

            for loser in losers {
                if let Some(loser_item) = raw_items.iter_mut().find(|i| &i.file_name == loser) {
                    if !loser_item.loses.contains(winner) {
                        loser_item.loses.push(winner.clone());
                    }
                }
            }
        }
    }

    let mut affected_archives_count = 0;
    for item in raw_items.iter_mut() {
        let mut colliders = HashSet::new();
        for w in &item.wins { colliders.insert(w.clone()); }
        for l in &item.loses { colliders.insert(l.clone()); }

        if !colliders.is_empty() {
            item.has_conflicts = true;
            let mut list: Vec<String> = colliders.into_iter().collect();
            list.sort();
            item.conflicts_with = list;
            affected_archives_count += 1;
        }
    }

    info!("Archive scan complete. Found {} items, {} total conflicts.", raw_items.len(), total_contested_hashes);

    Ok(ArchiveScanReport {
        archives: raw_items,
        total_conflicts: total_contested_hashes,
        affected_archives_count,
        unassociated_xl: xl_items,
    })
}

pub fn toggle_mod(base_game_path: &str, mod_name: &str, enable: bool) -> Result<(), String> {
    info!("Toggling archive mod: {} (Enable: {})", mod_name, enable);
    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");
    let clean_name = clean_name_str(mod_name);

    let enabled_archive_path = archive_dir.join(&clean_name);
    let disabled_archive_path = archive_dir.join(format!("{}.disabled", clean_name));

    if enable {
        if disabled_archive_path.exists() {
            fs::rename(&disabled_archive_path, &enabled_archive_path).map_err(|e| {
                error!("Failed to enable archive {}: {}", clean_name, e);
                e.to_string()
            })?;
        }
    } else {
        if enabled_archive_path.exists() {
            fs::rename(&enabled_archive_path, &disabled_archive_path).map_err(|e| {
                error!("Failed to disable archive {}: {}", clean_name, e);
                e.to_string()
            })?;
        }
    }

    let associations = load_xl_associations_map(base_game_path);
    let base_stem = get_archive_stem(&clean_name);

    let mut candidate_xl_names: Vec<String> = Vec::new();
    
    for (xl_name, target_arch) in &associations {
        if target_arch.eq_ignore_ascii_case(&clean_name) {
            candidate_xl_names.push(xl_name.clone());
        }
    }
    
    let candidate_archive_xl = format!("{}.xl", clean_name);
    let mut is_unlinked_arch = false;
    let mut actual_arch_xl = candidate_archive_xl.clone();
    for (k, v) in &associations {
        if k.eq_ignore_ascii_case(&candidate_archive_xl) {
            actual_arch_xl = k.clone();
            if v == "[UNLINKED]" {
                is_unlinked_arch = true;
            }
        }
    }
    if !is_unlinked_arch && !candidate_xl_names.contains(&actual_arch_xl) {
        candidate_xl_names.push(actual_arch_xl);
    }
    
    let candidate_stem_xl = format!("{}.xl", base_stem);
    let mut is_unlinked_stem = false;
    let mut actual_stem_xl = candidate_stem_xl.clone();
    for (k, v) in &associations {
        if k.eq_ignore_ascii_case(&candidate_stem_xl) {
            actual_stem_xl = k.clone();
            if v == "[UNLINKED]" {
                is_unlinked_stem = true;
            }
        }
    }
    if !is_unlinked_stem && !candidate_xl_names.contains(&actual_stem_xl) {
        candidate_xl_names.push(actual_stem_xl);
    }

    for xl_name in candidate_xl_names {
        let clean_xl = clean_name_str(&xl_name);
        let enabled_xl_path = archive_dir.join(&clean_xl);
        let disabled_xl_path = archive_dir.join(format!("{}.disabled", clean_xl));

        if enable {
            if disabled_xl_path.exists() {
                let _ = fs::rename(&disabled_xl_path, &enabled_xl_path);
            }
        } else {
            if enabled_xl_path.exists() {
                let _ = fs::rename(&enabled_xl_path, &disabled_xl_path);
            }
        }
    }

    Ok(())
}

pub fn save_modlist(base_game_path: &str, ordered_archives: Vec<String>) -> Result<ArchiveScanReport, String> {
    info!("Saving load order to r4_loadorder.txt");
    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");
    fs::create_dir_all(&archive_dir).map_err(|e| {
        error!("Failed to create archive directory: {}", e);
        e.to_string()
    })?;
    
    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    let mut file = File::create(&loadorder_path).map_err(|e| {
        error!("Failed to create r4_loadorder.txt: {}", e);
        e.to_string()
    })?;

    for name in ordered_archives {
        let clean_name = clean_name_str(&name);
        if clean_name.ends_with(".archive") || (clean_name.starts_with("[DIR] ") && clean_name.ends_with(".txt")) {
            let active_path = archive_dir.join(&clean_name);
            let disabled_path = archive_dir.join(format!("{}.disabled", clean_name));
            
            if active_path.exists() || disabled_path.exists() {
                let line = format!("{}\r\n", clean_name);
                file.write_all(line.as_bytes()).map_err(|e| {
                    error!("Failed to write to r4_loadorder.txt: {}", e);
                    e.to_string()
                })?;
            }
        }
    }

    sync_engine_modlist(&archive_dir);
    scan_archives(base_game_path)
}

pub fn save_categories(base_game_path: &str, categories_json: String) -> Result<(), String> {
    let cat_path = get_categories_config_path(base_game_path);
    if let Some(parent) = cat_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&cat_path, categories_json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_categories(base_game_path: &str) -> Result<String, String> {
    let cat_path = get_categories_config_path(base_game_path);
    if cat_path.exists() {
        fs::read_to_string(&cat_path).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

pub fn create_physical_folder(base_game_path: &str, folder_name: &str) -> Result<ArchiveScanReport, String> {
    info!("Creating physical folder: {}", folder_name);
    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");
    fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;
    
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let safe_name: String = folder_name
        .chars()
        .filter(|c| !forbidden.contains(c) && !c.is_control())
        .collect();
    let safe_trimmed = safe_name.trim();

    if safe_trimmed.is_empty() {
        error!("Folder creation failed: Name cannot be empty.");
        return Err("Folder name cannot be empty".to_string());
    }

    let file_name = format!("[DIR] {}.txt", safe_trimmed);
    let file_path = archive_dir.join(&file_name);
    
    if !file_path.exists() {
        File::create(&file_path).map_err(|e| {
            error!("Failed to create folder file {}: {}", file_name, e);
            e.to_string()
        })?;
        
        let loadorder_path = archive_dir.join("r4_loadorder.txt");
        let mut existing_content = String::new();
        
        if loadorder_path.exists() {
            if let Ok(content) = fs::read_to_string(&loadorder_path) {
                existing_content = content;
            }
        }
        
        let mut new_content = existing_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push_str("\r\n");
        }
        new_content.push_str(&format!("{}\r\n", file_name));
        
        fs::write(&loadorder_path, new_content).map_err(|e| {
            error!("Failed to append folder to r4_loadorder.txt: {}", e);
            e.to_string()
        })?;
    } else {
        warn!("Folder file already exists: {}", file_name);
    }
    
    sync_engine_modlist(&archive_dir);
    scan_archives(base_game_path)
}

pub fn delete_physical_folder(base_game_path: &str, folder_file_name: &str) -> Result<ArchiveScanReport, String> {
    info!("Deleting physical folder: {}", folder_file_name);
    let clean_name = clean_name_str(folder_file_name);
    if !clean_name.starts_with("[DIR] ") || !clean_name.ends_with(".txt") {
        error!("Safety check failed: Target {} is not a physical folder delimiter", folder_file_name);
        return Err("Safety check failed: Target is not a physical folder delimiter".to_string());
    }

    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");
    let active_path = archive_dir.join(&clean_name);
    let disabled_path = archive_dir.join(format!("{}.disabled", clean_name));

    if active_path.exists() {
        fs::remove_file(&active_path).map_err(|e| {
            error!("Failed to delete folder file {}: {}", clean_name, e);
            e.to_string()
        })?;
    }
    if disabled_path.exists() {
        fs::remove_file(&disabled_path).map_err(|e| {
            error!("Failed to delete disabled folder file {}: {}", clean_name, e);
            e.to_string()
        })?;
    }

    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    if loadorder_path.exists() {
        if let Ok(content) = fs::read_to_string(&loadorder_path) {
            let mut updated_lines = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if trimmed != clean_name && trimmed != folder_file_name {
                    updated_lines.push(trimmed.to_string());
                }
            }
            let mut new_loadorder = updated_lines.join("\r\n");
            if !new_loadorder.is_empty() {
                new_loadorder.push_str("\r\n");
            }
            fs::write(&loadorder_path, new_loadorder).map_err(|e| {
                error!("Failed to update r4_loadorder.txt after folder deletion: {}", e);
                e.to_string()
            })?;
        }
    }

    sync_engine_modlist(&archive_dir);
    scan_archives(base_game_path)
}

pub fn rename_physical_folder(base_game_path: &str, old_file_name: &str, new_folder_name: &str) -> Result<ArchiveScanReport, String> {
    info!("Renaming physical folder from {} to {}", old_file_name, new_folder_name);
    let old_clean_name = clean_name_str(old_file_name);
    if !old_clean_name.starts_with("[DIR] ") || !old_clean_name.ends_with(".txt") {
        error!("Safety check failed: Target {} is not a physical folder delimiter", old_file_name);
        return Err("Safety check failed: Target is not a physical folder delimiter".to_string());
    }

    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let safe_name: String = new_folder_name
        .chars()
        .filter(|c| !forbidden.contains(c) && !c.is_control())
        .collect();
    let safe_trimmed = safe_name.trim();

    if safe_trimmed.is_empty() {
        error!("Folder rename failed: New name cannot be empty.");
        return Err("Folder name cannot be empty".to_string());
    }

    let new_clean_name = format!("[DIR] {}.txt", safe_trimmed);
    let archive_dir = PathBuf::from(base_game_path).join("archive").join("pc").join("mod");

    let old_active_path = archive_dir.join(&old_clean_name);
    let old_disabled_path = archive_dir.join(format!("{}.disabled", old_clean_name));
    let new_active_path = archive_dir.join(&new_clean_name);
    let new_disabled_path = archive_dir.join(format!("{}.disabled", new_clean_name));

    if old_active_path.exists() {
        fs::rename(&old_active_path, &new_active_path).map_err(|e| {
            error!("Failed to rename folder file: {}", e);
            e.to_string()
        })?;
    } else if old_disabled_path.exists() {
        fs::rename(&old_disabled_path, &new_disabled_path).map_err(|e| {
            error!("Failed to rename disabled folder file: {}", e);
            e.to_string()
        })?;
    }

    let loadorder_path = archive_dir.join("r4_loadorder.txt");
    if loadorder_path.exists() {
        if let Ok(content) = fs::read_to_string(&loadorder_path) {
            let mut updated_lines = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if trimmed == old_clean_name || trimmed == old_file_name {
                    updated_lines.push(new_clean_name.clone());
                } else {
                    updated_lines.push(trimmed.to_string());
                }
            }
            let mut new_loadorder = updated_lines.join("\r\n");
            if !new_loadorder.is_empty() {
                new_loadorder.push_str("\r\n");
            }
            fs::write(&loadorder_path, new_loadorder).map_err(|e| {
                error!("Failed to update r4_loadorder.txt after folder rename: {}", e);
                e.to_string()
            })?;
        }
    }

    sync_engine_modlist(&archive_dir);
    scan_archives(base_game_path)
}

pub fn link_xl_to_archive(base_game_path: &str, xl_name: &str, archive_name: &str) -> Result<ArchiveScanReport, String> {
    info!("Linking XL file {} to archive {}", xl_name, archive_name);
    let mut map = load_xl_associations_map(base_game_path);
    map.insert(xl_name.to_string(), archive_name.to_string());
    save_xl_associations_map(base_game_path, &map)?;
    scan_archives(base_game_path)
}

pub fn unlink_xl_from_archive(base_game_path: &str, xl_name: &str) -> Result<ArchiveScanReport, String> {
    info!("Unlinking XL file {}", xl_name);
    let mut map = load_xl_associations_map(base_game_path);
    map.insert(xl_name.to_string(), "[UNLINKED]".to_string());
    save_xl_associations_map(base_game_path, &map)?;
    scan_archives(base_game_path)
}
