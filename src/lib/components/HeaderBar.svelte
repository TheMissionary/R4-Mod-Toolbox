<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { info, error } from '@tauri-apps/plugin-log';
  import { loadConfigFromDisk } from '$lib/theme';
  import { 
    RefreshCw, 
    CheckCircle2, 
    AlertTriangle, 
    AlertCircle,
    Play,
    X
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
  let isEngineSpinningUp = $state(false);
  let isGameRunning = $state(false);
  let isConfirmingKill = $state(false);
  let killConfirmTimer: ReturnType<typeof setTimeout> | null = null;
  let processInterval: ReturnType<typeof setInterval>;

  async function checkProcess() {
    try {
      const running = await invoke<boolean>('check_game_process_running');
      if (!running) {
        isGameRunning = false;
        isEngineSpinningUp = false;
        if (isConfirmingKill) {
          isConfirmingKill = false;
          if (killConfirmTimer) clearTimeout(killConfirmTimer);
        }
      } else {
        // If running but we aren't tracking it (e.g., launched externally)
        if (!isGameRunning && !isEngineSpinningUp && !isLaunching) {
          isGameRunning = true;
        }
      }
    } catch (err) {
      // non-fatal
    }
  }

  onMount(() => {
    checkProcess();
    processInterval = setInterval(checkProcess, 2500);
  });

  onDestroy(() => {
    if (processInterval) clearInterval(processInterval);
    if (killConfirmTimer) clearTimeout(killConfirmTimer);
  });

  async function handleLaunchGame() {
    if (!gamePath || isLaunching || isGameRunning || isEngineSpinningUp) return;
    isLaunching = true;
    info('Initiating game launch sequence...');
    try {
      const config = await loadConfigFromDisk();
      await invoke('launch_game', { gamePath, customLaunchPath: config.customLaunchPath });
      
      let attempts = 0;
      const pollInterval = setInterval(async () => {
        attempts++;
        try {
          const running = await invoke<boolean>('check_game_process_running');
          if (running) {
            clearInterval(pollInterval);
            isLaunching = false;
            isEngineSpinningUp = true;
            info('Process detected. Initiating 15s psychological engine load delay...');
            
            setTimeout(() => {
              // Only transition to Now Playing if the process didn't crash during the 15s
              if (isEngineSpinningUp) {
                isEngineSpinningUp = false;
                isGameRunning = true;
                info('Engine load delay complete. Game is now playing.');
              }
            }, 15000);
          } else if (attempts > 30) {
            clearInterval(pollInterval);
            isLaunching = false;
            error('Launch timeout: Process not detected after 30 seconds.');
          }
        } catch {
          if (attempts > 30) {
            clearInterval(pollInterval);
            isLaunching = false;
          }
        }
      }, 1000);
    } catch (err) {
      error(`Failed to launch game: ${err}`);
      isLaunching = false;
    }
  }

  async function handleKillClick() {
    if (!isConfirmingKill) {
      isConfirmingKill = true;
      if (killConfirmTimer) clearTimeout(killConfirmTimer);
      killConfirmTimer = setTimeout(() => {
        isConfirmingKill = false;
      }, 3500);
    } else {
      if (killConfirmTimer) clearTimeout(killConfirmTimer);
      isConfirmingKill = false;
      info('User confirmed force kill of Cyberpunk2077.exe');
      try {
        await invoke('kill_game_process');
        setTimeout(checkProcess, 600);
      } catch (err) {
        error(`Failed to kill game process: ${err}`);
      }
    }
  }
</script>

<header data-tauri-drag-region class="h-16 border-b border-nvidia-border bg-nvidia-bg flex items-center justify-between px-6 select-none shrink-0">
  <!-- Left: Context Title and Status -->
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

    <!-- Status Indicator -->
    <div class="flex items-center gap-2">
      {#if !gamePath}
        <AlertTriangle class="h-4 w-4 text-amber-400" />
        <span class="text-xs font-bold text-amber-400 uppercase tracking-wider">Setup Required</span>
      {:else if !isValid}
        <AlertCircle class="h-4 w-4 text-red-400" />
        <span class="text-xs font-bold text-red-400 uppercase tracking-wider">Unverified Path</span>
      {:else}
        <CheckCircle2 class="h-4 w-4 text-emerald-400" />
        <span class="text-xs font-bold text-emerald-400 uppercase tracking-wider">Cyberpunk 2077 Connected</span>
      {/if}
    </div>
  </div>

  <!-- Right: Global Actions -->
  <div class="flex items-center gap-3 shrink-0">
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

    <!-- Launch Game / Process Control -->
    {#if isGameRunning}
      <div class="flex items-stretch rounded overflow-hidden border border-nvidia-border bg-nvidia-card shadow-sm text-xs h-8">
        <div class="flex items-center justify-center gap-2 px-3 bg-nvidia-surface/90 text-nvidia-text-primary select-none">
          <div class="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse"></div>
          <span class="font-semibold tracking-wide">Now Playing</span>
        </div>
        <button
          type="button"
          onclick={handleKillClick}
          class="w-10 flex items-center justify-center border-l border-nvidia-border transition cursor-pointer {isConfirmingKill ? 'bg-red-600 hover:bg-red-500 text-white font-bold animate-pulse' : 'bg-nvidia-surface/50 hover:bg-red-500/20 text-nvidia-text-muted hover:text-red-400'}"
          title={isConfirmingKill ? 'Click again to confirm force stop' : 'Force stop (kill process)'}
        >
          {#if isConfirmingKill}
            <span class="text-[9px] uppercase font-bold tracking-tighter">Kill?</span>
          {:else}
            <X class="h-3.5 w-3.5" />
          {/if}
        </button>
      </div>
    {:else if isEngineSpinningUp || isLaunching}
      <button
        disabled
        class="flex items-center justify-center gap-2 px-4 h-8 rounded bg-amber-500/15 border border-amber-500/40 text-amber-400 font-semibold text-xs shadow-sm cursor-wait"
      >
        <RefreshCw class="h-3.5 w-3.5 animate-spin" />
        <span>{isEngineSpinningUp ? 'Loading Engine...' : 'Starting...'}</span>
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
