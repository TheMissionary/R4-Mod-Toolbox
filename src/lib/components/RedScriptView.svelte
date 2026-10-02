<script lang="ts">
  import type { RedScriptItem } from '$lib/types';
  import { Search, X, Eye, EyeOff } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';

  let {
    packages = $bindable([]),
    gamePath = '',
    onStateChanged
  }: {
    packages: RedScriptItem[];
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

  async function handleToggle(pkg: RedScriptItem) {
    if (!gamePath) return;
    const nextState = !pkg.enabled;
    try {
      await invoke('toggle_plugin_state', {
        gamePath,
        modType: 'redscript',
        modName: pkg.name,
        enable: nextState
      });
      pkg.enabled = nextState;
      packages = [...packages];
      if (onStateChanged) onStateChanged();
    } catch (err) {
      console.error(`Failed to toggle Redscript package ${pkg.name}:`, err);
    }
  }

  let filteredPackages = $derived.by(() => {
    if (!searchQuery.trim()) return packages;
    const q = searchQuery.toLowerCase().trim();
    return packages.filter(p => p.name.toLowerCase().includes(q));
  });

  let activeCount = $derived(packages.filter(p => p.enabled).length);
</script>

<div class="flex flex-col h-full select-none">
  <div class="flex items-center justify-between gap-3 mb-3 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-nvidia-text-muted" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter Redscript packages..."
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
        Active: <span class="text-nvidia-accent font-semibold">{activeCount}</span> of {packages.length}
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto space-y-1 pr-1">
    {#if filteredPackages.length === 0}
      <div class="p-8 rounded border border-nvidia-border/60 bg-nvidia-surface/30 text-center text-xs font-mono text-nvidia-text-muted">
        No Redscript packages detected.
      </div>
    {:else}
      {#each filteredPackages as pkg (pkg.name)}
        <div
          data-mod-item="true"
          data-mod-name={pkg.name}
          data-mod-path={pkg.path}
          data-mod-type="redscript"
          data-is-file="false"
          class="flex items-center justify-between px-3 rounded border border-nvidia-border/70 bg-nvidia-surface/40 hover:bg-nvidia-surface/70 transition density-row
            {pkg.enabled ? 'text-nvidia-text-primary' : 'opacity-50 text-nvidia-text-muted'}"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <button
              type="button"
              onclick={() => handleToggle(pkg)}
              class="p-1 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
              title={pkg.enabled ? 'Click to disable' : 'Click to enable'}
            >
              {#if pkg.enabled}
                <Eye class="h-3.5 w-3.5 text-nvidia-accent" />
              {:else}
                <EyeOff class="h-3.5 w-3.5 text-nvidia-text-muted" />
              {/if}
            </button>
            <span class="font-mono truncate {pkg.enabled ? 'text-nvidia-text-primary' : 'line-through text-nvidia-text-muted'}">
              {pkg.name}
            </span>
          </div>

          <div class="flex items-center gap-3 text-[11px] font-mono text-nvidia-text-muted shrink-0">
            <span class="px-1.5 py-0.2 rounded bg-nvidia-surface border border-nvidia-border/60 text-[10px] text-nvidia-text-muted">
              {pkg.reds_count} .reds
            </span>
            <span>{formatBytes(pkg.size_bytes)}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
