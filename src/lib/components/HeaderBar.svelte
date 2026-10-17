<script lang="ts">
  import { 
    RefreshCw, 
    FolderSearch, 
    FolderOpen, 
    Copy, 
    Check, 
    ChevronDown, 
    CheckCircle2, 
    AlertTriangle, 
    AlertCircle 
  } from 'lucide-svelte';
  import { open, message } from '@tauri-apps/plugin-dialog';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { invoke } from '@tauri-apps/api/core';
  import { loadConfigFromDisk, saveConfigToDisk } from '$lib/theme';
  import type { ScanResult } from '$lib/types';

  let { 
    gamePath = $bindable(''), 
    isLoading = false,
    scanResult = null,
    onRefresh 
  }: { 
    gamePath: string; 
    isLoading: boolean;
    scanResult?: ScanResult | null;
    onRefresh: () => void;
  } = $props();

  let isPopoverOpen = $state(false);
  let copiedFeedback = $state(false);
  let chipContainerEl = $state<HTMLDivElement | null>(null);

  let isValid = $derived(scanResult ? scanResult.is_valid_game_path : (gamePath.length > 0));

  let folderDisplayName = $derived.by(() => {
    if (!gamePath) return 'No Game Folder Configured';
    const cleaned = gamePath.replace(/[\\/]+$/, '');
    const parts = cleaned.split(/[\\/]/);
    return parts[parts.length - 1] || cleaned;
  });

  async function chooseDirectory() {
    isPopoverOpen = false;
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

  async function handleCopyPath() {
    if (!gamePath) return;
    try {
      await navigator.clipboard.writeText(gamePath);
      copiedFeedback = true;
      setTimeout(() => { copiedFeedback = false; }, 1500);
    } catch (err) {
      console.error('Failed to copy path:', err);
    }
  }

  async function handleOpenFolder() {
    if (!gamePath) return;
    try {
      await openPath(gamePath);
    } catch (err) {
      console.error('Failed to open game path in Explorer:', err);
    }
  }

  function handleHardRefresh() {
    window.location.reload();
  }

  function handleWindowClick(e: MouseEvent) {
    if (!isPopoverOpen) return;
    const target = e.target as Node | null;
    if (chipContainerEl && target && !chipContainerEl.contains(target)) {
      isPopoverOpen = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<header data-tauri-drag-region class="h-16 border-b border-nvidia-border bg-nvidia-bg flex items-center justify-between px-8 select-none">
  <!-- Left: Compact Status Chip & Dropdown Popover -->
  <div class="relative" bind:this={chipContainerEl}>
    <button
      type="button"
      onclick={() => isPopoverOpen = !isPopoverOpen}
      class="flex items-center gap-2.5 px-3 py-1.5 rounded-md border text-xs transition cursor-pointer shadow-xs {isPopoverOpen ? 'bg-nvidia-card border-nvidia-accent/60' : 'bg-nvidia-surface/80 hover:bg-nvidia-card border-nvidia-border hover:border-nvidia-border/90'}"
      title={gamePath ? `Target: ${gamePath} (Click for details)` : 'Click to configure game folder'}
    >
      <!-- Status Dot -->
      <div class="flex items-center justify-center">
        {#if !gamePath}
          <div class="h-2 w-2 rounded-full bg-amber-400"></div>
        {:else if !isValid}
          <div class="h-2 w-2 rounded-full bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.7)]"></div>
        {:else}
          <div class="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.7)]"></div>
        {/if}
      </div>

      <!-- Main Label -->
      <span class="font-semibold text-xs {isValid && gamePath ? 'text-nvidia-text-primary' : (gamePath ? 'text-red-400' : 'text-nvidia-text-muted')}">
        {folderDisplayName}
      </span>

      <!-- Status Pill -->
      {#if !gamePath}
        <span class="text-[10px] font-mono px-1.5 py-0.2 rounded border bg-amber-500/15 border-amber-500/30 text-amber-400 font-semibold">
          Setup
        </span>
      {:else if !isValid}
        <span class="text-[10px] font-mono px-1.5 py-0.2 rounded border bg-red-500/15 border-red-500/30 text-red-400 font-semibold">
          Unverified
        </span>
      {:else}
        <span class="text-[10px] font-mono px-1.5 py-0.2 rounded border bg-emerald-500/15 border-emerald-500/30 text-emerald-400 font-semibold">
          Connected
        </span>
      {/if}

      <ChevronDown class="h-3 w-3 text-nvidia-text-muted transition-transform duration-200 {isPopoverOpen ? 'rotate-180 text-nvidia-accent' : ''}" />
    </button>

    <!-- Dropdown Popover -->
    {#if isPopoverOpen}
      <div class="absolute left-0 top-full mt-2 w-[420px] rounded-lg bg-nvidia-card border border-nvidia-border shadow-2xl p-4 z-50 text-xs backdrop-blur-md">
        <!-- Popover Title Row -->
        <div class="flex items-center justify-between pb-2.5 mb-2.5 border-b border-nvidia-border/60">
          <div class="flex items-center gap-2">
            {#if !gamePath}
              <AlertTriangle class="h-4 w-4 text-amber-400" />
              <span class="font-bold text-nvidia-text-primary uppercase tracking-wider text-[11px]">No Game Path Configured</span>
            {:else if !isValid}
              <AlertCircle class="h-4 w-4 text-red-400" />
              <span class="font-bold text-red-400 uppercase tracking-wider text-[11px]">Unverified Game Directory</span>
            {:else}
              <CheckCircle2 class="h-4 w-4 text-emerald-400" />
              <span class="font-bold text-nvidia-text-primary uppercase tracking-wider text-[11px]">Cyberpunk 2077 Target</span>
            {/if}
          </div>
          <span class="text-[10px] font-mono text-nvidia-text-muted">
            {isValid ? 'Ready' : 'Attention'}
          </span>
        </div>

        <!-- Full Path Display -->
        <div class="mb-3">
          <div class="text-[10px] uppercase tracking-wider text-nvidia-text-muted font-semibold mb-1">
            Full Game Path
          </div>
          <div class="p-2.5 rounded bg-nvidia-bg border border-nvidia-border font-mono text-[11px] text-nvidia-text-primary break-all select-all">
            {gamePath || 'No folder selected'}
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-between pt-1 gap-2">
          <div class="flex items-center gap-1.5">
            <button
              type="button"
              onclick={handleCopyPath}
              disabled={!gamePath}
              class="flex items-center gap-1.5 px-2.5 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shadow-xs"
              title="Copy full path to clipboard"
            >
              {#if copiedFeedback}
                <Check class="h-3.5 w-3.5 text-nvidia-accent" />
                <span class="text-nvidia-accent font-medium">Copied!</span>
              {:else}
                <Copy class="h-3.5 w-3.5 text-nvidia-text-muted" />
                <span>Copy</span>
              {/if}
            </button>

            <button
              type="button"
              onclick={handleOpenFolder}
              disabled={!gamePath}
              class="flex items-center gap-1.5 px-2.5 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shadow-xs"
              title="Open folder in File Explorer"
            >
              <FolderOpen class="h-3.5 w-3.5 text-nvidia-text-muted" />
              <span>Explorer</span>
            </button>
          </div>

          <button
            type="button"
            onclick={chooseDirectory}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs font-semibold text-nvidia-text-primary transition cursor-pointer shadow-xs"
          >
            <FolderSearch class="h-3.5 w-3.5 text-nvidia-text-muted" />
            <span>Change Folder...</span>
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- Right Actions Container -->
  <div class="flex items-center gap-2.5 shrink-0">
    <button 
      type="button"
      onclick={chooseDirectory}
      class="flex items-center gap-2 px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0 whitespace-nowrap shadow-xs"
      title="Select or change game installation directory"
    >
      <FolderSearch class="h-3.5 w-3.5 text-nvidia-text-muted" />
      <span>Change Folder</span>
    </button>

    <button 
      type="button"
      onclick={handleHardRefresh}
      disabled={!gamePath || isLoading}
      class="flex items-center gap-2 px-3.5 py-1.5 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shrink-0 whitespace-nowrap shadow-xs"
    >
      <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
      <span>{isLoading ? 'Analyzing...' : 'Sync & Rescan'}</span>
    </button>
  </div>
</header>
