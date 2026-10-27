<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { info, error } from '@tauri-apps/plugin-log';
  import { 
    RefreshCw, 
    CheckCircle2, 
    AlertTriangle, 
    AlertCircle,
    Play
  } from 'lucide-svelte';
  import type { ScanResult } from '$lib/types';

  let { 
    gamePath = '', 
    isLoading = false,
    scanResult = null,
    currentTab = 'archive',
    isSidebarCollapsed = false,
    onRefresh 
  }: { 
    gamePath: string; 
    isLoading: boolean;
    scanResult?: ScanResult | null;
    currentTab?: string;
    isSidebarCollapsed?: boolean;
    onRefresh: () => void;
  } = $props();

  let isValid = $derived(scanResult ? scanResult.is_valid_game_path : (gamePath.length > 0));

  // Process & Launch State
  let isLaunching = $state(false);

  async function handleLaunchGame() {
    if (!gamePath || isLaunching) return;
    isLaunching = true;
    info('Initiating game launch sequence...');
    try {
      await invoke('launch_game', { gamePath });
      
      // Fire and forget: show loading state briefly so the user knows it clicked
      setTimeout(() => {
        isLaunching = false;
        info('Launch command sent successfully.');
      }, 2500);
    } catch (err) {
      error(`Failed to launch game: ${err}`);
      isLaunching = false;
    }
  }
</script>

<header data-tauri-drag-region class="h-16 border-b border-nvidia-border bg-nvidia-bg flex items-center justify-between px-6 select-none shrink-0">
  <!-- Left: Context Title -->
  <div class="flex items-center gap-6 pointer-events-none">
    {#if isSidebarCollapsed}
      <!-- Static App Title -->
      <div class="flex items-center gap-3">
        <h1 class="text-sm font-bold tracking-wider uppercase text-nvidia-text-primary leading-tight">
          R4 MOD TOOLBOX
        </h1>
      </div>
      <!-- Vertical Divider -->
      <div class="h-8 w-px bg-nvidia-border/60"></div>
    {/if}
  </div>

  <!-- Right: Global Actions & Status -->
  <div class="flex items-center gap-4 shrink-0">
    
    <!-- Status Indicator -->
    <div class="flex items-center gap-2 mr-2">
      {#if !gamePath}
        <AlertTriangle class="h-4 w-4 text-amber-400" />
        <span class="text-xs font-bold text-amber-400 uppercase tracking-wider">Setup Required</span>
      {:else if !isValid}
        <AlertCircle class="h-4 w-4 text-red-500" />
        <span class="text-xs font-bold text-red-500 uppercase tracking-wider">Disconnected</span>
      {:else}
        <CheckCircle2 class="h-4 w-4 text-nvidia-accent" />
        <span class="text-xs font-bold text-nvidia-accent uppercase tracking-wider">Connected</span>
      {/if}
    </div>

    <button 
      type="button"
      onclick={onRefresh}
      disabled={!gamePath || isLoading}
      class="flex items-center gap-2 px-4 h-8 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-nvidia-text-primary font-semibold text-xs transition disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shadow-xs"
    >
      <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin text-nvidia-accent' : 'text-nvidia-text-muted'}" />
      <span>{isLoading ? 'Analyzing...' : 'Sync & Rescan'}</span>
    </button>

    <div class="w-px h-6 bg-nvidia-border/60 mx-1"></div>

    <!-- Launch Game -->
    {#if isLaunching}
      <button
        disabled
        class="flex items-center justify-center gap-2 px-4 h-8 rounded bg-amber-500/15 border border-amber-500/40 text-amber-400 font-semibold text-xs shadow-sm cursor-wait"
      >
        <RefreshCw class="h-3.5 w-3.5 animate-spin" />
        <span>Starting...</span>
      </button>
    {:else}
      <button
        type="button"
        onclick={handleLaunchGame}
        disabled={!isValid}
        class="flex items-center justify-center gap-2 px-4 h-8 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition shadow-sm disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
        title={!isValid ? 'Configure a valid game directory to enable launch' : 'Launch Cyberpunk 2077'}
      >
        <Play class="h-3.5 w-3.5 fill-black" />
        <span>Launch Game</span>
      </button>
    {/if}
  </div>
</header>
