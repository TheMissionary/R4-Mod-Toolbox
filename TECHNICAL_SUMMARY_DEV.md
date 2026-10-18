# Technical Summary & Development Handoff to Google AI Studio (GAIS)

**Project:** R4 Mod Toolbox (Cyberpunk 2077 Desktop Mod Manager)  
**Target Environment:** Tauri v2 (Rust backend) + Svelte 5 / SvelteKit (Tailwind CSS v4 frontend)  
**Primary Directory:** `G:\R4-Mod-TBX`  
**Active Development Branch:** `dev` (Commit `421704d`)  
**Production Release Baseline:** `main` (Tag `v0.37.1-beta.3`, Commit `fa93a49`)  
**Date:** 2026-10-17  

---

## 1. Context & Purpose of this Document

This document provides a comprehensive technical briefing for sessions in **Google AI Studio (GAIS)** returning to the R4 Mod Toolbox codebase. It bridges the gap between the initial conversation state and the current development head, outlining all architectural changes, resolved defects, new feature implementations, file system modifications, and build artifacts.

---

## 2. Git & Remote Security Configuration

To ensure absolute safety regarding release integrity and data protection, the repository uses a strict multi-remote workflow:

### Remotes
- **`origin` (Public Repository):** `https://github.com/TheMissionary/R4-Mod-Toolbox.git`
  - **Push URL:** `DISABLED_NO_PUSH_WITHOUT_AUTHORIZATION`
  - *Safety Rule:* Pushes to `origin` are strictly prohibited during normal development. They are only unlocked during formal, user-approved release tagging.
- **`backup` (Private Development Remote):** `https://github.com/TheMissionary/R4-Mod-Toolbox-dev.git`
  - **Push URL:** Active (`git push backup dev`).
  - *Safety Rule:* All incremental commits on `dev` are automatically pushed to this private mirror.

### Privacy Configuration
- **`.agent/` Directory:** Explicitly ignored in `.gitignore` to prevent agent configuration, scratch files, and internal session logs from ever entering the git tree.

---

## 3. Phase 1: Critical First-Run Hotfix (`v0.37.1-beta.3`)

Before implementing user-facing features on `dev`, a critical hotfix release was required to address defects that would have immediately compromised external beta testers on clean installations.

### Defects Resolved
1. **Hardcoded Developer Paths:**
   - Previously, `+page.svelte` and `DEFAULT_APP_CONFIG` initialized `gamePath` to `G:\SteamLibrary\steamapps\common\Cyberpunk 2077`. On any other machine, this triggered an initial failed scan and visual error state.
   - **Fix:** Initialized `gamePath` as an empty string (`''`) and introduced dynamic first-run guidance.
2. **Folder Picker & Parent Path Auto-Resolution:**
   - In previous builds, clicking "Verify Game Path" opened Windows File Explorer at `bin\x64` by default without validation, leading users to select `bin\x64` instead of the root directory.
   - **Fix:** Implemented `validate_and_resolve_game_path(candidate_path: String)` in `src-tauri/src/lib.rs`. If a user selects `bin` or `bin\x64`, the backend automatically resolves upward to the game root, verifies `bin\x64\Cyberpunk2077.exe`, and persists the validated path to `%APPDATA%\r4-mod-toolbox\config.json`.
3. **Hero Banner Reactive States:**
   - Replaced a static "ENGINE CONNECTED" banner with reactive states:
     - `NO GAME FOLDER CONFIGURED` (Amber prompt on initial startup).
     - `GAME EXECUTABLE NOT FOUND` (Red warning if path is invalid).
     - `ENGINE CONNECTED` (Green confirmation upon verification).
4. **Type and Component Casing:**
   - Resolved casing mismatches in `RedScriptView` imports and fixed NavRail `ActiveView` type collisions.

### Release Execution
- Version bumped to `0.37.1` across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Merged into `main`, tagged `v0.37.1-beta.3`, pushed to public `origin`, and the push URL was re-locked.

---

## 4. Phase 2: UI/UX Enhancements Implemented on `dev`

All feature work was executed on `dev` following the user-approved UI/UX modernization roadmap.

### Feature 1: HeaderBar Path Chip & Popover (`Commit 14a435c`)
- **Objective:** Eliminate the wide, persistent `TARGET PATH` text box that consumed header space.
- **Implementation (`src/lib/components/HeaderBar.svelte`):**
  - Introduced a compact status chip: `● Cyberpunk 2077 [Connected] ▾` (Emerald) or `[Select Folder] ▾` (Amber).
  - Clicking the chip opens a context popover containing:
    - Current directory path with one-click clipboard copy.
    - Direct **Show in Explorer** action (`@tauri-apps/plugin-opener`).
    - **Change Game Folder** action.
  - Reorganized header right actions into clean `[ Change Folder ]` and `[ Sync & Rescan ]` buttons.

### Feature 2: Dynamic Launch Button & Process Control (`Commit c65e3e0`)
- **Objective:** Provide real-time game execution feedback and a non-invasive force-stop kill mechanism.
- **Backend (`src-tauri/src/lib.rs`):**
  - Added `check_game_process_running()`: Executes Windows `tasklist /FI "IMAGENAME eq Cyberpunk2077.exe" /NH` using Windows API flag `CREATE_NO_WINDOW (0x08000000)` to eliminate CMD window flashes.
  - Added `kill_game_process()`: Executes `taskkill /F /IM Cyberpunk2077.exe /T` with `CREATE_NO_WINDOW`.
- **Frontend (`src/routes/+page.svelte`):**
  - Background polling loop monitors process state at 2.5-second intervals.
  - Sidebar button dynamically transitions between three states:
    1. **Idle:** `[ ▶ Launch Game ]` (Disabled if path invalid).
    2. **Launching:** `[ ⟳ Loading... ]` spinner state.
    3. **Running (75/25 Split Button):**
       - **Left 75%:** `● Now Playing` with pulsing emerald dot.
       - **Right 25%:** `✕` Force Stop button.
       - **2-Step Safety Confirmation:** Clicking `✕` transitions it to a flashing red `Kill?` state with a 3.5-second auto-reset timer. Clicking a second time immediately executes `kill_game_process()`.

### Feature 3: Configurable `[NEW]` Mod Badge System (`Commit 421704d`)
- **Objective:** Identify recently installed or updated mods across all 5 mod pages, adhering to vertical column alignment and user-configured timeframes.
- **Backend Timestamp Scanner (`src-tauri/src/archive.rs` & `src-tauri/src/lib.rs`):**
  - Added `get_path_timestamp_ms(&Path) -> Option<u64>` comparing NTFS `created` and `modified` times (`max(created, modified)`).
  - For folder-based mods (`CET`, `RED4ext`, etc.), it checks both the folder and immediate child files to guarantee accurate detection when archive extractors preserve author folder dates.
  - Added `created_at_ms: Option<u64>` to `ArchiveItem`, `CetPluginItem`, `Red4extPluginItem`, `RedScriptItem`, and `R6TweaksItem`.
- **Settings & Persistence (`src/lib/theme.ts` & `src-tauri/src/lib.rs`):**
  - Extended `AppConfig` with `new_mods_hours: Option<u32>` (default `6`) and `show_new_mods: Option<bool>` (default `false`).
  - Automatically saved to `%APPDATA%\r4-mod-toolbox\config.json`.
  - State is synchronized across all mod tabs—changing the threshold on one view updates all others.
- **Unified Toolbar Control (`src/lib/components/NewModsControl.svelte`):**
  - Segmented pill containing a `[ Sparkles New (Count) ]` toggle button and numerical hours input (`min: 1`, `max: 8760`).
  - Active button styles dynamically with the user's accent color (`var(--theme-accent)`).
- **Dynamic Contrast & Readability (`src/lib/theme.ts` & `src/app.css`):**
  - Implemented `getContrastColor(hex)` calculating perceived luminance, setting `--theme-accent-contrast` (`#000000` or `#ffffff`).
  - Guarantees high readability on `NEW` badges and active buttons in both light and dark modes across all theme presets (NVIDIA Green, Cyberpunk Yellow, Netrunner Cyan, Militech Blue, etc.).
- **Laser-Aligned Column Placement:**
  - **Archive Mods:** Placed `[NEW]` immediately next to `#{archiveRank}` inside a fixed-width container (`min-w-[34px] text-center`).
  - **Framework Mods (CET / RED4ext / Redscript / R6Tweaks):** Placed `[NEW]` immediately to the right of the folder/file icon container (`w-[18px]` spacer/chevron + icon).
  - Every badge down the list shares the exact same horizontal X-coordinate, creating a consistent vertical column.

---

## 5. Phase 3: Standalone Build Pipeline & Artifacts

### Packaging Resolution
When testing release binaries compiled with plain `cargo build --release`, Tauri v2 defaults to `devUrl` (`http://localhost:1420`), causing `localhost refused to connect (ERR_CONNECTION_REFUSED)` in standalone execution.

Running the complete packaging pipeline:
```powershell
& npm.cmd run tauri build
```
Executes Vite frontend compilation (`vite build && node patch-paths.js`) and embeds all static assets directly into the binary via `tauri-build`.

### Generated Executables
- `G:\R4-Mod-TBX\r4-mod-toolbox-dev.exe` (10,957,312 bytes)
- `G:\R4-Mod-TBX\r4-mod-toolbox.exe` (10,957,312 bytes)
- `G:\R4-Mod-TBX\src-tauri\target\release\r4-mod-toolbox.exe` (10,957,312 bytes)

*(Baseline backup of prior version preserved at `G:\R4-Backups-Release-Baseline\r4-mod-toolbox-v0.37.0-beta.2.exe`).*

---

## 6. Comprehensive File Inventory

```
G:\R4-Mod-TBX\
├── TECHNICAL_SUMMARY_DEV.md         # This technical documentation
├── r4-mod-toolbox-dev.exe           # Standalone development test binary (10.9 MB)
├── r4-mod-toolbox.exe               # Standalone release binary (10.9 MB)
├── package.json                     # v0.37.1 dependencies & build scripts
├── .gitignore                       # Configured for .agent/, *.exe, build/
├── src-tauri/
│   ├── Cargo.toml                   # v0.37.1
│   ├── tauri.conf.json              # Window, bundle, and frontendDist configurations
│   ├── src/
│   │   ├── lib.rs                   # Process monitoring, AppConfig, plugin scanners
│   │   └── archive.rs               # Archive scanner, get_path_timestamp_ms helper
│   └── target/release/
│       └── r4-mod-toolbox.exe       # Tauri release artifact
└── src/
    ├── app.css                      # Semantic tokens (--theme-accent-contrast)
    ├── routes/
    │   └── +page.svelte             # Process polling, newMods state, view bindings
    └── lib/
        ├── types.ts                 # Extended mod items with created_at_ms & AppConfig
        ├── theme.ts                 # isModNew, getContrastColor, config hydration
        └── components/
            ├── HeaderBar.svelte     # Status chip & dropdown popover
            ├── NewModsControl.svelte# Reusable segmented [New] pill control
            ├── ArchiveView.svelte   # [NEW] badge next to #{archiveRank}
            ├── CetView.svelte       # [NEW] badge next to folder/file icon
            ├── Red4extView.svelte   # [NEW] badge next to folder/file icon
            ├── RedScriptView.svelte # [NEW] badge next to folder/file icon
            └── R6TweaksView.svelte  # [NEW] badge next to folder/file icon
```

---

## 7. Open Considerations & Next Steps for GAIS

1. **User Experience Verification:**
   - Confirm ergonomic feel of the 2-step force-stop kill timer (currently 3.5 seconds).
   - Verify layout and font readability of `[NEW]` badge across custom accent colors.
2. **Review Callout A5 (Sync & Rescan In-Memory Refresh):**
   - In `HeaderBar.svelte`, `handleHardRefresh` currently calls `window.location.reload()`, which resets scroll position and folder expansion state. Transitioning this to call the reactive `onRefresh` prop will provide non-destructive rescans.
3. **Category Delimiter Boundaries:**
   - Maintain REDengine compatibility: category delimiter files must remain 0-byte `.txt` markers (`[CAT] *.txt`), never `.archive` files, as 0-byte `.archive` files crash REDengine at launch.
4. **Subsequent Release Graduation:**
   - When ready to release features from `dev` to public users, merge `dev` into `main`, bump version (e.g., `0.38.0`), unlock `origin` push permissions, and publish the tagged release.
