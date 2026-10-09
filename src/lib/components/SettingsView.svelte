<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
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
    Crown,
    Type,
    Sparkles,
    FileText,
    FolderOpen,
    FileCode
  } from 'lucide-svelte';

  let draft = $state<ThemeSettings>(loadThemeSettings());
  let currentAccentHex = $state('#76B900');
  let selectedBasePreset = $state<string>(APP_FONT_PRESETS[0].value);
  let customBaseInput = $state('');
  let selectedModsPreset = $state<string>(MOD_FONT_PRESETS[0].value);
  let customModsInput = $state('');
  let customEditorPath = $state('');
  let isDirty = $state(false);

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
    });
  });

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

  <div class="flex-1 overflow-y-auto px-6 pb-6 space-y-6">
    <!-- Mode Switch -->
    <div class="flex items-center justify-start">
      <div class="flex items-center p-0.5 rounded-lg border border-nvidia-border bg-nvidia-surface/60">
        <button
          type="button"
          onclick={() => handleModeToggle('dark')}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold transition cursor-pointer {draft.mode === 'dark' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
        >
          <Moon class="h-3.5 w-3.5 text-nvidia-accent" />
          <span>Dark Mode</span>
        </button>
        <button
          type="button"
          onclick={() => handleModeToggle('light')}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold transition cursor-pointer {draft.mode === 'light' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
        >
          <Sun class="h-3.5 w-3.5 text-amber-500" />
          <span>Light Mode</span>
        </button>
      </div>
    </div>

    <!-- Live Preview Mockup -->
    <div
      class="rounded-lg border shadow-inner p-4 space-y-3 transition-colors duration-200 max-w-3xl"
      style="
        background-color: {activeColors.bg};
        border-color: {activeColors.border};
        font-family: {draft.fontFamilyBase};
        color: {activeColors.textPrimary};
      "
    >
      <!-- Mockup Header -->
      <div
        class="px-3 py-2 rounded border flex items-center justify-between"
        style="
          background-color: {activeColors.surface};
          border-color: {activeColors.border};
        "
      >
        <div class="flex items-center gap-2">
          <div
            class="h-5 w-5 rounded flex items-center justify-center text-[10px] font-black"
            style="background-color: {activeColors.accent}; color: #000000;"
          >
            R4
          </div>
          <span class="text-xs font-bold tracking-wider uppercase">R4 MOD TOOLBOX</span>
        </div>
        <span class="text-[10px] px-2 py-0.5 rounded border font-mono" style="background-color: {activeColors.card}; border-color: {activeColors.border}; color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
          G:\Cyberpunk 2077
        </span>
      </div>

      <!-- Mockup Nav & Mod Row -->
      <div class="flex gap-3 items-start">
        <div class="w-32 space-y-1.5 shrink-0">
          <div
            class="px-2.5 py-1.5 rounded text-xs font-bold flex items-center justify-between"
            style="background-color: {activeColors.card}; color: {activeColors.accent}; border: 1px solid {activeColors.border};"
          >
            <span>Archive</span>
            <span class="text-[10px] font-mono opacity-80" style="font-family: {draft.fontFamilyMods};">45</span>
          </div>
          <div
            class="px-2.5 py-1.5 rounded text-xs opacity-75"
            style="color: {activeColors.textMuted};"
          >
            <span>CET Mods</span>
          </div>
        </div>

        <!-- Mod Row (28px height) -->
        <div
          class="flex-1 rounded border flex items-center justify-between px-3 transition-all duration-150"
          style="
            background-color: {activeColors.card};
            border-color: {activeColors.border};
            height: 1.75rem;
            padding-top: 0.125rem;
            padding-bottom: 0.125rem;
          "
        >
          <div class="flex items-center gap-2 min-w-0">
            <GripVertical class="h-3.5 w-3.5 shrink-0" style="color: {activeColors.textMuted};" />
            <div class="w-6 h-3.5 rounded-full relative p-0.5 shrink-0" style="background-color: {activeColors.accent};">
              <div class="h-2.5 w-2.5 rounded-full bg-black translate-x-2.5"></div>
            </div>
            <span class="text-xs font-medium truncate" style="color: {activeColors.textPrimary}; font-family: {draft.fontFamilyMods};">
              EquipmentEx.archive
            </span>
            <span class="text-[9px] font-bold px-1.5 py-0.5 rounded border shrink-0" style="background-color: {activeColors.surface}; color: {activeColors.accent}; border-color: {activeColors.accent}; font-family: {draft.fontFamilyMods};">
              XL
            </span>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            <span class="text-[10px] shrink-0" style="color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
              (142 assets)
            </span>
            <div class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium" style="background-color: {activeColors.accent}25; color: {activeColors.accent}; border: 1px solid {activeColors.accent}50;">
              <Crown class="h-2.5 w-2.5" />
              <span>Winning</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 2-Column Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-4xl">
      <!-- Left Column: Accent Color -->
      <div class="p-4 rounded-xl border border-nvidia-border bg-nvidia-surface/50 flex flex-col justify-between space-y-3">
        <div class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted">
          <Sparkles class="h-4 w-4 text-nvidia-accent" />
          <span>Accent Color</span>
        </div>

        <div class="flex items-center gap-3 py-1">
          <div class="relative h-10 w-14 rounded-lg border border-nvidia-border overflow-hidden shrink-0 shadow-sm hover:border-nvidia-accent transition cursor-pointer">
            <input
              type="color"
              value={currentAccentHex}
              oninput={handleColorWheelChange}
              class="absolute -inset-4 h-20 w-24 cursor-pointer bg-transparent"
              title="Click to choose accent color"
            />
          </div>

          <div class="flex-1">
            <input
              type="text"
              bind:value={currentAccentHex}
              oninput={handleManualHexInput}
              placeholder="#76B900"
              maxlength="7"
              class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded-lg text-sm font-mono text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent uppercase text-center font-bold tracking-wider"
            />
          </div>
        </div>

        <p class="text-xs text-nvidia-text-muted leading-tight">
          Foundational shades derive automatically from your chosen accent.
        </p>
      </div>

      <!-- Right Column: Typography -->
      <div class="p-4 rounded-xl border border-nvidia-border bg-nvidia-surface/50 space-y-3">
        <div class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted">
          <Type class="h-4 w-4 text-nvidia-accent" />
          <span>Typography</span>
        </div>

        <!-- Application Font -->
        <div class="space-y-1">
          <span class="text-xs text-nvidia-text-muted font-medium">Application</span>
          <select
            onchange={handleBasePresetChange}
            class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded-md text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
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
              placeholder="Enter font name..."
              class="w-full px-3 py-1.5 bg-nvidia-surface border border-nvidia-accent/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-nvidia-accent mt-1"
            />
          {/if}
        </div>

        <!-- Mod Listing Font -->
        <div class="space-y-1">
          <span class="text-xs text-nvidia-text-muted font-medium">Mod Listing</span>
          <select
            onchange={handleModsPresetChange}
            class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded-md text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
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
              placeholder="Enter coding font name..."
              class="w-full px-3 py-1.5 bg-nvidia-surface border border-cyan-500/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-cyan-400 mt-1"
            />
          {/if}
        </div>
      </div>
    </div>

    <!-- Text Editor Configuration -->
    <div class="max-w-4xl">
      <div class="p-4 rounded-xl border border-nvidia-border bg-nvidia-surface/50 space-y-3">
        <div class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted">
          <FileCode class="h-4 w-4 text-nvidia-accent" />
          <span>Custom Text Editor</span>
        </div>
        <p class="text-xs text-nvidia-text-muted leading-tight">
          Select an executable (e.g., VS Code, Notepad++) to open configuration files. Leave blank to use Windows Notepad.
        </p>
        <div class="flex items-center gap-3">
          <input
            type="text"
            bind:value={customEditorPath}
            oninput={markDirty}
            placeholder="C:\Program Files\Notepad++\notepad++.exe"
            class="flex-1 px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/50 focus:outline-none focus:border-nvidia-accent font-mono"
          />
          <button
            type="button"
            onclick={handleBrowseEditor}
            class="px-4 py-2 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0"
          >
            Browse...
          </button>
        </div>
      </div>
    </div>

    <!-- Diagnostics & Support -->
    <div class="max-w-4xl">
      <div class="p-4 rounded-xl border border-nvidia-border bg-nvidia-surface/50 flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted">
            <FileText class="h-4 w-4 text-nvidia-accent" />
            <span>Diagnostics & Support</span>
          </div>
          <p class="text-xs text-nvidia-text-muted mt-1">
            Application logs are saved automatically. Use these files when reporting issues.
          </p>
        </div>
        <button
          type="button"
          onclick={handleOpenLogs}
          class="flex items-center gap-2 px-4 py-2 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0 shadow-xs"
        >
          <FolderOpen class="h-4 w-4 text-nvidia-text-muted" />
          <span>Open Log Folder</span>
        </button>
      </div>
    </div>

  </div>

  <!-- Footer -->
  <div class="px-6 py-4 border-t border-nvidia-border bg-nvidia-surface/40 flex items-center justify-between shrink-0">
    <button
      type="button"
      onclick={handleResetDefault}
      class="flex items-center gap-2 px-4 py-2 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer"
      title="Restore factory default for active mode"
    >
      <RotateCcw class="h-4 w-4 text-nvidia-text-muted" />
      <span>Reset {draft.mode === 'light' ? 'Light' : 'Dark'} Defaults</span>
    </button>

    <button
      type="button"
      onclick={handleApply}
      disabled={!isDirty}
      class="flex items-center gap-2 px-6 py-2 rounded font-semibold text-xs transition shadow-sm cursor-pointer {isDirty ? 'bg-nvidia-accent hover:brightness-105 text-black' : 'bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border cursor-not-allowed'}"
    >
      <Check class="h-4 w-4 {isDirty ? 'stroke-[2.5]' : ''}" />
      <span>{isDirty ? 'Apply Settings' : 'Settings Saved'}</span>
    </button>
  </div>
</div>
