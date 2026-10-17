<script lang="ts">
  import { RefreshCw, FolderSearch } from 'lucide-svelte';
  import { open, message } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { loadConfigFromDisk, saveConfigToDisk } from '$lib/theme';

  let { 
    gamePath = $bindable(''), 
    isLoading = false,
    onRefresh 
  }: { 
    gamePath: string; 
    isLoading: boolean;
    onRefresh: () => void;
  } = $props();

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

  function handleHardRefresh() {
    window.location.reload();
  }
</script>

<header data-tauri-drag-region class="h-16 border-b border-nvidia-border bg-nvidia-bg flex items-center justify-between px-8 select-none">
  <!-- Left Target Path Container (Truncates smoothly on narrow windows) -->
  <div class="flex items-center gap-2.5 min-w-0 flex-1 mr-4">
    <span class="text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted shrink-0">Target Path:</span>
    <span class="text-xs font-mono px-2.5 py-1 rounded bg-nvidia-card border border-nvidia-border text-nvidia-text-primary truncate max-w-xl" title={gamePath || 'No folder selected'}>
      {gamePath || 'No folder selected'}
    </span>
  </div>

  <!-- Right Actions Container (shrink-0 & whitespace-nowrap prevents button squishing) -->
  <div class="flex items-center gap-2.5 shrink-0">
    <button 
      type="button"
      onclick={chooseDirectory}
      class="flex items-center gap-2 px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-xs text-nvidia-text-primary transition cursor-pointer shrink-0 whitespace-nowrap shadow-xs"
    >
      <FolderSearch class="h-3.5 w-3.5 text-nvidia-text-muted" />
      <span>Verify Game Path</span>
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
