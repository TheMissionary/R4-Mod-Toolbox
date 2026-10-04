# R4 Mod Toolbox

**Advanced Mod Manager and Conflict Resolver for Cyberpunk 2077**

R4 Mod Toolbox is a high-performance, standalone desktop application designed to give you absolute control over your Cyberpunk 2077 modding ecosystem. Built with Rust and Svelte 5, it provides real-time conflict resolution, drag-and-drop load ordering, and seamless management across all major mod frameworks.

## 🚀 Key Features

* **Real-Time Conflict Resolution:** Instantly visualize archive overrides and conflicts using a 60fps in-memory calculation engine. Know exactly which mods are winning and losing before you even launch the game.
* **Drag-and-Drop Load Ordering:** Effortlessly reorder your `.archive` mods. Group them using physical `[CAT]` folder delimiters that natively integrate with the REDengine `modlist.txt`.
* **Archive Profiles:** Save and instantly swap between different load order presets using a seamless "Radio Station" UI.
* **Universal Framework Support:** Complete visibility and toggle control over Archive, Cyber Engine Tweaks (CET), RED4ext, and Redscript/R6 mods.
* **Customizable Workspace:** Personalize your experience with dynamic accent colors, dual typography engines (Application vs. Mod Listing), and light/dark modes.

## 🛠️ Technical Stack

* **Frontend:** SvelteKit 2, Svelte 5 (Runes), Tailwind CSS v4, Vite 8
* **Backend:** Rust, Tauri v2
* **Architecture:** Zero-latency IPC bridging, atomic file operations, and strict data integrity validation.

## 🤝 Credits & Acknowledgements

Special thanks to **rfuzzo** for the [red4lib](https://github.com/rfuzzo/red4lib) Rust crate. `red4lib` powers the core archive parsing engine of this application, enabling the rapid, in-memory hash scanning required for real-time conflict detection.

## 📝 License

This project is licensed under the MIT License.
