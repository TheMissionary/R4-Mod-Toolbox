<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { loadConfigFromDisk, saveConfigToDisk } from '$lib/theme';
  import type { LedgerEntry } from '$lib/types';
  import {
    Clock,
    FolderSearch,
    ExternalLink,
    Copy,
    Check,
    CalendarDays
  } from 'lucide-svelte';

  let {
    gamePath = '',
    showNewMods = $bindable(false),
    newModsHours = $bindable(6),
    onNavigateToMod
  }: {
    gamePath: string;
    showNewMods: boolean;
    newModsHours: number;
    onNavigateToMod: (tab: string, modName: string) => void;
  } = $props();

  let recentMods = $state<LedgerEntry[]>([]);
  let copiedFeedback = $state(false);

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    entry: LedgerEntry | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    entry: null
  });

  const MOD_TYPE_LABELS: Record<string, string> = {
    'archive': 'Archive',
    'cet': 'CET',
    'red4ext': 'RED4ext',
    'redscript': 'R6\\Scripts',
    'r6tweaks': 'R6\\Tweaks'
  };

  // Group mods by exact timestamp
  let groupedMods = $derived.by(() => {
    const groups = new Map<number, LedgerEntry[]>();
    for (const mod of recentMods) {
      if (!groups.has(mod.first_seen)) {
        groups.set(mod.first_seen, []);
      }
      groups.get(mod.first_seen)!.push(mod);
    }
    
    const arr = Array.from(groups.entries()).map(([ts, mods]) => {
      mods.sort((a, b) => a.name.localeCompare(b.name));
      return { timestamp: ts, mods };
    });
    
    arr.sort((a, b) => b.timestamp - a.timestamp);
    return arr;
  });

  function formatGroupHeader(ts: number) {
    if (!ts) return 'Unknown Date';
    const d = new Date(ts);
    const today = new Date();
    const isToday = d.getDate() === today.getDate() && d.getMonth() === today.getMonth() && d.getFullYear() === today.getFullYear();
    
    const timeStr = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (isToday) {
      return `Today at ${timeStr}`;
    }
    return `${d.toLocaleDateString()} at ${timeStr}`;
  }

  async function loadRecentMods() {
    if (!gamePath) return;
    try {
      recentMods = await invoke<LedgerEntry[]>('get_recent_mods', { gamePath, hours: newModsHours });
    } catch (err) {
      console.error(`Failed to load recent mods: ${err}`);
    }
  }

  function handleTimeframeChange() {
    loadRecentMods();
    loadConfigFromDisk().then(config => {
      config.newModsHours = newModsHours;
      saveConfigToDisk(config);
    });
  }

  let newModsSaveTimeout: ReturnType<typeof setTimeout>;
  function handleNewModsConfigChange() {
    clearTimeout(newModsSaveTimeout);
    newModsSaveTimeout = setTimeout(async () => {
      try {
        const config = await loadConfigFromDisk();
        config.newModsHours = newModsHours;
        // showNewMods is volatile, do not save to disk
        await saveConfigToDisk(config);
      } catch (err) {
        console.error(`Failed to save new mods config: ${err}`);
      }
    }, 200);
  }

  function toggleShowNewMods() {
    showNewMods = !showNewMods;
    handleNewModsConfigChange();
  }

  onMount(() => {
    loadRecentMods();
  });

  $effect(() => {
    if (gamePath) {
      loadRecentMods();
    }
  });

  function openContextMenu(event: MouseEvent, entry: LedgerEntry) {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    const menuWidth = 200;
    const menuHeight = 120;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    contextMenu = { visible: true, x: posX, y: posY, entry };
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.entry = null;
      copiedFeedback = false;
    }
  }

  async function handleShowInExplorer() {
    if (!contextMenu.entry) return;
    try {
      await revealItemInDir(contextMenu.entry.path);
    } catch (err) {
      console.error(`Failed to reveal file in explorer: ${err}`);
    }
    closeContextMenu();
  }

  async function handleShowInApp() {
    if (!contextMenu.entry) return;
    const entry = contextMenu.entry;
    
    const tabMap: Record<string, string> = {
      'archive': 'archive',
      'cet': 'cet',
      'red4ext': 'red4ext',
      'redscript': 'redscript',
      'r6tweaks': 'r6tweaks'
    };
    
    const targetTab = tabMap[entry.mod_type];
    if (targetTab && onNavigateToMod) {
      onNavigateToMod(targetTab, entry.name);
    }
    closeContextMenu();
  }

  async function handleCopyName() {
    if (!contextMenu.entry) return;
    try {
      await navigator.clipboard.writeText(contextMenu.entry.name);
      copiedFeedback = true;
      setTimeout(() => {
        closeContextMenu();
      }, 400);
    } catch (err) {
      console.error(`Failed to copy name: ${err}`);
      closeContextMenu();
    }
  }
</script>

<svelte:window onclick={closeContextMenu} oncontextmenu={(e) => e.preventDefault()} />

<!-- Context Menu -->
{#if contextMenu.visible && contextMenu.entry}
  <div
    class="fixed z-50 w-48 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center gap-2">
      <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={contextMenu.entry.name}>
        {contextMenu.entry.name}
      </span>
    </div>
    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <FolderSearch class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Explorer</span>
      </button>
      <button
        type="button"
        onclick={handleShowInApp}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        title="Switches to the correct tab and highlights the mod"
      >
        <ExternalLink class="h-3.5 w-3.5 text-cyan-400" />
        <span>Show in App</span>
      </button>
      <button
        type="button"
        onclick={handleCopyName}
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

<div class="flex flex-col h-full overflow-hidden bg-nvidia-bg text-nvidia-text-primary select-none">
  
  <!-- Top Control Bar -->
  <div class="p-6 pb-4 shrink-0 border-b border-nvidia-border/50 bg-nvidia-surface/20 flex items-start justify-between gap-4">
    <div>
      <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
        <Clock class="h-4 w-4" />
        <span>MOD HISTORY & LEDGER</span>
      </div>
      <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">Recently Added Mods</h2>
      <p class="text-xs text-nvidia-text-muted max-w-2xl">
        Track mods added to your workspace over time.
      </p>
    </div>
  </div>

  <!-- Ledger Table Area -->
  <div class="flex-1 flex flex-col p-6 overflow-hidden">
    <div class="rounded-lg border border-nvidia-border bg-nvidia-card flex flex-col h-full shadow-sm overflow-hidden">
      
      <!-- Table Toolbar -->
      <div class="px-4 py-3 bg-nvidia-surface/60 border-b border-nvidia-border flex items-start justify-between shrink-0">
        <div class="flex items-center gap-2 mt-1.5">
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Ledger Entries</span>
          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-bg border border-nvidia-border text-nvidia-text-muted">
            {recentMods.length}
          </span>
        </div>
        
        <div class="flex flex-col items-end gap-1.5">
          <div class="flex items-center gap-2">
            <select
              bind:value={newModsHours}
              onchange={handleTimeframeChange}
              class="py-1.5 px-2 bg-nvidia-bg border border-nvidia-border rounded text-xs font-bold text-nvidia-text-primary focus:outline-none cursor-pointer"
            >
              <option value={1}>Last 1 Hour</option>
              <option value={6}>Last 6 Hours</option>
              <option value={24}>Last 24 Hours</option>
              <option value={72}>Last 3 Days</option>
              <option value={168}>Last 7 Days</option>
              <option value={720}>Last 30 Days</option>
            </select>

            <button
              type="button"
              onclick={toggleShowNewMods}
              class="flex items-center gap-2 px-3 py-1.5 rounded border transition-colors cursor-pointer {showNewMods ? 'border-nvidia-accent/60 bg-nvidia-accent/10 font-semibold' : 'border-nvidia-border bg-nvidia-surface hover:bg-nvidia-card text-nvidia-text-muted hover:text-nvidia-text-primary font-medium'}"
              style={showNewMods ? 'background-color: var(--theme-accent); color: var(--theme-accent-contrast);' : ''}
            >
              <span
                class="inline-flex items-center justify-center px-1.5 py-0.2 rounded text-[9px] font-mono font-bold tracking-wider uppercase shrink-0 shadow-xs"
                style={showNewMods ? 'background-color: rgba(0,0,0,0.25); color: var(--theme-accent-contrast);' : 'background-color: var(--theme-accent); color: var(--theme-accent-contrast);'}
              >
                NEW
              </span>
              <span>{showNewMods ? 'Highlights Active' : 'Apply Label'}</span>
            </button>
          </div>
          <div class="text-[9px] text-nvidia-text-muted leading-tight">
            Temporarily applies label to the mods shown below
          </div>
        </div>
      </div>

      <!-- Table Body (Grouped) -->
      <div class="flex-1 overflow-y-auto p-2 space-y-4 bg-nvidia-bg/30">
        {#if groupedMods.length === 0}
          <div class="p-10 flex flex-col items-center justify-center text-center text-nvidia-text-muted h-full">
            <Clock class="h-8 w-8 mb-3 opacity-20" />
            <p class="text-xs font-mono">No new mods detected in the selected timeframe.</p>
          </div>
        {:else}
          {#each groupedMods as group (group.timestamp)}
            <div class="space-y-1">
              <!-- Group Header -->
              <div class="flex items-center gap-2 px-2 py-1.5 mb-1 border-b border-nvidia-border/50">
                <CalendarDays class="h-3.5 w-3.5 text-nvidia-accent" />
                <span class="text-[11px] font-bold text-nvidia-text-primary uppercase tracking-wider">
                  Added {formatGroupHeader(group.timestamp)}
                </span>
                <span class="text-[10px] font-mono text-nvidia-text-muted ml-1">
                  ({group.mods.length} {group.mods.length === 1 ? 'mod' : 'mods'})
                </span>
              </div>

              <!-- Group Items -->
              {#each group.mods as entry (entry.path)}
                <div
                  oncontextmenu={(e) => openContextMenu(e, entry)}
                  class="grid grid-cols-[120px_1fr] gap-4 px-3 rounded border border-transparent hover:border-nvidia-border/50 hover:bg-nvidia-surface/80 transition items-center cursor-context-menu density-row ml-2"
                >
                  <div>
                    <span class="text-[9px] font-mono font-bold uppercase px-1.5 py-0.5 rounded border bg-nvidia-accent/10 border-nvidia-accent/20 text-nvidia-accent">
                      {MOD_TYPE_LABELS[entry.mod_type] || entry.mod_type}
                    </span>
                  </div>
                  <div class="text-xs font-mono text-nvidia-text-primary truncate" title={entry.name}>
                    {entry.name}
                  </div>
                </div>
              {/each}
            </div>
          {/each}
        {/if}
      </div>

    </div>
  </div>
</div>