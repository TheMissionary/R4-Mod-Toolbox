<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import {
    MOD_FONT_PRESETS,
    DARK_ATMOSPHERES,
    LIGHT_ATMOSPHERES,
    generateThemeColors,
    loadThemeSettings,
    applyThemeSettings,
    loadConfigFromDisk,
    saveConfigToDisk,
    type ThemeSettings
  } from '$lib/theme';
  import {
    RotateCcw, Check, Settings, Sun, Moon, Sparkles, Palette, FolderSearch,
    ChevronDown, ChevronRight, Wrench, GripVertical, Folder, FileCode, LayoutGrid, FileText, Image as ImageIcon, X, Download
  } from 'lucide-svelte';

  let { 
    gamePath = $bindable(''),
    onRefresh,
    onSettingsSaved
  }: { 
    gamePath?: string;
    onRefresh?: () => void;
    onSettingsSaved?: () => void;
  } = $props();

  let draft = $state<ThemeSettings>(loadThemeSettings());
  let currentAccentHex = $state('#76B900');
  let selectedModsPreset = $state<string>(MOD_FONT_PRESETS[0].value);
  let customModsInput = $state('');
  let customEditorPath = $state('');
  let customPhotoPath = $state('');
  let enableOrganizerView = $state(false);
  let isDirty = $state(false);
  
  let showAdvancedIntegrations = $state(false);
  let isExporting = $state(false);
  let exportSuccess = $state(false);

  let activeColors = $derived(generateThemeColors(draft.mode, draft.atmosphere, currentAccentHex));
  let availableAtmospheres = $derived(draft.mode === 'light' ? Object.keys(LIGHT_ATMOSPHERES) : Object.keys(DARK_ATMOSPHERES));
  let previewBannerUrl = $derived(draft.heroBannerPath ? convertFileSrc(draft.heroBannerPath) : null);

  function markDirty() { isDirty = true; }

  onMount(() => {
    draft = loadThemeSettings();
    currentAccentHex = draft.accentHex.toUpperCase();

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
      customPhotoPath = config.customPhotoPath || '';
      enableOrganizerView = config.enableOrganizerView || false;
    });
  });

  async function chooseDirectory() {
    try {
      const selected = await open({ directory: true, multiple: false, title: 'Select Cyberpunk 2077 Root Folder', defaultPath: gamePath || undefined });
      if (typeof selected === 'string') {
        const resolved = await invoke<string | null>('validate_and_resolve_game_path', { candidatePath: selected });
        if (resolved) {
          gamePath = resolved;
          try {
            const config = await loadConfigFromDisk();
            config.targetGamePath = resolved;
            await saveConfigToDisk(config);
          } catch (e) { console.error(e); }
          if (onRefresh) onRefresh();
        }
      }
    } catch (err) { console.error(err); }
  }

  function setAccentColor(hex: string) {
    currentAccentHex = hex.toUpperCase();
    draft.accentHex = currentAccentHex;
    markDirty();
  }

  function handleModeToggle(newMode: 'dark' | 'light') {
    draft.mode = newMode;
    draft.atmosphere = newMode === 'light' ? 'cloud' : 'graphite';
    markDirty();
  }

  function handleAtmosphereSelect(atm: string) {
    draft.atmosphere = atm;
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
    if (/^#[0-9A-Fa-f]{6}$/.test(val)) { setAccentColor(val); }
  }

  function handleModsPresetChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedModsPreset = val;
    if (val !== 'custom') { draft.fontFamilyMods = val; } 
    else { draft.fontFamilyMods = customModsInput ? `"${customModsInput}", monospace` : MOD_FONT_PRESETS[0].value; }
    markDirty();
  }

  function handleCustomModsInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.trim();
    customModsInput = raw;
    draft.fontFamilyMods = raw ? `"${raw}", monospace` : MOD_FONT_PRESETS[0].value;
    markDirty();
  }

  async function handleBrowseBanner() {
    try {
      const selected = await open({
        multiple: false, directory: false,
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
        title: 'Select Hero Banner Image'
      });
      if (typeof selected === 'string') {
        draft.heroBannerPath = selected;
        markDirty();
      }
    } catch (err) { console.error(err); }
  }

  function handleClearBanner() {
    draft.heroBannerPath = undefined;
    markDirty();
  }

  async function handleBrowseEditor() {
    try {
      const selected = await open({ multiple: false, directory: false, filters: [{ name: 'Executables', extensions: ['exe'] }], title: 'Select Custom Text Editor' });
      if (typeof selected === 'string') { customEditorPath = selected; markDirty(); }
    } catch (err) { console.error(err); }
  }

  async function handleBrowsePhotoPath() {
    try {
      const selected = await open({ directory: true, multiple: false, title: 'Select Cyberpunk 2077 Photo Folder' });
      if (typeof selected === 'string') { customPhotoPath = selected; markDirty(); }
    } catch (err) { console.error(err); }
  }

  async function handleExportModList() {
    if (!gamePath) return;
    try {
      const savePath = await save({ filters: [{ name: 'Text Document', extensions: ['txt'] }], defaultPath: 'R4_Mod_List.txt', title: 'Export Mod List' });
      if (savePath) {
        isExporting = true;
        await invoke('export_mod_list', { gamePath, exportPath: savePath });
        exportSuccess = true;
        setTimeout(() => { exportSuccess = false; }, 2500);
      }
    } catch (err) { console.error(err); } finally { isExporting = false; }
  }

  async function handleImportThemePack() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'R4 Theme Pack', extensions: ['r4theme', 'zip'] }],
        title: 'Import Theme Pack'
      });
      if (typeof selected === 'string') {
        const importedTheme = await invoke<ThemeSettings>('import_theme_pack', { filePath: selected });
        draft = { ...draft, ...importedTheme };
        currentAccentHex = draft.accentHex.toUpperCase();
        
        const matchedMods = MOD_FONT_PRESETS.find(p => p.value === draft.fontFamilyMods);
        if (matchedMods) {
          selectedModsPreset = matchedMods.value;
          customModsInput = '';
        } else {
          selectedModsPreset = 'custom';
          customModsInput = draft.fontFamilyMods.replace(/^"|"$/g, '').replace(/, monospace$/, '');
        }
        
        markDirty();
      }
    } catch (err) {
      console.error('Failed to import theme pack:', err);
    }
  }

  // State-aware reset: preserves active mode and restores mode-specific atmosphere
  function handleResetDefault() {
    if (draft.mode === 'light') {
      draft.mode = 'light';
      draft.atmosphere = 'cloud';
    } else {
      draft.mode = 'dark';
      draft.atmosphere = 'graphite';
    }
    draft.accentHex = '#76B900';
    currentAccentHex = '#76B900';
    draft.fontFamilyMods = MOD_FONT_PRESETS[0].value;
    draft.heroBannerPath = undefined;
    draft.sidebarImage = undefined;
    draft.surfaceOpacity = undefined;
    draft.backdropBlur = undefined;
    draft.uiRounding = undefined;
    draft.borderThickness = undefined;
    draft.neonIntensity = undefined;
    selectedModsPreset = MOD_FONT_PRESETS[0].value;
    customModsInput = '';
    customEditorPath = '';
    customPhotoPath = '';
    enableOrganizerView = false;
    markDirty();
  }

  async function handleApply() {
    if (selectedModsPreset === 'custom' && customModsInput.trim()) {
      draft.fontFamilyMods = `"${customModsInput.trim()}", monospace`;
    }
    const plainPayload: ThemeSettings = JSON.parse(JSON.stringify(draft));
    applyThemeSettings(plainPayload);
    
    try {
      let config = await loadConfigFromDisk();
      config.theme = plainPayload;
      config.customTextEditorPath = customEditorPath.trim();
      config.customPhotoPath = customPhotoPath.trim();
      config.enableOrganizerView = enableOrganizerView;
      await saveConfigToDisk(config);
      isDirty = false;
      window.dispatchEvent(new CustomEvent('theme-updated'));
      if (onSettingsSaved) onSettingsSaved();
    } catch (err) { console.error(err); }
  }
</script>

<div class="flex flex-col h-full overflow-hidden select-none" style="background-color: var(--theme-bg); color: var(--theme-text-primary);">
  <div class="p-6 pb-4 shrink-0">
    <div class="flex items-center gap-2 text-xs font-mono mb-2" style="color: var(--theme-accent);">
      <Settings class="h-4 w-4" />
      <span>APPLICATION SETTINGS</span>
    </div>
    <h2 class="text-xl font-bold mb-1" style="color: var(--theme-text-primary);">Configuration & Theming</h2>
    <p class="text-xs max-w-xl" style="color: var(--theme-text-muted);">
      Customize the visual appearance, typography, and external tool integrations for the R4 Mod Toolbox.
    </p>
  </div>

  <div class="flex-1 overflow-y-auto px-6 pb-6">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-6xl">
      
      <!-- Left Column: Unified Theme Engine -->
      <div class="rounded-xl border flex flex-col h-fit overflow-hidden" style="background-color: var(--theme-card); border-color: var(--theme-border);">
        <div class="px-4 py-3 border-b flex items-center justify-between gap-2.5" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
          <div class="flex items-center gap-2.5">
            <Palette class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Appearance & Theming</h3>
          </div>
          <button
            type="button"
            onclick={handleImportThemePack}
            class="flex items-center gap-1.5 px-3 py-1 rounded border text-[10px] font-bold uppercase tracking-wider transition cursor-pointer hover:brightness-125"
            style="background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: color-mix(in srgb, var(--theme-accent) 40%, transparent); color: var(--theme-accent);"
          >
            <Download class="h-3.5 w-3.5" />
            <span>Import .r4theme</span>
          </button>
        </div>

        <div class="p-5 space-y-6">
          
          <!-- Top Row: Mode & Accent -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <!-- Mode Switch -->
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
                <Sun class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
                <span>Color Mode</span>
              </div>
              <div class="flex items-center p-0.5 rounded border w-fit" style="background-color: var(--theme-bg); border-color: var(--theme-border);">
                <button
                  type="button"
                  onclick={() => handleModeToggle('dark')}
                  class="flex items-center gap-1.5 px-3 py-1.5 rounded-sm text-[11px] font-bold uppercase tracking-wider transition cursor-pointer border"
                  style="{draft.mode === 'dark' ? 'background-color: var(--theme-card); color: var(--theme-text-primary); border-color: var(--theme-border);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
                >
                  <Moon class="h-3.5 w-3.5" style="{draft.mode === 'dark' ? 'color: var(--theme-accent);' : ''}" />
                  <span>Dark</span>
                </button>
                <button
                  type="button"
                  onclick={() => handleModeToggle('light')}
                  class="flex items-center gap-1.5 px-3 py-1.5 rounded-sm text-[11px] font-bold uppercase tracking-wider transition cursor-pointer border"
                  style="{draft.mode === 'light' ? 'background-color: var(--theme-card); color: var(--theme-text-primary); border-color: var(--theme-border);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
                >
                  <Sun class="h-3.5 w-3.5" style="{draft.mode === 'light' ? 'color: var(--theme-accent);' : ''}" />
                  <span>Light</span>
                </button>
              </div>
            </div>

            <!-- Accent Color -->
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
                <Sparkles class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
                <span>Accent Color</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="relative h-8 w-12 rounded border overflow-hidden shrink-0 shadow-sm transition cursor-pointer" style="border-color: var(--theme-border);">
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
                  class="w-28 px-3 py-1.5 border rounded text-xs font-mono focus:outline-none uppercase text-center font-bold tracking-wider"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                />
              </div>
            </div>
          </div>

          <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

          <!-- Base Atmosphere Selection -->
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
              <LayoutGrid class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
              <span>Base Atmosphere</span>
            </div>
            <div class="flex items-center gap-3 pt-1">
              {#each availableAtmospheres as atm}
                {@const atmColors = draft.mode === 'light' ? LIGHT_ATMOSPHERES[atm] : DARK_ATMOSPHERES[atm]}
                <button
                  type="button"
                  onclick={() => handleAtmosphereSelect(atm)}
                  class="h-8 w-8 rounded-full border-2 transition-all cursor-pointer shadow-sm"
                  style="background-color: {atmColors.bg}; border-color: {draft.atmosphere === atm ? 'var(--theme-accent)' : 'var(--theme-border)'}; {draft.atmosphere === atm ? 'transform: scale(1.1);' : ''}"
                  title={atm.charAt(0).toUpperCase() + atm.slice(1)}
                ></button>
              {/each}
            </div>
          </div>

          <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

          <!-- Typography & Banner Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
                <FileCode class="h-3.5 w-3.5" style="color: var(--color-info);" />
                <span>Data Font (Monospace)</span>
              </div>
              <select
                onchange={handleModsPresetChange}
                class="w-full px-2 py-1.5 border rounded text-xs focus:outline-none cursor-pointer font-sans"
                style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
              >
                {#each MOD_FONT_PRESETS as preset}
                  <option value={preset.value} selected={selectedModsPreset === preset.value} style="background-color: var(--theme-surface); color: var(--theme-text-primary);">
                    {preset.label}
                  </option>
                {/each}
                <option value="custom" selected={selectedModsPreset === 'custom'} style="background-color: var(--theme-surface); color: var(--color-info); font-weight: bold;">
                  + Custom Font...
                </option>
              </select>
              {#if selectedModsPreset === 'custom'}
                <input
                  type="text"
                  bind:value={customModsInput}
                  oninput={handleCustomModsInput}
                  placeholder="Font name..."
                  class="w-full px-2 py-1.5 border rounded text-xs focus:outline-none"
                  style="background-color: var(--theme-surface); border-color: color-mix(in srgb, var(--color-info) 60%, transparent); color: var(--theme-text-primary);"
                />
              {/if}
            </div>

            <div class="space-y-2">
              <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
                <ImageIcon class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
                <span>Hero Banner Image</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={handleBrowseBanner}
                  class="flex-1 px-3 py-1.5 rounded border text-xs transition cursor-pointer truncate text-left"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  title={draft.heroBannerPath || 'Select Image...'}
                >
                  {draft.heroBannerPath ? draft.heroBannerPath.split('\\').pop() : 'Select Image...'}
                </button>
                {#if draft.heroBannerPath}
                  <button
                    type="button"
                    onclick={handleClearBanner}
                    class="p-1.5 rounded transition cursor-pointer hover:bg-[var(--theme-surface)]"
                    style="color: var(--color-danger);"
                    title="Clear Banner"
                  >
                    <X class="h-3.5 w-3.5" />
                  </button>
                {/if}
              </div>
            </div>
          </div>

          <!-- Live Preview Mockup -->
          <div class="pt-2">
            <div class="text-[10px] font-bold uppercase tracking-wider mb-2" style="color: var(--theme-text-muted);">Live Preview</div>
            <div
              class="rounded-lg border overflow-hidden flex flex-col shadow-inner relative"
              style="background-color: {activeColors.bg}; border-color: {activeColors.border};"
            >
              <!-- Mini Header with Banner -->
              <div class="relative px-3 py-2 border-b flex items-center justify-between overflow-hidden" style="background-color: {activeColors.bg}; border-color: {activeColors.border};">
                {#if previewBannerUrl}
                  <div class="absolute inset-0 z-0 pointer-events-none">
                    <img src={previewBannerUrl} alt="Preview Banner" class="absolute inset-0 w-full h-full object-cover object-[center_20%] opacity-60" />
                    <div class="absolute inset-0" style="background: linear-gradient(to bottom, color-mix(in srgb, {activeColors.bg} 60%, transparent) 0%, {activeColors.bg} 100%);"></div>
                  </div>
                {/if}
                <div class="relative z-10 flex items-center gap-2">
                  <div class="h-4 w-4 rounded flex items-center justify-center text-[8px] font-black" style="background-color: {activeColors.accent}; color: {activeColors.accentContrast};">R4</div>
                  <span class="text-[10px] font-bold tracking-wider uppercase" style="color: {activeColors.textPrimary};">R4 Mod Toolbox</span>
                </div>
                <div class="relative z-10 h-4 w-16 rounded" style="background-color: {activeColors.accent};"></div>
              </div>
              
              <!-- Mini Body (Seamless List) -->
              <div class="p-3 space-y-0">
                <!-- Group -->
                <div class="border-b px-2 py-1.5 flex items-center gap-2 relative overflow-hidden" style="border-color: {activeColors.border};">
                  <div class="absolute left-0 top-0 bottom-0 w-0.5" style="background-color: {activeColors.accent};"></div>
                  <ChevronDown class="h-3 w-3" style="color: {activeColors.textMuted};" />
                  <Folder class="h-3 w-3" style="color: {activeColors.textMuted};" />
                  <span class="text-[9px] font-bold uppercase tracking-wider" style="color: {activeColors.textPrimary};">Gameplay Structure</span>
                </div>
                
                <!-- Mod -->
                <div class="border-b px-2 py-1.5 flex items-center justify-between ml-2" style="border-color: {activeColors.border};">
                  <div class="flex items-center gap-1.5">
                    <GripVertical class="h-3 w-3" style="color: {activeColors.textMuted};" />
                    <div class="w-3 h-3 rounded-sm border flex items-center justify-center" style="background-color: {activeColors.accent}; border-color: {activeColors.accent};">
                      <Check class="h-2.5 w-2.5" style="color: {activeColors.accentContrast};" strokeWidth={3} />
                    </div>
                    <span class="text-[10px]" style="color: {activeColors.textPrimary}; font-family: {draft.fontFamilyMods};">EquipmentEx.archive</span>
                    <span class="text-[7px] font-bold px-1 py-0.5 rounded border" style="background-color: color-mix(in srgb, var(--color-info) 15%, transparent); color: var(--color-info); border-color: color-mix(in srgb, var(--color-info) 40%, transparent); font-family: {draft.fontFamilyMods};">XL</span>
                  </div>
                  <div class="flex items-center gap-1">
                    <div class="h-1.5 w-1.5 rounded-full" style="background-color: var(--color-success); box-shadow: 0 0 4px var(--color-success);"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

        </div>
      </div>

      <!-- Right Column: Directories & Integrations -->
      <div class="flex flex-col gap-4">
        <div class="rounded-xl border flex flex-col h-fit overflow-hidden" style="background-color: var(--theme-card); border-color: var(--theme-border);">
          <div class="px-4 py-3 border-b flex items-center gap-2.5" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
            <FolderSearch class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Game Directory</h3>
          </div>
          
          <div class="p-5 space-y-2">
            <p class="text-[11px] leading-tight" style="color: var(--theme-text-muted);">
              The root installation folder for Cyberpunk 2077. Required for the toolbox to function.
            </p>
            <div class="flex items-center gap-2 pt-1">
              <div class="flex-1 px-3 py-1.5 border rounded text-xs font-mono truncate select-all" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);">
                {gamePath || 'No folder selected'}
              </div>
              <button
                type="button"
                onclick={chooseDirectory}
                class="px-3 py-1.5 rounded border text-xs font-semibold transition cursor-pointer shrink-0 shadow-xs hover:brightness-125"
                style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
              >
                Change...
              </button>
            </div>
          </div>
        </div>

        <!-- Advanced Integrations Collapsible -->
        <div class="rounded-xl border flex flex-col h-fit overflow-hidden transition-all" style="background-color: var(--theme-card); border-color: var(--theme-border);">
          <button
            type="button"
            onclick={() => showAdvancedIntegrations = !showAdvancedIntegrations}
            class="w-full px-4 py-3 flex items-center justify-between cursor-pointer transition-colors hover:brightness-125"
            style="background-color: var(--theme-surface);"
          >
            <div class="flex items-center gap-2.5">
              <Wrench class="h-4 w-4" style="color: var(--theme-text-muted);" />
              <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Advanced Integrations</h3>
            </div>
            {#if showAdvancedIntegrations}
              <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
            {:else}
              <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
            {/if}
          </button>

          {#if showAdvancedIntegrations}
            <div class="p-5 space-y-6 border-t" style="background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent); border-color: color-mix(in srgb, var(--theme-border) 60%, transparent);">
              
              <!-- Organizer View Toggle -->
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <LayoutGrid class="h-4 w-4" style="color: var(--color-info);" />
                    <span class="text-xs font-semibold" style="color: var(--theme-text-primary);">Enable Organizer View</span>
                  </div>
                  <button
                    type="button"
                    onclick={() => { enableOrganizerView = !enableOrganizerView; markDirty(); }}
                    class="w-9 h-5 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer border"
                    style="{enableOrganizerView ? 'background-color: var(--theme-accent); border-color: var(--theme-accent);' : 'background-color: var(--theme-surface); border-color: var(--theme-border);'}"
                  >
                    <div class="h-3.5 w-3.5 rounded-full transition-transform transform {enableOrganizerView ? 'translate-x-4 shadow-xs' : 'translate-x-0'}" style="background-color: {enableOrganizerView ? 'var(--theme-accent-contrast)' : 'var(--theme-text-muted)'};"></div>
                  </button>
                </div>
                <p class="text-[11px] leading-tight pl-6" style="color: var(--theme-text-muted);">
                  Adds a virtual category layer to the Archive tab for visual organization without altering physical load order.
                </p>
              </div>

              <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

              <!-- Custom Text Editor -->
              <div class="space-y-2">
                <span class="text-xs font-semibold" style="color: var(--theme-text-primary);">Custom Text Editor</span>
                <p class="text-[11px] leading-tight" style="color: var(--theme-text-muted);">
                  Select an executable (e.g., VS Code, Notepad++) to open configuration files.
                </p>
                <div class="flex items-center gap-2">
                  <input
                    type="text"
                    bind:value={customEditorPath}
                    oninput={markDirty}
                    placeholder="Leave blank for Notepad"
                    class="flex-1 px-3 py-1.5 border rounded text-xs focus:outline-none font-mono"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  />
                  <button
                    type="button"
                    onclick={handleBrowseEditor}
                    class="px-3 py-1.5 rounded border text-xs transition cursor-pointer shrink-0 hover:brightness-125"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  >
                    Browse...
                  </button>
                </div>
              </div>

              <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

              <!-- Custom Photo Folder -->
              <div class="space-y-2">
                <span class="text-xs font-semibold" style="color: var(--theme-text-primary);">Cyberpunk 2077 Photo Folder</span>
                <p class="text-[11px] leading-tight" style="color: var(--theme-text-muted);">
                  Select your in-game photo mode screenshot directory to enable quick access from the sidebar.
                </p>
                <div class="flex items-center gap-2">
                  <input
                    type="text"
                    bind:value={customPhotoPath}
                    oninput={markDirty}
                    placeholder="Leave blank to disable"
                    class="flex-1 px-3 py-1.5 border rounded text-xs focus:outline-none font-mono"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  />
                  <button
                    type="button"
                    onclick={handleBrowsePhotoPath}
                    class="px-3 py-1.5 rounded border text-xs transition cursor-pointer shrink-0 hover:brightness-125"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  >
                    Browse...
                  </button>
                </div>
              </div>

              <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

              <!-- Export Mod List -->
              <div class="space-y-2">
                <span class="text-xs font-semibold" style="color: var(--theme-text-primary);">Export Mod List</span>
                <p class="text-[11px] leading-tight" style="color: var(--theme-text-muted);">
                  Generate a comprehensive text file containing your entire load order and framework mod lists.
                </p>
                <button
                  type="button"
                  onclick={handleExportModList}
                  disabled={isExporting || !gamePath}
                  class="px-4 py-1.5 rounded border text-xs transition cursor-pointer shadow-xs flex items-center gap-2 w-fit disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-125"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                >
                  {#if exportSuccess}
                    <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
                    <span class="font-semibold" style="color: var(--theme-accent);">Exported Successfully!</span>
                  {:else}
                    <FileText class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
                    <span>{isExporting ? 'Exporting...' : 'Export to .txt'}</span>
                  {/if}
                </button>
              </div>

            </div>
          {/if}
        </div>
      </div>

    </div>
  </div>

  <!-- Footer Action Bar -->
  <div class="px-6 py-4 border-t flex items-center justify-end shrink-0 gap-3" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
    <button
      type="button"
      onclick={handleResetDefault}
      class="flex items-center gap-2 px-4 py-1.5 rounded border text-xs transition cursor-pointer hover:brightness-125"
      style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
      title="Restore factory default for active mode"
    >
      <RotateCcw class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
      <span>Reset {draft.mode === 'light' ? 'Light' : 'Dark'} Defaults</span>
    </button>

    <button
      type="button"
      onclick={handleApply}
      disabled={!isDirty}
      class="flex items-center gap-2 px-6 py-1.5 rounded font-semibold text-xs transition shadow-sm cursor-pointer"
      style="{isDirty ? 'background-color: var(--theme-accent); color: var(--theme-accent-contrast);' : 'background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted); cursor: not-allowed;'}"
    >
      <Check class="h-4 w-4 {isDirty ? 'stroke-[2.5]' : ''}" />
      <span>{isDirty ? 'Apply Settings' : 'Settings Saved'}</span>
    </button>
  </div>
</div>