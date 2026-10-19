<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open, message } from '@tauri-apps/plugin-dialog';
  import {
    APP_FONT_PRESETS,
    MOD_FONT_PRESETS,
    generateThemeColors,
    loadThemeSettings,
    applyThemeSettings,
    loadConfigFromDisk,
    saveConfigToDisk,
    type ThemeSettings
  } from '$lib/theme';
  import {
    RotateCcw,
    Check,
    Settings,
    Sun,
    Moon,
    GripVertical,
    Type,
    Sparkles,
    FolderOpen,
    FileCode,
    Palette,
    Play,
    Trash2,
    FolderSearch,
    Folder,
    ChevronDown,
    ChevronRight,
    MoreVertical,
    Crown
  } from 'lucide-svelte';

  let { 
    gamePath = $bindable(''),
    onRefresh 
  }: { 
    gamePath?: string;
    onRefresh?: () => void;
  } = $props();

  let draft = $state<ThemeSettings>(loadThemeSettings());
  let currentAccentHex = $state('#76B900');
  let selectedBasePreset = $state<string>(APP_FONT_PRESETS[0].value);
  let customBaseInput = $state('');
  let selectedModsPreset = $state<string>(MOD_FONT_PRESETS[0].value);
  let customModsInput = $state('');
  let customEditorPath = $state('');
  let customLaunchPath = $state('');
  let isDirty = $state(false);
  let clearedFeedback = $state(false);

  let activeColors = $derived(draft.mode === 'light' ? draft.lightColors : draft.darkColors);

  function markDirty() {
    isDirty = true;
  }

  onMount(() => {
    draft = loadThemeSettings();
    const activePal = draft.mode === 'light' ? draft.lightColors : draft.darkColors;
    currentAccentHex = activePal.accent.toUpperCase();

    const currentBase = draft.fontFamilyBase || APP_FONT_PRESETS[0].value;
    const matchedBase = APP_FONT_PRESETS.find(p => p.value === currentBase);
    if (matchedBase) {
      selectedBasePreset = matchedBase.value;
      customBaseInput = '';
    } else {
      selectedBasePreset = 'custom';
      customBaseInput = currentBase.replace(/^"|"$/g, '').replace(/, sans-serif$/, '');
    }

    const currentMods = draft.fontFamilyMods || MOD_FONT_PRESETS[0].value;
    const matchedMods = MOD_FONT_PRESETS.find(p => p.value === currentMods);
    if (matchedMods) {
      selectedModsPreset = matchedMods.value;
      customModsInput = '';
    } else {
      selectedModsPreset = 'custom';
      customModsInput = currentMods.replace(/^"|"$/g, '').replace(/, monospace$/, '');
    }

    loadConfigFromDisk().then(config => {
      customEditorPath = config.customTextEditorPath || '';
      customLaunchPath = config.customLaunchPath || '';
    });
  });

  async function chooseDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Cyberpunk 2077 Root Folder',
        defaultPath: gamePath || undefined
      });
      
      if (typeof selected === 'string') {
        const resolved = await invoke<string | null>('validate_and_resolve_game_path', { candidatePath: selected });
        if (resolved) {
          gamePath = resolved;
          try {
            const config = await loadConfigFromDisk();
            config.targetGamePath = resolved;
            await saveConfigToDisk(config);
          } catch (e) {
            console.error('Failed to save updated gamePath:', e);
          }
          if (onRefresh) onRefresh();
        } else {
          await message(
            'The selected directory does not appear to be a valid Cyberpunk 2077 root folder (bin\\x64\\Cyberpunk2077.exe was not found).\n\nPlease select the main Cyberpunk 2077 game directory.',
            { title: 'Invalid Game Directory', kind: 'error' }
          );
        }
      }
    } catch (err) {
      console.error('Directory picker failed:', err);
    }
  }

  function setAccentColor(hex: string) {
    currentAccentHex = hex.toUpperCase();
    if (draft.mode === 'light') {
      draft.lightColors = generateThemeColors('light', hex);
    } else {
      draft.darkColors = generateThemeColors('dark', hex);
    }
    markDirty();
  }

  function handleModeToggle(newMode: 'dark' | 'light') {
    draft.mode = newMode;
    const activePal = newMode === 'light' ? draft.lightColors : draft.darkColors;
    currentAccentHex = activePal.accent.toUpperCase();
    markDirty();
  }

  function handleColorWheelChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    setAccentColor(val);
  }

  function handleManualHexInput(e: Event) {
    let val = (e.target as HTMLInputElement).value.trim();
    if (!val.startsWith('#')) val = '#' + val;
    currentAccentHex = val.toUpperCase();

    if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
      setAccentColor(val);
    }
  }

  function handleBasePresetChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedBasePreset = val;
    if (val !== 'custom') {
      draft.fontFamilyBase = val;
    } else {
      draft.fontFamilyBase = customBaseInput ? `"${customBaseInput}", sans-serif` : APP_FONT_PRESETS[0].value;
    }
    markDirty();
  }

  function handleCustomBaseInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.trim();
    customBaseInput = raw;
    draft.fontFamilyBase = raw ? `"${raw}", sans-serif` : APP_FONT_PRESETS[0].value;
    markDirty();
  }

  function handleModsPresetChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedModsPreset = val;
    if (val !== 'custom') {
      draft.fontFamilyMods = val;
    } else {
      draft.fontFamilyMods = customModsInput ? `"${customModsInput}", monospace` : MOD_FONT_PRESETS[0].value;
    }
    markDirty();
  }

  function handleCustomModsInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.trim();
    customModsInput = raw;
    draft.fontFamilyMods = raw ? `"${raw}", monospace` : MOD_FONT_PRESETS[0].value;
    markDirty();
  }

  async function handleBrowseEditor() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'Executables', extensions: ['exe'] }],
        title: 'Select Custom Text Editor'
      });
      if (typeof selected === 'string') {
        customEditorPath = selected;
        markDirty();
      }
    } catch (err) {
      console.error('Failed to select editor:', err);
    }
  }

  async function handleBrowseLaunchPath() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'Executables & Shortcuts', extensions: ['exe', 'lnk'] }],
        title: 'Select Custom Launch Target'
      });
      if (typeof selected === 'string') {
        customLaunchPath = selected;
        markDirty();
      }
    } catch (err) {
      console.error('Failed to select launch path:', err);
    }
  }

  function handleResetDefault() {
    if (draft.mode === 'light') {
      draft.lightColors = generateThemeColors('light', '#5A8F00');
      currentAccentHex = '#5A8F00';
    } else {
      draft.darkColors = generateThemeColors('dark', '#76B900');
      currentAccentHex = '#76B900';
    }
    draft.fontFamilyBase = APP_FONT_PRESETS[0].value;
    draft.fontFamilyMods = MOD_FONT_PRESETS[0].value;
    selectedBasePreset = APP_FONT_PRESETS[0].value;
    selectedModsPreset = MOD_FONT_PRESETS[0].value;
    customBaseInput = '';
    customModsInput = '';
    customEditorPath = '';
    customLaunchPath = '';
    markDirty();
  }

  async function handleApply() {
    if (selectedBasePreset === 'custom' && customBaseInput.trim()) {
      draft.fontFamilyBase = `"${customBaseInput.trim()}", sans-serif`;
    }
    if (selectedModsPreset === 'custom' && customModsInput.trim()) {
      draft.fontFamilyMods = `"${customModsInput.trim()}", monospace`;
    }
    draft.fontFamily = draft.fontFamilyBase;

    const plainPayload: ThemeSettings = JSON.parse(JSON.stringify(draft));
    
    applyThemeSettings(plainPayload);
    
    try {
      let config = await loadConfigFromDisk();
      config.theme = plainPayload;
      config.customTextEditorPath = customEditorPath.trim();
      config.customLaunchPath = customLaunchPath.trim();
      await saveConfigToDisk(config);
      isDirty = false;
    } catch (err) {
      console.error('Failed to save config:', err);
    }
  }

  async function handleOpenLogs() {
    try {
      await invoke('open_log_folder');
    } catch (err) {
      console.error('Failed to open log folder:', err);
    }
  }

  async function handleClearLogs() {
    try {
      await invoke('clear_user_log');
      clearedFeedback = true;
      setTimeout(() => { clearedFeedback = false; }, 2000);
    } catch (err) {
      console.error('Failed to clear logs:', err);
    }
  }
</script>

<div class="flex flex-col h-full overflow-hidden bg-nvidia-bg text-nvidia-text-primary">
  <div class="p-6 pb-4 shrink-0">
    <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
      <Settings class="h-4 w-4" />
      <span>APPLICATION SETTINGS</span>
    </div>
    <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">Configuration & Theming</h2>
    <p class="text-xs text-nvidia-text-muted max-w-xl">
      Customize the visual appearance, typography, and external tool integrations for the R4 Mod Toolbox.
    </p>
  </div>

  <div class="flex-1 overflow-y-auto px-6 pb-6">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-6xl">
      
      <!-- Left Column: Directories & Integrations -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 flex flex-col h-fit">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <FolderSearch class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Directories & Integrations</h3>
        </div>
        
        <div class="p-5 space-y-6">
          <!-- Game Directory -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-nvidia-text-primary">Game Directory <span class="text-red-400">*</span></span>
            </div>
            <p class="text-[11px] text-nvidia-text-muted leading-tight">
              The root installation folder for Cyberpunk 2077. Required for the toolbox to function.
            </p>
            <div class="flex items-center gap-2">
              <div class="flex-1 px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary font-mono truncate select-all">
                {gamePath || 'No folder selected'}
              </div>
              <button
                type="button"
                onclick={chooseDirectory}
                class="px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs font-semibold text-nvidia-text-primary transition cursor-pointer shrink-0 shadow-xs"
              >
                Change...
              </button>
            </div>
          </div>

          <div class="h-px bg-nvidia-border/50"></div>

          <!-- Custom Launch Path -->
          <div class="space-y-2">
            <span class="text-xs font-semibold text-nvidia-text-primary">Custom Launch Path</span>
            <p class="text-[11px] text-nvidia-text-muted leading-tight">
              Select a custom executable or shortcut (.lnk) to launch instead of the default Cyberpunk2077.exe.
            </p>
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={customLaunchPath}
                oninput={markDirty}
                placeholder="Leave blank for default"
                class="flex-1 px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/50 focus:outline-none focus:border-nvidia-accent font-mono"
              />
              <button
                type="button"
                onclick={handleBrowseLaunchPath}
                class="px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0"
              >
                Browse...
              </button>
            </div>
          </div>

          <div class="h-px bg-nvidia-border/50"></div>

          <!-- Custom Text Editor -->
          <div class="space-y-2">
            <span class="text-xs font-semibold text-nvidia-text-primary">Custom Text Editor</span>
            <p class="text-[11px] text-nvidia-text-muted leading-tight">
              Select an executable (e.g., VS Code, Notepad++) to open configuration files.
            </p>
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={customEditorPath}
                oninput={markDirty}
                placeholder="Leave blank for Notepad"
                class="flex-1 px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/50 focus:outline-none focus:border-nvidia-accent font-mono"
              />
              <button
                type="button"
                onclick={handleBrowseEditor}
                class="px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0"
              >
                Browse...
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column: Appearance & Preview -->
      <div class="flex flex-col gap-6">
        
        <!-- Appearance & Typography -->
        <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 flex flex-col h-fit">
          <div class="px-4 py-2.5 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <Palette class="h-4 w-4 text-nvidia-accent" />
              <h3 class="text-sm font-bold uppercase tracking-wider">Appearance & Typography</h3>
            </div>
            
            <!-- Integrated Mode Switch -->
            <div class="flex items-center p-0.5 rounded border border-nvidia-border bg-nvidia-bg">
              <button
                type="button"
                onclick={() => handleModeToggle('dark')}
                class="flex items-center gap-1.5 px-2 py-1 rounded-sm text-[10px] font-bold uppercase tracking-wider transition cursor-pointer {draft.mode === 'dark' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
              >
                <Moon class="h-3 w-3 text-nvidia-accent" />
                <span>Dark</span>
              </button>
              <button
                type="button"
                onclick={() => handleModeToggle('light')}
                class="flex items-center gap-1.5 px-2 py-1 rounded-sm text-[10px] font-bold uppercase tracking-wider transition cursor-pointer {draft.mode === 'light' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
              >
                <Sun class="h-3 w-3 text-amber-500" />
                <span>Light</span>
              </button>
            </div>
          </div>

          <div class="p-5 space-y-5">
            <!-- Accent Color -->
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-xs font-semibold text-nvidia-text-primary">
                <Sparkles class="h-3.5 w-3.5 text-nvidia-accent" />
                <span>Accent Color</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="relative h-8 w-12 rounded border border-nvidia-border overflow-hidden shrink-0 shadow-sm hover:border-nvidia-accent transition cursor-pointer">
                  <input
                    type="color"
                    value={currentAccentHex}
                    oninput={handleColorWheelChange}
                    class="absolute -inset-4 h-16 w-20 cursor-pointer bg-transparent"
                    title="Click to choose accent color"
                  />
                </div>
                <input
                  type="text"
                  bind:value={currentAccentHex}
                  oninput={handleManualHexInput}
                  placeholder="#76B900"
                  maxlength="7"
                  class="w-32 px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs font-mono text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent uppercase text-center font-bold tracking-wider"
                />
              </div>
            </div>

            <div class="h-px bg-nvidia-border/50"></div>

            <!-- Typography Grid -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-2">
                <div class="flex items-center gap-2 text-xs font-semibold text-nvidia-text-primary">
                  <Type class="h-3.5 w-3.5 text-nvidia-accent" />
                  <span>App Font</span>
                </div>
                <select
                  onchange={handleBasePresetChange}
                  class="w-full px-2 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
                >
                  {#each APP_FONT_PRESETS as preset}
                    <option value={preset.value} selected={selectedBasePreset === preset.value} class="bg-nvidia-surface text-nvidia-text-primary">
                      {preset.label}
                    </option>
                  {/each}
                  <option value="custom" selected={selectedBasePreset === 'custom'} class="bg-nvidia-surface text-nvidia-accent font-semibold">
                    + Custom Font...
                  </option>
                </select>
                {#if selectedBasePreset === 'custom'}
                  <input
                    type="text"
                    bind:value={customBaseInput}
                    oninput={handleCustomBaseInput}
                    placeholder="Font name..."
                    class="w-full px-2 py-1.5 bg-nvidia-surface border border-nvidia-accent/60 rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent"
                  />
                {/if}
              </div>

              <div class="space-y-2">
                <div class="flex items-center gap-2 text-xs font-semibold text-nvidia-text-primary">
                  <FileCode class="h-3.5 w-3.5 text-cyan-400" />
                  <span>Mod Font</span>
                </div>
                <select
                  onchange={handleModsPresetChange}
                  class="w-full px-2 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-cyan-400 cursor-pointer font-sans"
                >
                  {#each MOD_FONT_PRESETS as preset}
                    <option value={preset.value} selected={selectedModsPreset === preset.value} class="bg-nvidia-surface text-nvidia-text-primary">
                      {preset.label}
                    </option>
                  {/each}
                  <option value="custom" selected={selectedModsPreset === 'custom'} class="bg-nvidia-surface text-cyan-400 font-semibold">
                    + Custom Font...
                  </option>
                </select>
                {#if selectedModsPreset === 'custom'}
                  <input
                    type="text"
                    bind:value={customModsInput}
                    oninput={handleCustomModsInput}
                    placeholder="Font name..."
                    class="w-full px-2 py-1.5 bg-nvidia-surface border border-cyan-500/60 rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-cyan-400"
                  />
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Live Preview Mockup -->
        <div
          class="rounded-xl border shadow-inner p-4 space-y-3 transition-colors duration-200"
          style="
            background-color: {activeColors.bg};
            border-color: {activeColors.border};
            font-family: {draft.fontFamilyBase};
            color: {activeColors.textPrimary};
          "
        >
          <div class="flex items-center gap-1.5 opacity-60 mb-2" style="color: {activeColors.textPrimary};">
            <Palette class="h-3.5 w-3.5" style="color: {activeColors.accent};" />
            <span class="text-[10px] font-bold uppercase tracking-wider">Theme Preview</span>
          </div>

          <!-- Mock Category Row -->
          <div
            class="rounded-lg border flex items-center justify-between px-3 py-1.5 relative overflow-hidden"
            style="background-color: {activeColors.accent}14; border-color: {activeColors.accent}59;"
          >
            <div class="absolute left-0 top-0 bottom-0 w-1" style="background-color: {activeColors.accent};"></div>
            <div class="flex items-center gap-3 pl-1">
              <GripVertical class="h-4 w-4" style="color: {activeColors.textMuted};" />
              <div class="w-7 h-4 rounded-full relative p-0.5 shrink-0" style="background-color: #3f3f46; border: 1px solid #52525b;">
                <div class="h-2.5 w-2.5 rounded-full bg-zinc-100 translate-x-3"></div>
              </div>
              <div class="flex items-center gap-1.5">
                <ChevronDown class="h-4 w-4" style="color: {activeColors.textMuted};" />
                <Folder class="h-4 w-4" style="color: {activeColors.textMuted};" />
                <span class="font-bold uppercase tracking-wider text-xs">CORE FRAMEWORK</span>
              </div>
            </div>
          </div>

          <!-- Mock Mod Row -->
          <div
            class="rounded border flex items-center justify-between px-3 ml-3"
            style="
              background-color: {activeColors.surface};
              border-color: {activeColors.border};
              height: 1.75rem;
            "
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <GripVertical class="h-3.5 w-3.5 shrink-0" style="color: {activeColors.textMuted};" />
              <div class="w-7 h-4 rounded-full relative p-0.5 shrink-0" style="background-color: #3f3f46; border: 1px solid #52525b;">
                <div class="h-2.5 w-2.5 rounded-full bg-zinc-100 translate-x-3"></div>
              </div>
              <span class="text-[10px] px-1 py-0.2 rounded border shrink-0" style="background-color: {activeColors.card}; color: {activeColors.textMuted}; border-color: {activeColors.border}; font-family: {draft.fontFamilyMods};">
                #1
              </span>
              <span class="text-[11px] font-medium truncate" style="color: {activeColors.textPrimary}; font-family: {draft.fontFamilyMods};">
                EquipmentEx.archive
              </span>
              <span class="text-[9px] font-bold px-1.5 py-0.2 rounded border shrink-0" style="background-color: #083344cc; color: #22d3ee; border-color: #0e749099; font-family: {draft.fontFamilyMods};">
                XL
              </span>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <div class="w-3 flex items-center justify-center shrink-0">
                <div class="h-2.5 w-2.5 rounded-full bg-[#22c55e]" style="box-shadow: 0 0 6px rgba(34,197,94,0.7);"></div>
              </div>
              <div class="flex items-center gap-1 px-1 py-0.5 rounded text-[11px]" style="color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
                <span class="font-medium">2</span>
                <ChevronRight class="h-3 w-3" />
              </div>
              <MoreVertical class="h-3.5 w-3.5" style="color: {activeColors.textMuted};" />
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>

  <!-- Footer Action Bar -->
  <div class="px-6 py-4 border-t border-nvidia-border bg-nvidia-surface/40 flex items-center justify-between shrink-0">
    
    <!-- Diagnostics moved to footer -->
    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={handleClearLogs}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded hover:bg-red-500/10 border border-transparent hover:border-red-500/30 text-xs text-nvidia-text-muted hover:text-red-400 transition cursor-pointer"
      >
        {#if clearedFeedback}
          <Check class="h-3.5 w-3.5 text-nvidia-accent" />
          <span class="text-nvidia-accent">Cleared!</span>
        {:else}
          <Trash2 class="h-3.5 w-3.5" />
          <span>Clear Logs</span>
        {/if}
      </button>
      <div class="w-px h-4 bg-nvidia-border/60"></div>
      <button
        type="button"
        onclick={handleOpenLogs}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border text-xs text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
      >
        <FolderOpen class="h-3.5 w-3.5" />
        <span>Open Log Folder</span>
      </button>
    </div>

    <div class="flex items-center gap-3">
      <button
        type="button"
        onclick={handleResetDefault}
        class="flex items-center gap-2 px-4 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer"
        title="Restore factory default for active mode"
      >
        <RotateCcw class="h-3.5 w-3.5 text-nvidia-text-muted" />
        <span>Reset {draft.mode === 'light' ? 'Light' : 'Dark'} Defaults</span>
      </button>

      <button
        type="button"
        onclick={handleApply}
        disabled={!isDirty}
        class="flex items-center gap-2 px-6 py-1.5 rounded font-semibold text-xs transition shadow-sm cursor-pointer {isDirty ? 'bg-nvidia-accent hover:brightness-105 text-black' : 'bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border cursor-not-allowed'}"
      >
        <Check class="h-4 w-4 {isDirty ? 'stroke-[2.5]' : ''}" />
        <span>{isDirty ? 'Apply Settings' : 'Settings Saved'}</span>
      </button>
    </div>
  </div>
</div>
