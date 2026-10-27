# R4 Mod Toolbox

**Advanced Mod Manager and Conflict Resolver for Cyberpunk 2077**

![Beta Release](https://img.shields.io/badge/Release-v0.39.0_(Beta_4)-orange.svg)
![Platform](https://img.shields.io/badge/Platform-Windows_10_%7C_11-blue.svg)
![R4 Mod Toolbox Header](https://raw.githubusercontent.com/TheMissionary/R4-Mod-Toolbox/main/assets/headertemplate2.png)

R4 Mod Toolbox is a high-performance, standalone desktop application built specifically for **manual modders**. If you prefer to download and extract your own mods but want a powerful, visualized dashboard to manage load orders, resolve conflicts, and organize your workspace, R4 is built for you. 

It does not hijack your downloads or obfuscate your files in virtual file systems. It simply gives you absolute, deterministic control over your existing Cyberpunk 2077 modding ecosystem.

---

## ⚠️ Beta Release v0.39.0
R4 Mod Toolbox has entered its public beta release period. The core engine is stable, but we are actively pursuing community testing, feedback, and edge-case reports from users with massive, complex modlists. Please report any bugs or UX friction in the [Issues tab](https://github.com/TheMissionary/R4-Mod-Toolbox/issues). 

---


## 🚀 Key Features

* **Real-Time Conflict Resolution:** R4 scans the internal hashes of every `.archive` file in milliseconds. Instantly see exactly which mods are winning (green) and losing (red) asset conflicts before you even launch the game.
* **Drag-and-Drop Load Ordering:** Effortlessly reorder your mods to change conflict winners. The UI updates conflict states in real-time as you drag.
* **Universal Framework Support:** Complete visibility with Windows Explorer-style file trees and safe toggle control over **Archive**, **CET**, **RED4ext**, **R6\Scripts**, and **R6\Tweaks**.
* **Mod History & Ledger:** Track every mod by its "First Seen" date. Use the universal search to instantly locate any file across your entire modding history.
* **Smart XL Linking:** ArchiveXL requires `.xl` files to load custom entities. R4 automatically pairs these sidecar files with their parent `.archive`, allowing you to toggle both with a single click.
* **Advanced Integrations:** Link your custom text editor, custom game launch path, and Cyberpunk 2077 Photo Folder for seamless access directly from the sidebar.
* **Customizable Workspace:** Personalize your experience with dynamic accent colors, dual typography engines (featuring Geist and GeistMono), and light/dark modes.

---

![Archive Conflict Summary](https://raw.githubusercontent.com/TheMissionary/R4-Mod-Toolbox/main/assets/R4-ArchiveConflictSummary.png)

![CET Explorer](https://raw.githubusercontent.com/TheMissionary/R4-Mod-Toolbox/main/assets/R4-CET-Explorer.png)

## 📂 Transparency: How R4 Manages Your Files

Because R4 is built for manual modders, we believe in 100% transparency regarding what the app does to your files. 

### 1. Disabling Mods Safely
R4 uses two highly optimized methods to disable mods without deleting your data:
* **Archive Mods:** Toggling an archive simply renames it in-place (appending `.disabled`). It never leaves your mod folder and retains its exact alphabetical position.
* **Frameworks & Scripts:** Toggling CET, RED4ext, or R6 mods executes an in-place swap to move the disabled files into your `Cyberpunk 2077\Disabled_Mods` folder. This prevents the REDengine from throwing fatal compilation errors on boot.


### 2. Physical Categories & Load Order Management
To help you organize massive modlists, R4 allows you to create custom Categories. R4 manages these categories safely alongside the game's native load order system.

**What you will see in your `archive\pc\mod` folder:**
* `[CAT] Your Category Name.txt` (These are 0-byte physical markers).
* `r4_loadorder.txt` (R4's internal master tracker).
* `modlist.txt` (The standard engine load order file).

**How it works:**
1. When you create a category, R4 generates a 0-byte `.txt` file to act as a physical visual separator in your folder.
2. R4 reads and writes your exact visual layout (including categories) to `r4_loadorder.txt`.
3. Every time you make a change, R4 dynamically generates a clean `modlist.txt` containing only your active `.archive` files, ensuring the game engine receives exactly what it needs to run safely.
4. **Toggling Categories:** If you disable a category in the app, R4 renames the marker to `.txt.disabled` and automatically disables every mod nested beneath it.

---

## 🛠️ Technical Stack

* **Frontend:** SvelteKit 2, Svelte 5 (Runes), Tailwind CSS v4, Vite 8
* **Backend:** Rust, Tauri v2
* **Architecture:** Zero-latency IPC bridging, atomic file operations, and strict data integrity validation.

---

## 🤝 Credits & Acknowledgements

A massive thank you to **rfuzzo** for their foundational work on Wolvenkit and the creation of the [red4lib](https://github.com/rfuzzo/red4lib) Rust crate. `red4lib` powers the core archive parsing engine of this application. Without their continuous innovation and dedication to the Cyberpunk 2077 modding ecosystem, the real-time archive conflict resolution engine powering R4 would not be possible.

## 📝 License

This project is licensed under the MIT License.
