<script lang="ts">
  import { untrack } from 'svelte';
  import {
    DEFAULT_DARK_THEME,
    DEFAULT_LIGHT_THEME,
    THEME_COLOR_META,
    FONT_PRESETS,
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
    Type
  } from 'lucide-svelte';

  let {
    isOpen = $bindable(false)
  }: {
    isOpen: boolean;
  } = $props();

  // Local sandboxed draft settings
  let draft = $state<ThemeSettings>(loadThemeSettings());
  let activeEditingKey = $state<keyof AppThemeColors>('accent');
  let manualHex = $state('#76B900');
  let customFontText = $state('');

  // Active palette for the draft mode (Dark or Light)
  let activeColors = $derived(draft.mode === 'light' ? draft.lightColors : draft.darkColors);

  // Isolate initialization so mutating draft NEVER re-triggers this effect
  $effect(() => {
    if (isOpen) {
      untrack(() => {
        draft = loadThemeSettings();
        activeEditingKey = 'accent';
        const currentVal = (draft.mode === 'light' ? draft.lightColors : draft.darkColors)['accent'];
        manualHex = currentVal.toUpperCase();
        customFontText = draft.fontFamily;
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

  function handleFontPresetChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    draft.fontFamily = val;
    customFontText = val;
  }

  function handleCustomFontInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    customFontText = val;
    draft.fontFamily = val || FONT_PRESETS[0].value;
  }

  function handleResetDefault() {
    if (draft.mode === 'light') {
      draft.lightColors = { ...DEFAULT_LIGHT_THEME };
    } else {
      draft.darkColors = { ...DEFAULT_DARK_THEME };
    }
    draft.fontFamily = FONT_PRESETS[0].value;
    customFontText = FONT_PRESETS[0].value;
    draft.isCompact = false;
    manualHex = (draft.mode === 'light' ? DEFAULT_LIGHT_THEME : DEFAULT_DARK_THEME)[activeEditingKey].toUpperCase();
  }

  async function handleApply() {
    await applyAndPersistTheme(draft);
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
        <!-- THE INTERACTIVE SANDBOXED CANVAS (Live Mockup)                     -->
        <!-- ----------------------------------------------------------------- -->
        <div class="space-y-1">
          <div class="flex items-center justify-between text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
            <span>Live Staging Sandbox (Preview Only)</span>
            <span class="text-[10px] lowercase font-normal opacity-70">updates in real-time</span>
          </div>

          <div
            class="rounded-lg border shadow-inner p-3.5 space-y-3 transition-colors duration-200"
            style="
              background-color: {activeColors.bg};
              border-color: {activeColors.border};
              font-family: {draft.fontFamily};
              color: {activeColors.textPrimary};
            "
          >
            <!-- Mockup Mini Header -->
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
              <span class="text-[10px] font-mono px-2 py-0.5 rounded border" style="background-color: {activeColors.card}; border-color: {activeColors.border}; color: {activeColors.textMuted};">
                G:\Cyberpunk 2077
              </span>
            </div>

            <!-- Mockup Mini Nav & Mod Row -->
            <div class="flex gap-2.5 items-start">
              <!-- Mini Sidebar Nav -->
              <div class="w-28 space-y-1 shrink-0">
                <div
                  class="px-2 py-1 rounded text-[11px] font-bold flex items-center justify-between"
                  style="background-color: {activeColors.card}; color: {activeColors.accent}; border: 1px solid {activeColors.border};"
                >
                  <span>Archive</span>
                  <span class="text-[9px] font-mono opacity-80">45</span>
                </div>
                <div
                  class="px-2 py-1 rounded text-[11px] opacity-75"
                  style="color: {activeColors.textMuted};"
                >
                  <span>CET Mods</span>
                </div>
              </div>

              <!-- Mini Mod Row (Responds to Compact Density!) -->
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
                  <span class="text-xs font-mono font-medium truncate" style="color: {activeColors.textPrimary};">
                    EquipmentEx.archive
                  </span>
                  <span class="text-[9px] font-mono font-bold px-1 rounded border shrink-0" style="background-color: {activeColors.surface}; color: {activeColors.accent}; border-color: {activeColors.accent};">
                    XL
                  </span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <span class="text-[10px] font-mono shrink-0" style="color: {activeColors.textMuted};">
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
        <!-- TYPOGRAPHY / FONT SELECTOR                                        -->
        <!-- ----------------------------------------------------------------- -->
        <div class="space-y-1.5 pt-1">
          <div class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-nvidia-text-muted">
            <Type class="h-3.5 w-3.5" />
            <span>Installed System Typography</span>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
            <!-- Preset Selector -->
            <div>
              <select
                onchange={handleFontPresetChange}
                class="w-full px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent cursor-pointer"
              >
                {#each FONT_PRESETS as preset}
                  <option
                    value={preset.value}
                    selected={draft.fontFamily === preset.value}
                    class="bg-nvidia-surface text-nvidia-text-primary"
                  >
                    {preset.label}
                  </option>
                {/each}
              </select>
            </div>

            <!-- Custom Font Name Input -->
            <div>
              <input
                type="text"
                bind:value={customFontText}
                oninput={handleCustomFontInput}
                placeholder="Or type any Windows font (e.g. Cascadia Code)"
                class="w-full px-3 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/70 focus:outline-none focus:border-nvidia-accent"
              />
            </div>
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
