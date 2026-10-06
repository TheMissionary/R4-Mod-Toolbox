<script lang="ts">
  import { untrack } from 'svelte';
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
    X,
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

  let {
    isOpen = $bindable(false)
  }: {
    isOpen: boolean;
  } = $props();

  let draft = $state<ThemeSettings>(loadThemeSettings());
  
  let currentAccentHex = $state('#76B900');

  let selectedBasePreset = $state<string>(APP_FONT_PRESETS[0].value);
  let customBaseInput = $state('');

  let selectedModsPreset = $state<string>(MOD_FONT_PRESETS[0].value);
  let customModsInput = $state('');

  let customEditorPath = $state('');

  let activeColors = $derived(draft.mode === 'light' ? draft.lightColors : draft.darkColors);

  $effect(() => {
    if (isOpen) {
      untrack(() => {
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
    }
  });

  function setAccentColor(hex: string) {
    currentAccentHex = hex.toUpperCase();
    if (draft.mode === 'light') {
      draft.lightColors = generateThemeColors('light', hex);
    } else {
      draft.darkColors = generateThemeColors('dark', hex);
    }
  }

  function handleModeToggle(newMode: 'dark' | 'light') {
    draft.mode = newMode;
    const activePal = newMode === 'light' ? draft.lightColors : draft.darkColors;
    currentAccentHex = activePal.accent.toUpperCase();
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
  }

  function handleCustomBaseInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.trim();
    customBaseInput = raw;
    draft.fontFamilyBase = raw ? `"${raw}", sans-serif` : APP_FONT_PRESETS[0].value;
  }

  function handleModsPresetChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedModsPreset = val;
    if (val !== 'custom') {
      draft.fontFamilyMods = val;
    } else {
      draft.fontFamilyMods = customModsInput ? `"${customModsInput}", monospace` : MOD_FONT_PRESETS[0].value;
    }
  }

  function handleCustomModsInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.trim();
    customModsInput = raw;
    draft.fontFamilyMods = raw ? `"${raw}", monospace` : MOD_FONT_PRESETS[0].value;
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
    
    // Apply CSS variables immediately
    applyThemeSettings(plainPayload);
    
    // Save to disk including the new editor path
    try {
      let config = await loadConfigFromDisk();
      config.theme = plainPayload;
      config.customTextEditorPath = customEditorPath.trim();
      await saveConfigToDisk(config);
    } catch (err) {
      console.error('Failed to save config:', err);
    }
    
    isOpen = false;
  }

  function handleCancel() {
    isOpen = false;
  }

  async function handleOpenLogs() {
    try {
      await invoke('open_log_folder');
    } catch (err) {
      console.error('Failed to open log folder:', err);
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={handleCancel}
    role="presentation"
  >
    <div
      class="w-full max-w-xl rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40 shrink-0">
        <div class="flex items-center gap-2">
          <Settings class="h-4 w-4 text-nvidia-accent" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Settings</span>
        </div>
        <button
          type="button"
          onclick={handleCancel}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-3.5 overflow-y-auto">
        <!-- Top Controls: Mode Switch Only (Compact View is Permanently Locked) -->
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

        <!-- ----------------------------------------------------------------- -->
        <!-- LIVE PREVIEW MOCKUP (Permanently Compact 28px)                     -->
        <!-- ----------------------------------------------------------------- -->
        <div
          class="rounded-lg border shadow-inner p-3 space-y-2.5 transition-colors duration-200"
          style="
            background-color: {activeColors.bg};
            border-color: {activeColors.border};
            font-family: {draft.fontFamilyBase};
            color: {activeColors.textPrimary};
          "
        >
          <!-- Mockup Header -->
          <div
            class="px-2.5 py-1.5 rounded border flex items-center justify-between"
            style="
              background-color: {activeColors.surface};
              border-color: {activeColors.border};
            "
          >
            <div class="flex items-center gap-2">
              <div
                class="h-4 w-4 rounded flex items-center justify-center text-[9px] font-black"
                style="background-color: {activeColors.accent}; color: #000000;"
              >
                R4
              </div>
              <span class="text-[11px] font-bold tracking-wider uppercase">R4 MOD TOOLBOX</span>
            </div>
            <span class="text-[9px] px-1.5 py-0.2 rounded border font-mono" style="background-color: {activeColors.card}; border-color: {activeColors.border}; color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
              G:\Cyberpunk 2077
            </span>
          </div>

          <!-- Mockup Nav & Mod Row -->
          <div class="flex gap-2 items-start">
            <div class="w-24 space-y-1 shrink-0">
              <div
                class="px-2 py-1 rounded text-[10px] font-bold flex items-center justify-between"
                style="background-color: {activeColors.card}; color: {activeColors.accent}; border: 1px solid {activeColors.border};"
              >
                <span>Archive</span>
                <span class="text-[8px] font-mono opacity-80" style="font-family: {draft.fontFamilyMods};">45</span>
              </div>
              <div
                class="px-2 py-1 rounded text-[10px] opacity-75"
                style="color: {activeColors.textMuted};"
              >
                <span>CET Mods</span>
              </div>
            </div>

            <!-- Mod Row (28px height) -->
            <div
              class="flex-1 rounded border flex items-center justify-between px-2.5 transition-all duration-150"
              style="
                background-color: {activeColors.card};
                border-color: {activeColors.border};
                height: 1.75rem;
                padding-top: 0.125rem;
                padding-bottom: 0.125rem;
              "
            >
              <div class="flex items-center gap-1.5 min-w-0">
                <GripVertical class="h-3 w-3 shrink-0" style="color: {activeColors.textMuted};" />
                <div class="w-5 h-3 rounded-full relative p-0.5 shrink-0" style="background-color: {activeColors.accent};">
                  <div class="h-2 w-2 rounded-full bg-black translate-x-2"></div>
                </div>
                <span class="text-xs font-medium truncate" style="color: {activeColors.textPrimary}; font-family: {draft.fontFamilyMods};">
                  EquipmentEx.archive
                </span>
                <span class="text-[8px] font-bold px-1 rounded border shrink-0" style="background-color: {activeColors.surface}; color: {activeColors.accent}; border-color: {activeColors.accent}; font-family: {draft.fontFamilyMods};">
                  XL
                </span>
              </div>

              <div class="flex items-center gap-1.5 shrink-0">
                <span class="text-[9px] shrink-0" style="color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
                  (142 assets)
                </span>
                <div class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[9px] font-medium" style="background-color: {activeColors.accent}25; color: {activeColors.accent}; border: 1px solid {activeColors.accent}50;">
                  <Crown class="h-2 w-2" />
                  <span>Winning</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- ----------------------------------------------------------------- -->
        <!-- 2-COLUMN SIDE-BY-SIDE CONFIGURATION GRID                          -->
        <!-- ----------------------------------------------------------------- -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5 pt-1">
          <!-- Left Column: Accent Color -->
          <div class="p-3 rounded-xl border border-nvidia-border bg-nvidia-surface/50 flex flex-col justify-between space-y-2.5">
            <div class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
              <Sparkles class="h-3.5 w-3.5 text-nvidia-accent" />
              <span>Accent Color</span>
            </div>

            <div class="flex items-center gap-2.5 py-0.5">
              <div class="relative h-9 w-12 rounded-lg border border-nvidia-border overflow-hidden shrink-0 shadow-sm hover:border-nvidia-accent transition cursor-pointer">
                <input
                  type="color"
                  value={currentAccentHex}
                  oninput={handleColorWheelChange}
                  class="absolute -inset-4 h-18 w-20 cursor-pointer bg-transparent"
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
                  class="w-full px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded-lg text-xs font-mono text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent uppercase text-center font-bold tracking-wider"
                />
              </div>
            </div>

            <p class="text-[10px] text-nvidia-text-muted leading-tight">
              Foundational shades derive automatically from your chosen accent.
            </p>
          </div>

          <!-- Right Column: Typography -->
          <div class="p-3 rounded-xl border border-nvidia-border bg-nvidia-surface/50 space-y-2">
            <div class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
              <Type class="h-3.5 w-3.5 text-nvidia-accent" />
              <span>Typography</span>
            </div>

            <!-- Application Font -->
            <div class="space-y-0.5">
              <span class="text-[10px] text-nvidia-text-muted font-medium">Application</span>
              <select
                onchange={handleBasePresetChange}
                class="w-full px-2.5 py-1.5 bg-nvidia-surface border border-nvidia-border rounded-md text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
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
                  class="w-full px-2.5 py-1 bg-nvidia-surface border border-nvidia-accent/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-nvidia-accent mt-1"
                />
              {/if}
            </div>

            <!-- Mod Listing Font -->
            <div class="space-y-0.5">
              <span class="text-[10px] text-nvidia-text-muted font-medium">Mod Listing</span>
              <select
                onchange={handleModsPresetChange}
                class="w-full px-2.5 py-1.5 bg-nvidia-surface border border-nvidia-border rounded-md text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
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
                  class="w-full px-2.5 py-1 bg-nvidia-surface border border-cyan-500/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-cyan-400 mt-1"
                />
              {/if}
            </div>
          </div>
        </div>

        <!-- ----------------------------------------------------------------- -->
        <!-- TEXT EDITOR CONFIGURATION                                         -->
        <!-- ----------------------------------------------------------------- -->
        <div class="pt-1">
          <div class="p-3 rounded-xl border border-nvidia-border bg-nvidia-surface/50 space-y-2.5">
            <div class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
              <FileCode class="h-3.5 w-3.5 text-nvidia-accent" />
              <span>Custom Text Editor</span>
            </div>
            <p class="text-[10px] text-nvidia-text-muted leading-tight">
              Select an executable (e.g., VS Code, Notepad++) to open configuration files. Leave blank to use Windows Notepad.
            </p>
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={customEditorPath}
                placeholder="C:\Program Files\Notepad++\notepad++.exe"
                class="flex-1 px-2.5 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/50 focus:outline-none focus:border-nvidia-accent font-mono"
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

        <!-- ----------------------------------------------------------------- -->
        <!-- DIAGNOSTICS & SUPPORT SECTION                                     -->
        <!-- ----------------------------------------------------------------- -->
        <div class="pt-1">
          <div class="p-3 rounded-xl border border-nvidia-border bg-nvidia-surface/50 flex items-center justify-between">
            <div>
              <div class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
                <FileText class="h-3.5 w-3.5 text-nvidia-accent" />
                <span>Diagnostics & Support</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted mt-1">
                Application logs are saved automatically. Use these files when reporting issues.
              </p>
            </div>
            <button
              type="button"
              onclick={handleOpenLogs}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0 shadow-xs"
            >
              <FolderOpen class="h-3.5 w-3.5 text-nvidia-text-muted" />
              <span>Open Log Folder</span>
            </button>
          </div>
        </div>

      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-nvidia-border bg-nvidia-surface/40 flex items-center justify-between shrink-0">
        <button
          type="button"
          onclick={handleResetDefault}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer"
          title="Restore factory default for active mode"
        >
          <RotateCcw class="h-3.5 w-3.5 text-nvidia-text-muted" />
          <span>Reset {draft.mode === 'light' ? 'Light' : 'Dark'} Defaults</span>
        </button>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={handleCancel}
            class="px-3.5 py-1.5 rounded hover:bg-nvidia-surface text-xs text-nvidia-text-primary hover:text-nvidia-text-primary transition cursor-pointer"
          >
            Cancel
          </button>

          <button
            type="button"
            onclick={handleApply}
            class="flex items-center gap-1.5 px-4 py-1.5 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition shadow-sm cursor-pointer"
          >
            <Check class="h-3.5 w-3.5 stroke-[2.5]" />
            <span>Apply Settings</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
