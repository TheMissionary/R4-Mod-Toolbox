<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { info, error } from '@tauri-apps/plugin-log';
  import { ChevronRight, Play } from 'lucide-svelte';
  import { loadThemeSettings } from '$lib/theme';

  let { 
    gamePath = '', 
    currentTab = 'archive',
  }: { 
    gamePath: string; 
    currentTab?: string;
  } = $props();

  let isValid = $derived(gamePath.length > 0);
  let isLaunching = $state(false);
  let heroBannerUrl = $state<string | null>(null);

  function updateBanner() {
    const settings = loadThemeSettings();
    if (settings.heroBannerPath) {
      heroBannerUrl = convertFileSrc(settings.heroBannerPath);
    } else {
      heroBannerUrl = null;
    }
  }

  onMount(() => {
    updateBanner();
    window.addEventListener('theme-updated', updateBanner);
    return () => window.removeEventListener('theme-updated', updateBanner);
  });

  async function handleLaunchGame() {
    if (!gamePath || isLaunching) return;
    isLaunching = true;
    info('Initiating game launch sequence...');
    try {
      await invoke('launch_game', { gamePath });
      setTimeout(() => {
        isLaunching = false;
        info('Launch command sent successfully.');
      }, 2500);
    } catch (err) {
      error(`Failed to launch game: ${err}`);
      isLaunching = false;
    }
  }

  let breadcrumbs = $derived.by(() => {
    const base = ['Cyberpunk 2077'];
    switch (currentTab) {
      case 'archive': return [...base, 'archive', 'pc', 'mod'];
      case 'cet': return [...base, 'bin', 'x64', 'plugins', 'cyber_engine_tweaks', 'mods'];
      case 'red4ext': return [...base, 'red4ext', 'plugins'];
      case 'redscript': return [...base, 'r6', 'scripts'];
      case 'r6tweaks': return [...base, 'r6', 'tweaks'];
      case 'history': return [...base, 'Mod History & Ledger'];
      case 'settings': return [...base, 'Application Settings'];
      case 'help': return [...base, 'Help & Documentation'];
      case 'about': return [...base, 'About R4'];
      default: return base;
    }
  });
</script>

<header data-tauri-drag-region class="relative h-14 border-b flex items-center justify-between px-6 select-none shrink-0 overflow-hidden" style="background-color: var(--theme-bg); border-color: var(--theme-border);">
  
  <!-- Hero Banner Background Layer -->
  {#if heroBannerUrl}
    <div class="absolute inset-0 z-0 pointer-events-none">
      <img 
        src={heroBannerUrl} 
        alt="" 
        onerror={(e) => e.currentTarget.style.display = 'none'}
        class="absolute inset-0 w-full h-full object-cover object-[center_20%] opacity-60" 
      />
      <!-- Gradient Mask: 60% dark at top to 100% solid theme-bg at bottom -->
      <div class="absolute inset-0" style="background: linear-gradient(to bottom, color-mix(in srgb, var(--theme-bg) 60%, transparent) 0%, var(--theme-bg) 100%);"></div>
    </div>
  {/if}

  <!-- Left: Breadcrumbs -->
  <div class="relative z-10 flex items-center gap-2 pointer-events-none">
    {#each breadcrumbs as crumb, i}
      <span class="text-xs font-mono font-semibold" style="color: {i === breadcrumbs.length - 1 ? 'var(--theme-text-primary)' : 'var(--theme-text-muted)'};">
        {crumb}
      </span>
      {#if i < breadcrumbs.length - 1}
        <ChevronRight class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
      {/if}
    {/each}
  </div>

  <!-- Right: Launch Game -->
  <div class="relative z-10 flex items-center gap-4 shrink-0">
    <button
      type="button"
      onclick={handleLaunchGame}
      disabled={!isValid || isLaunching}
      class="flex items-center justify-center gap-2 px-5 h-8 rounded font-bold text-xs transition shadow-sm disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer hover:brightness-110"
      style="background-color: var(--theme-accent); color: var(--theme-accent-contrast);"
      title={!isValid ? 'Configure a valid game directory to enable launch' : 'Launch Cyberpunk 2077'}
    >
      <Play class="h-3.5 w-3.5" style="fill: var(--theme-accent-contrast); color: var(--theme-accent-contrast);" />
      <span>{isLaunching ? 'Starting...' : 'Launch Game'}</span>
    </button>
  </div>
</header>