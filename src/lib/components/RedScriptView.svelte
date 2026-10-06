<script lang="ts">
  import type { RedScriptItem } from '$lib/types';
  import { Search, X, FolderSearch, Power, Copy, Check, Folder, FileCode } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick } from 'svelte';

  let {
    packages = $bindable([]),
    gamePath = '',
    onStateChanged,
    targetHighlightMod = null
  }: {
    packages: RedScriptItem[];
    gamePath: string;
    onStateChanged?: () => void;
    targetHighlightMod?: string | null;
  } = $props();

  let searchQuery = $state('');
  let copiedFeedback = $state(false);

  let highlightedModName = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;
  const modNodeMap = new Map<string, HTMLElement>();

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    pkg: RedScriptItem | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    pkg: null
  });

  function registerModNode(node: HTMLElement, modName: string) {
    modNodeMap.set(modName, node);
    return {
      update(newName: string) {
        if (newName !== modName) {
          modNodeMap.delete(modName);
          modName = newName;
          modNodeMap.set(modName, node);
        }
      },
      destroy() {
        modNodeMap.delete(modName);
      }
    };
  }

  async function focusModInMainList(modName: string) {
    if (searchQuery !== '') {
      searchQuery = '';
    }

    await tick();

    let targetElement = modNodeMap.get(modName);

    if (!targetElement) {
      await new Promise(r => setTimeout(r, 60));
      targetElement = modNodeMap.get(modName);
    }

    if (targetElement) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });

      highlightedModName = modName;
      if (highlightTimeoutId) clearTimeout(highlightTimeoutId);
      highlightTimeoutId = window.setTimeout(() => {
        highlightedModName = null;
      }, 2500);
    }
  }

  $effect(() => {
    if (targetHighlightMod) {
      focusModInMainList(targetHighlightMod);
    }
  });

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

  function openContextMenu(event: MouseEvent, pkg: RedScriptItem) {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    const menuWidth = 230;
    const menuHeight = 145;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    contextMenu = {
      visible: true,
      x: posX,
      y: posY,
      pkg
    };
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.pkg = null;
      copiedFeedback = false;
    }
  }

  function handleContextMenuToggle() {
    if (contextMenu.pkg) {
      handleToggle(contextMenu.pkg);
    }
    closeContextMenu();
  }

  async function handleContextMenuShowInExplorer() {
    if (!contextMenu.pkg) return;
    try {
      await revealItemInDir(contextMenu.pkg.path);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeContextMenu();
  }

  async function handleContextMenuCopyName() {
    if (!contextMenu.pkg) return;
    try {
      await navigator.clipboard.writeText(contextMenu.pkg.name);
      copiedFeedback = true;
      setTimeout(() => {
        closeContextMenu();
      }, 400);
    } catch (err) {
      console.error('Failed to copy package name:', err);
      closeContextMenu();
    }
  }

  let filteredPackages = $derived.by(() => {
    if (!searchQuery.trim()) return packages;
    const q = searchQuery.toLowerCase().trim();
    return packages.filter(p => p.name.toLowerCase().includes(q));
  });

  let activeCount = $derived(packages.filter(p => p.enabled).length);
</script>

<svelte:window onclick={closeContextMenu} />

<!-- Custom Context Menu -->
{#if contextMenu.visible && contextMenu.pkg}
  <div
    class="fixed z-50 w-56 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center justify-between gap-2">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.pkg.is_dir}
          <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
        {:else}
          <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
        {/if}
        <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={contextMenu.pkg.name}>
          {contextMenu.pkg.name}
        </span>
      </div>
      <span class="text-[9px] font-mono uppercase px-1 py-0.2 rounded border {contextMenu.pkg.enabled ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-red-500/15 border-red-500/40 text-red-400'} shrink-0">
        {contextMenu.pkg.enabled ? 'Active' : 'Disabled'}
      </span>
    </div>

    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleContextMenuShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <FolderSearch class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Explorer</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuToggle}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <Power class="h-3.5 w-3.5 {contextMenu.pkg.enabled ? 'text-amber-400' : 'text-nvidia-accent'}" />
        <span>{contextMenu.pkg.enabled ? 'Disable Mod' : 'Enable Mod'}</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuCopyName}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <div class="flex items-center gap-2.5">
          {#if copiedFeedback}
            <Check class="h-3.5 w-3.5 text-nvidia-accent" />
            <span class="text-nvidia-accent font-medium">Copied!</span>
          {:else}
            <Copy class="h-3.5 w-3.5 text-nvidia-text-muted group-hover:text-nvidia-text-primary" />
            <span>Copy Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="flex flex-col h-full select-none" oncontextmenu={(e) => e.preventDefault()}>
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
        {@const isHighlighted = highlightedModName === pkg.name}
        <div
          use:registerModNode={pkg.name}
          data-mod-item="true"
          data-mod-name={pkg.name}
          data-mod-path={pkg.path}
          data-mod-type="redscript"
          data-is-file={!pkg.is_dir}
          oncontextmenu={(e) => openContextMenu(e, pkg)}
          class="flex items-center justify-between px-3 rounded border transition-all duration-300 density-row
            {isHighlighted ? 'border-[#76b900] ring-2 ring-[#76b900] bg-[#76b900]/20 shadow-[0_0_15px_rgba(118,185,0,0.35)] scale-[1.008] z-10 relative' : 'border-nvidia-border/70 bg-nvidia-surface/40 hover:bg-nvidia-surface/70'}
            {pkg.enabled ? 'text-nvidia-text-primary' : 'opacity-50 text-nvidia-text-muted'}"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <!-- Subtle Calmed Switch -->
            <button
              type="button"
              onclick={() => handleToggle(pkg)}
              aria-label={pkg.enabled ? "Disable package " + pkg.name : "Enable package " + pkg.name}
              class="w-7 h-4 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer {pkg.enabled ? 'bg-zinc-700/80 border border-zinc-600' : 'bg-zinc-900/90 border border-zinc-800'}"
            >
              <div class="h-2.5 w-2.5 rounded-full transition-transform transform {pkg.enabled ? 'translate-x-3 bg-zinc-100 shadow-xs' : 'translate-x-0 bg-zinc-500'}"></div>
            </button>

            <!-- Folder vs Loose File Icon -->
            {#if pkg.is_dir}
              <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
            {:else}
              <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
            {/if}

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
