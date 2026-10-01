<script lang="ts">
  import { RefreshCw, FolderSearch } from 'lucide-svelte';
  import { open } from '@tauri-apps/plugin-dialog';

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
        title: 'Select Cyberpunk 2077 Root Folder'
      });
      
      if (typeof selected === 'string') {
        gamePath = selected;
        if (onRefresh) onRefresh();
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
  <div class="flex items-center gap-3">
    <span class="text-xs font-semibold uppercase tracking-wider text-nvidia-text-muted">Target Path:</span>
    <span class="text-xs font-mono px-2 py-0.5 rounded bg-nvidia-card border border-nvidia-border text-gray-200 max-w-sm truncate">
      {gamePath || 'No folder selected'}
    </span>
  </div>

  <div class="flex items-center gap-3">
    <button 
      onclick={chooseDirectory}
      class="flex items-center gap-2 px-3 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-card/80 border border-nvidia-border text-xs text-gray-200 transition cursor-pointer"
    >
      <FolderSearch class="h-3.5 w-3.5 text-nvidia-text-muted" />
      <span>Verify Game Path</span>
    </button>

    <button 
      onclick={handleHardRefresh}
      disabled={!gamePath || isLoading}
      class="flex items-center gap-2 px-3.5 py-1.5 rounded bg-nvidia-accent hover:bg-nvidia-accent-hover text-black font-semibold text-xs transition disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
    >
      <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
      <span>{isLoading ? 'Analyzing...' : 'Sync & Rescan'}</span>
    </button>
  </div>
</header>
