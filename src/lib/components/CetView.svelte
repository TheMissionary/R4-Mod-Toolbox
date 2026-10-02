<script lang="ts">
  import type { CetPluginItem } from '$lib/types';
  import { Search, X, Eye, EyeOff } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';

  let {
    plugins = $bindable([]),
    gamePath = '',
    onStateChanged
  }: {
    plugins: CetPluginItem[];
    gamePath: string;
    onStateChanged?: () => void;
  } = $props();

  let searchQuery = $state('');

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  async function handleToggle(plugin: CetPluginItem) {
    if (!gamePath) return;
    const nextState = !plugin.enabled;
    try {
      await invoke('toggle_plugin_state', {
        gamePath,
        modType: 'cet',
        modName: plugin.name,
        enable: nextState
      });
      plugin.enabled = nextState;
      plugins = [...plugins];
      if (onStateChanged) onStateChanged();
    } catch (err) {
      console.error(`Failed to toggle CET plugin ${plugin.name}:`, err);
    }
  }

  let filteredPlugins = $derived.by(() => {
    if (!searchQuery.trim()) return plugins;
    const q = searchQuery.toLowerCase().trim();
    return plugins.filter(p => p.name.toLowerCase().includes(q));
  });

  let activeCount = $derived(plugins.filter(p => p.enabled).length);
</script>

<div class="flex flex-col h-full select-none">
  <div class="flex items-center justify-between gap-3 mb-3 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-nvidia-text-muted" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter CET plugins..."
          class="w-full pl-9 pr-8 py-1.5 rounded border border-nvidia-border bg-nvidia-surface/80 text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-hidden focus:border-nvidia-accent font-sans"
        />
        {#if searchQuery !== ''}
          <button
            type="button"
            onclick={() => searchQuery = ''}
            class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
            title="Clear search"
          >
            <X class="h-3.5 w-3.5" />
          </button>
        {/if}
      </div>
      <div class="text-xs font-mono text-nvidia-text-muted shrink-0">
        Active: <span class="text-nvidia-accent font-semibold">{activeCount}</span> of {plugins.length}
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto space-y-1 pr-1">
    {#if filteredPlugins.length === 0}
      <div class="p-8 rounded border border-nvidia-border/60 bg-nvidia-surface/30 text-center text-xs font-mono text-nvidia-text-muted">
        No CET plugins detected.
      </div>
    {:else}
      {#each filteredPlugins as plugin (plugin.name)}
        <div
          data-mod-item="true"
          data-mod-name={plugin.name}
          data-mod-path={plugin.path}
          data-mod-type="cet"
          data-is-file="false"
          class="flex items-center justify-between px-3 rounded border border-nvidia-border/70 bg-nvidia-surface/40 hover:bg-nvidia-surface/70 transition density-row
            {plugin.enabled ? 'text-nvidia-text-primary' : 'opacity-50 text-nvidia-text-muted'}"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <button
              type="button"
              onclick={() => handleToggle(plugin)}
              class="p-1 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
              title={plugin.enabled ? 'Click to disable' : 'Click to enable'}
            >
              {#if plugin.enabled}
                <Eye class="h-3.5 w-3.5 text-nvidia-accent" />
              {:else}
                <EyeOff class="h-3.5 w-3.5 text-nvidia-text-muted" />
              {/if}
            </button>
            <span class="font-mono truncate {plugin.enabled ? 'text-nvidia-text-primary' : 'line-through text-nvidia-text-muted'}">
              {plugin.name}
            </span>
          </div>

          <div class="flex items-center gap-3 text-[11px] font-mono text-nvidia-text-muted shrink-0">
            {#if plugin.has_init}
              <span class="px-1.5 py-0.2 rounded bg-nvidia-surface border border-nvidia-border/60 text-[10px] text-nvidia-accent">
                init.lua
              </span>
            {/if}
            <span>{formatBytes(plugin.size_bytes)}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
