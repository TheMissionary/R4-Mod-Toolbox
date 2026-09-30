<script lang="ts">
  import type { CetPluginItem } from '$lib/types';
  import { Search, Eye, EyeOff } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';

  let {
    plugins = $bindable([]),
    gamePath = ''
  }: {
    plugins: CetPluginItem[];
    gamePath: string;
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
    <div class="relative flex-1 max-w-md">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-nvidia-text-muted" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Filter CET plugins..."
        class="w-full pl-9 pr-3 py-1.5 rounded border border-nvidia-border bg-nvidia-surface/80 text-xs text-white placeholder:text-nvidia-text-muted/60 focus:outline-hidden focus:border-nvidia-accent font-sans"
      />
    </div>
    <div class="text-xs font-mono text-nvidia-text-muted">
      Active: <span class="text-nvidia-accent font-semibold">{activeCount}</span> / {plugins.length}
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
          class="flex items-center justify-between px-3 py-2 rounded border border-nvidia-border/70 bg-nvidia-surface/40 hover:bg-nvidia-surface/70 transition
            {plugin.enabled ? 'text-gray-200' : 'opacity-50 text-nvidia-text-muted'}"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <button
              type="button"
              onclick={() => handleToggle(plugin)}
              class="p-1 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-white transition cursor-pointer"
              title={plugin.enabled ? 'Click to disable' : 'Click to enable'}
            >
              {#if plugin.enabled}
                <Eye class="h-3.5 w-3.5 text-nvidia-accent" />
              {:else}
                <EyeOff class="h-3.5 w-3.5 text-nvidia-text-muted" />
              {/if}
            </button>
            <span class="text-xs font-mono truncate {plugin.enabled ? 'text-white' : 'line-through text-nvidia-text-muted'}">
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