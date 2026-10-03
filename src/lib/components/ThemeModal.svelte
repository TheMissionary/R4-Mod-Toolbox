<script lang="ts">
  import { untrack } from 'svelte';
  import {
    DEFAULT_DARK_THEME,
    DEFAULT_LIGHT_THEME,
    THEME_COLOR_META,
    APP_FONT_PRESETS,
    MOD_FONT_PRESETS,
    loadThemeSettings,
    applyAndPersistTheme,
    type AppThemeColors,
    type ThemeSettings
  } from '$lib/theme';
  import {
    X,
    RotateCcw,
    Check,
    Palette,
    Sun,
    Moon,
    GripVertical,
    Crown,
    Maximize2,
    Minimize2,
    Type,
    FileCode2
  } from 'lucide-svelte';

  let {
    isOpen = $bindable(false)
  }: {
    isOpen: boolean;
  } = $props();

  let draft = $state<ThemeSettings>(loadThemeSettings());
  let activeEditingKey = $state<keyof AppThemeColors>('accent');
  let manualHex = $state('#76B900');

  let selectedBasePreset = $state<string>(APP_FONT_PRESETS[0].value);
  let customBaseInput = $state('');

  let selectedModsPreset = $state<string>(MOD_FONT_PRESETS[0].value);
  let customModsInput = $state('');

  let activeColors = $derived(draft.mode === 'light' ? draft.lightColors : draft.darkColors);

  $effect(() => {
    if (isOpen) {
      untrack(() => {
        draft = loadThemeSettings();
        activeEditingKey = 'accent';
        const currentVal = (draft.mode === 'light' ? draft.lightColors : draft.darkColors)['accent'];
        manualHex = currentVal.toUpperCase();

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
      });
    }
  });

  function selectToken(key: keyof AppThemeColors) {
    activeEditingKey = key;
    manualHex = activeColors[key].toUpperCase();
  }

  function handleModeToggle(newMode: 'dark' | 'light') {
    draft.mode = newMode;
    const currentPalette = newMode === 'light' ? draft.lightColors : draft.darkColors;
    manualHex = currentPalette[activeEditingKey].toUpperCase();
  }

  function handleDensityToggle() {
    draft.isCompact = !draft.isCompact;
  }

  function handleColorWheelChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    if (draft.mode === 'light') {
      draft.lightColors[activeEditingKey] = val;
    } else {
      draft.darkColors[activeEditingKey] = val;
    }
    manualHex = val.toUpperCase();
  }

  function handleManualHexInput(e: Event) {
    let val = (e.target as HTMLInputElement).value.trim();
    if (!val.startsWith('#')) val = '#' + val;
    manualHex = val.toUpperCase();

    if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
      if (draft.mode === 'light') {
        draft.lightColors[activeEditingKey] = val;
      } else {
        draft.darkColors[activeEditingKey] = val;
      }
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

  function handleResetDefault() {
    if (draft.mode === 'light') {
      draft.lightColors = { ...DEFAULT_LIGHT_THEME };
    } else {
      draft.darkColors = { ...DEFAULT_DARK_THEME };
    }
    draft.fontFamilyBase = APP_FONT_PRESETS[0].value;
    draft.fontFamilyMods = MOD_FONT_PRESETS[0].value;
    selectedBasePreset = APP_FONT_PRESETS[0].value;
    selectedModsPreset = MOD_FONT_PRESETS[0].value;
    customBaseInput = '';
    customModsInput = '';
    draft.isCompact = false;
    manualHex = (draft.mode === 'light' ? DEFAULT_LIGHT_THEME : DEFAULT_DARK_THEME)[activeEditingKey].toUpperCase();
  }

  async function handleApply() {
    if (selectedBasePreset === 'custom' && customBaseInput.trim()) {
      draft.fontFamilyBase = `"${customBaseInput.trim()}", sans-serif`;
    }
    if (selectedModsPreset === 'custom' && customModsInput.trim()) {
      draft.fontFamilyMods = `"${customModsInput.trim()}", monospace`;
    }
    draft.fontFamily = draft.fontFamilyBase;

    // Unpack plain object snapshot from Svelte 5 reactive proxy
    const plainPayload: ThemeSettings = JSON.parse(JSON.stringify(draft));
    await applyAndPersistTheme(plainPayload);
    isOpen = false;
  }

  function handleCancel() {
    isOpen = false;
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={handleCancel}
    role="presentation"
  >
    <div
      class="w-full max-w-2xl rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col max-h-[92vh]"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40 shrink-0">
        <div class="flex items-center gap-2.5">
          <Palette class="h-4 w-4 text-nvidia-accent" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Theme & Display Studio</span>
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
      <div class="p-5 space-y-4 overflow-y-auto flex-1">
        <!-- Top Controls: Mode & Density Switches -->
        <div class="flex items-center justify-between gap-4">
          <!-- Light / Dark Mode Toggle -->
          <div class="flex items-center p-1 rounded-lg border border-nvidia-border bg-nvidia-surface/60">
            <button
              type="button"
              onclick={() => handleModeToggle('dark')}
              class="flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-semibold transition cursor-pointer {draft.mode === 'dark' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
            >
              <Moon class="h-3.5 w-3.5 text-nvidia-accent" />
              <span>Dark Mode</span>
            </button>
            <button
              type="button"
              onclick={() => handleModeToggle('light')}
              class="flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-semibold transition cursor-pointer {draft.mode === 'light' ? 'bg-nvidia-card text-nvidia-text-primary shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:text-nvidia-text-primary'}"
            >
              <Sun class="h-3.5 w-3.5 text-amber-500" />
              <span>Light Mode</span>
            </button>
          </div>

          <!-- Density (Compact View) Toggle -->
          <button
            type="button"
            onclick={handleDensityToggle}
            class="flex items-center gap-2 px-3.5 py-2 rounded-lg border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface transition text-xs font-medium cursor-pointer"
          >
            {#if draft.isCompact}
              <Minimize2 class="h-3.5 w-3.5 text-nvidia-accent" />
              <span class="text-nvidia-text-primary font-semibold">Compact View: <strong class="text-nvidia-accent">ON</strong> (28px)</span>
            {:else}
              <Maximize2 class="h-3.5 w-3.5 text-nvidia-text-muted" />
              <span class="text-nvidia-text-muted">Compact View: <strong class="text-nvidia-text-muted">OFF</strong> (36px)</span>
            {/if}
          </button>
        </div>

        <!-- ----------------------------------------------------------------- -->
        <!-- LIVE STAGING SANDBOX: Demonstrates Dual Fonts & Colors            -->
        <!-- ----------------------------------------------------------------- -->
        <div class="space-y-1">
          <div class="flex items-center justify-between text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
            <span>Live Staging Sandbox (Preview Only)</span>
            <span class="text-[10px] lowercase font-normal opacity-70">shows app + mod fonts</span>
          </div>

          <div
            class="rounded-lg border shadow-inner p-3.5 space-y-3 transition-colors duration-200"
            style="
              background-color: {activeColors.bg};
              border-color: {activeColors.border};
              font-family: {draft.fontFamilyBase};
              color: {activeColors.textPrimary};
            "
          >
            <!-- Mockup Header (uses App Font) -->
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
                <span class="text-xs font-bold tracking-wider uppercase">RED4 MOD TOOLBOX</span>
              </div>
              <span class="text-[10px] px-2 py-0.5 rounded border font-mono" style="background-color: {activeColors.card}; border-color: {activeColors.border}; color: {activeColors.textMuted}; font-family: {draft.fontFamilyMods};">
                G:\Cyberpunk 2077
              </span>
            </div>

            <!-- Mockup Nav & Mod Row -->
            <div class="flex gap-2.5 items-start">
              <!-- Mini Sidebar Nav (uses App Font) -->
              <div class="w-28 space-y-1 shrink-0">
                <div
                  class="px-2 py-1 rounded text-[11px] font-bold flex items-center justify-between"
                  style="background-color: {activeColors.card}; color: {activeColors.accent}; border: 1px solid {activeColors.border};"
                >
                  <span>Archive</span>
                  <span class="text-[9px] font-mono opacity-80" style="font-family: {draft.fontFamilyMods};">45</span>
                </div>
                <div
                  class="px-2 py-1 rounded text-[11px] opacity-75"
                  style="color: {activeColors.textMuted};"
                >
                  <span>CET Mods</span>
                </div>
              </div>

              <!-- Mini Mod Row (uses Mod Listing Font!) -->
              <div
                class="flex-1 rounded border flex items-center justify-between px-3 transition-all duration-150"
                style="
                  background-color: {activeColors.card};
                  border-color: {activeColors.border};
                  height: {draft.isCompact ? '1.75rem' : '2.25rem'};
                  padding-top: {draft.isCompact ? '0.125rem' : '0.375rem'};
                  padding-bottom: {draft.isCompact ? '0.125rem' : '0.375rem'};
                "
              >
                <div class="flex items-center gap-2 min-w-0">
                  <GripVertical class="h-3.5 w-3.5 shrink-0" style="color: {activeColors.textMuted};" />
                  <div class="w-6 h-3.5 rounded-full relative p-0.5 shrink-0" style="background-color: {activeColors.accent};">
                    <div class="h-2.5 w-2.5 rounded-full bg-black translate-x-2.5"></div>
                  </div>
                  <!-- Mod Filename in Mod Listing Font -->
                  <span class="text-xs font-medium truncate" style="color: {activeColors.textPrimary}; font-family: {draft.fontFamilyMods};">
                    EquipmentEx.archive
                  </span>
                  <span class="text-[9px] font-bold px-1 rounded border shrink-0" style="background-color: {activeColors.surface}; color: {activeColors.accent}; border-color: {activeColors.accent}; font-family: {draft.fontFamilyMods};">
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
        </div>

        <!-- ----------------------------------------------------------------- -->
        <!-- COLOR TOKEN PALETTE: 8 Swatches                                   -->
        <!-- ----------------------------------------------------------------- -->
        <div class="space-y-1.5 pt-1">
          <span class="text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
            Color Tokens (Click to customize)
          </span>

          <div class="grid grid-cols-4 sm:grid-cols-8 gap-2">
            {#each THEME_COLOR_META as meta}
              {@const isSelected = activeEditingKey === meta.key}
              {@const currentColor = activeColors[meta.key]}
              <button
                type="button"
                onclick={() => selectToken(meta.key)}
                class="flex flex-col items-center gap-1.5 p-2 rounded-lg border transition cursor-pointer {isSelected ? 'border-nvidia-accent bg-nvidia-surface ring-2 ring-nvidia-accent/50 shadow-sm' : 'border-nvidia-border bg-nvidia-surface/40 hover:border-nvidia-border/90'}"
              >
                <div
                  class="h-6 w-6 rounded-md border border-nvidia-border shadow-xs"
                  style="background-color: {currentColor};"
                ></div>
                <span class="text-[10px] font-mono font-medium text-nvidia-text-primary truncate w-full text-center">
                  {meta.label}
                </span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Active Color Swatch Picker -->
        {#if activeEditingKey}
          {@const meta = THEME_COLOR_META.find(m => m.key === activeEditingKey)}
          <div class="p-3 rounded-lg border border-nvidia-border bg-nvidia-surface/60 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <span class="text-xs font-bold text-nvidia-text-primary uppercase">{meta?.label} Color</span>
              <p class="text-[11px] text-nvidia-text-muted truncate">{meta?.description}</p>
            </div>

            <div class="flex items-center gap-2.5 shrink-0">
              <div class="relative h-8 w-11 rounded border border-nvidia-border overflow-hidden shrink-0 shadow-sm">
                <input
                  type="color"
                  value={activeColors[activeEditingKey]}
                  oninput={handleColorWheelChange}
                  class="absolute -inset-4 h-18 w-18 cursor-pointer bg-transparent"
                  title="Click to open color picker"
                />
              </div>

              <input
                type="text"
                bind:value={manualHex}
                oninput={handleManualHexInput}
                placeholder="#76B900"
                maxlength="7"
                class="w-24 px-2.5 py-1 bg-nvidia-surface border border-nvidia-border rounded text-xs font-mono text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent uppercase text-center"
              />
            </div>
          </div>
        {/if}

        <!-- ----------------------------------------------------------------- -->
        <!-- CLEAN DUAL TYPOGRAPHY CONTROLS (Full-Width + Custom Font Option)  -->
        <!-- ----------------------------------------------------------------- -->
        <div class="space-y-3.5 pt-1 border-t border-nvidia-border/60">
          <!-- 1. Application Base Typography -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
              <div class="flex items-center gap-1.5">
                <Type class="h-3.5 w-3.5 text-nvidia-accent" />
                <span>1. Application UI Typography (Buttons, Menus, Chrome)</span>
              </div>
              <span class="text-[10px] font-normal lowercase opacity-75">sans-serif</span>
            </div>

            <select
              onchange={handleBasePresetChange}
              class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
            >
              {#each APP_FONT_PRESETS as preset}
                <option
                  value={preset.value}
                  selected={selectedBasePreset === preset.value}
                  class="bg-nvidia-surface text-nvidia-text-primary"
                >
                  {preset.label}
                </option>
              {/each}
              <option value="custom" selected={selectedBasePreset === 'custom'} class="bg-nvidia-surface text-nvidia-accent font-semibold">
                + Custom Font (Type Name)...
              </option>
            </select>

            {#if selectedBasePreset === 'custom'}
              <div class="pt-1">
                <input
                  type="text"
                  bind:value={customBaseInput}
                  oninput={handleCustomBaseInput}
                  placeholder="Enter font name installed on your PC (e.g. Inter, Roboto, SF Pro)..."
                  class="w-full px-3 py-1.5 bg-nvidia-surface border border-nvidia-accent/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-nvidia-accent"
                />
              </div>
            {/if}
          </div>

          <!-- 2. Mod Listing Typography -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
              <div class="flex items-center gap-1.5">
                <FileCode2 class="h-3.5 w-3.5 text-cyan-400" />
                <span>2. Mod Listing Typography (Filenames, Ranks, Metrics)</span>
              </div>
              <span class="text-[10px] font-normal lowercase opacity-75">monospace</span>
            </div>

            <select
              onchange={handleModsPresetChange}
              class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer font-sans"
            >
              {#each MOD_FONT_PRESETS as preset}
                <option
                  value={preset.value}
                  selected={selectedModsPreset === preset.value}
                  class="bg-nvidia-surface text-nvidia-text-primary"
                >
                  {preset.label}
                </option>
              {/each}
              <option value="custom" selected={selectedModsPreset === 'custom'} class="bg-nvidia-surface text-cyan-400 font-semibold">
                + Custom Font (Type Name)...
              </option>
            </select>

            {#if selectedModsPreset === 'custom'}
              <div class="pt-1">
                <input
                  type="text"
                  bind:value={customModsInput}
                  oninput={handleCustomModsInput}
                  placeholder="Enter coding font installed on your PC (e.g. JetBrains Mono, Fira Code)..."
                  class="w-full px-3 py-1.5 bg-nvidia-surface border border-cyan-500/60 rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-cyan-400"
                />
              </div>
            {/if}
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
            class="px-3.5 py-1.5 rounded hover:bg-nvidia-surface text-xs text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
          >
            Cancel
          </button>

          <button
            type="button"
            onclick={handleApply}
            class="flex items-center gap-1.5 px-4 py-1.5 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition shadow-sm cursor-pointer"
          >
            <Check class="h-3.5 w-3.5 stroke-[2.5]" />
            <span>Apply Theme</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
