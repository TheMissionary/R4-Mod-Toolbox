<script lang="ts">
  import { onMount, untrack, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { loadConfigFromDisk, saveConfigToDisk } from '$lib/theme';
  import type { LedgerEntry, ScanResult } from '$lib/types';
  import {
    Clock, FolderSearch, ExternalLink, Copy, Check, CalendarDays, Search, X,
    Archive, Cpu, Puzzle, FileCode2, Sliders
  } from 'lucide-svelte';

  let {
    gamePath = '',
    newModsHours = $bindable(6),
    targetHighlightMod = null,
    onNavigateToMod,
    currentTab = 'history',
    scanResult = null
  }: {
    gamePath: string;
    newModsHours: number;
    targetHighlightMod?: string | null;
    onNavigateToMod: (tab: string, modName: string) => void;
    currentTab?: string;
    scanResult?: ScanResult | null;
  } = $props();

  let recentMods = $state<LedgerEntry[]>([]);
  let copiedFeedback = $state(false);
  let searchQuery = $state('');

  let highlightedModName = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;
  const modNodeMap = new Map<string, HTMLElement>();

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

  $effect(() => {
    if (currentTab !== 'history' && searchQuery !== '') {
      untrack(() => { searchQuery = ''; });
    }
  });

  $effect(() => {
    const trigger = scanResult;
    if (trigger && gamePath) {
      untrack(() => { loadRecentMods(); });
    }
  });

  $effect(() => {
    if (searchQuery.trim() !== '' && newModsHours !== 999999) {
      untrack(() => {
        newModsHours = 999999;
        handleTimeframeChange();
      });
    }
  });

  $effect(() => {
    if (targetHighlightMod) {
      untrack(() => { focusModInHistory(targetHighlightMod as string); });
    }
  });

  async function focusModInHistory(modName: string) {
    if (searchQuery !== '') { searchQuery = ''; }
    if (newModsHours !== 999999) {
      newModsHours = 999999;
      await handleTimeframeChange();
    }
    await tick();
    let targetElement = modNodeMap.get(modName);
    let retries = 0;
    while (!targetElement && retries < 10) {
      await new Promise(r => setTimeout(r, 50));
      targetElement = modNodeMap.get(modName);
      retries++;
    }
    if (targetElement) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
      highlightedModName = modName;
      if (highlightTimeoutId) clearTimeout(highlightTimeoutId);
      highlightTimeoutId = window.setTimeout(() => { highlightedModName = null; }, 2500);
    }
  }

  let groupedMods = $derived.by(() => {
    const query = searchQuery.toLowerCase().trim();
    const filtered = query ? recentMods.filter(m => m.name.toLowerCase().includes(query)) : recentMods;
    const groups = new Map<number, LedgerEntry[]>();
    for (const mod of filtered) {
      if (!groups.has(mod.first_seen)) { groups.set(mod.first_seen, []); }
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
    if (isToday) { return `Today at ${timeStr}`; }
    return `${d.toLocaleDateString()} at ${timeStr}`;
  }

  async function loadRecentMods() {
    if (!gamePath) return;
    try {
      recentMods = await invoke<LedgerEntry[]>('get_recent_mods', { gamePath, hours: newModsHours });
    } catch (err) { console.error(`Failed to load recent mods: ${err}`); }
  }

  async function handleTimeframeChange() {
    await loadRecentMods();
    loadConfigFromDisk().then(config => {
      config.newModsHours = newModsHours;
      saveConfigToDisk(config);
    });
  }

  onMount(() => { loadRecentMods(); });

  $effect(() => { if (gamePath) { loadRecentMods(); } });

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
    try { await revealItemInDir(contextMenu.entry.path); } catch (err) { console.error(err); }
    closeContextMenu();
  }

  async function handleShowInApp() {
    if (!contextMenu.entry) return;
    const entry = contextMenu.entry;
    const tabMap: Record<string, string> = {
      'archive': 'archive', 'cet': 'cet', 'red4ext': 'red4ext', 'redscript': 'redscript', 'r6tweaks': 'r6tweaks'
    };
    const targetTab = tabMap[entry.mod_type];
    if (targetTab && onNavigateToMod) { onNavigateToMod(targetTab, entry.name); }
    closeContextMenu();
  }

  async function handleCopyName() {
    if (!contextMenu.entry) return;
    try {
      await navigator.clipboard.writeText(contextMenu.entry.name);
      copiedFeedback = true;
      setTimeout(() => { closeContextMenu(); }, 400);
    } catch (err) { console.error(err); closeContextMenu(); }
  }
</script>

<svelte:window onclick={closeContextMenu} oncontextmenu={(e) => e.preventDefault()} />

<!-- Context Menu -->
{#if contextMenu.visible && contextMenu.entry}
  <div
    class="fixed z-50 w-48 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center gap-2" style="border-color: var(--theme-border); background-color: var(--theme-surface);">
      <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);" title={contextMenu.entry.name}>
        {contextMenu.entry.name}
      </span>
    </div>
    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <FolderSearch class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in Explorer</span>
      </button>
      <button
        type="button"
        onclick={handleShowInApp}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
        title="Switches to the correct tab and highlights the mod"
      >
        <ExternalLink class="h-3.5 w-3.5" style="color: var(--color-info);" />
        <span>Show in App</span>
      </button>
      <button
        type="button"
        onclick={handleCopyName}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <div class="flex items-center gap-2.5">
          {#if copiedFeedback}
            <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
            <span class="font-medium" style="color: var(--theme-accent);">Copied!</span>
          {:else}
            <Copy class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
            <span>Copy Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="flex flex-col h-full overflow-hidden select-none" style="background-color: var(--theme-bg); color: var(--theme-text-primary);">
  
  <!-- Top Control Bar -->
  <div class="p-6 pb-4 shrink-0 border-b flex items-start justify-between gap-4" style="border-color: var(--theme-border); background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent);">
    <div>
      <div class="flex items-center gap-2 text-xs font-mono mb-2" style="color: var(--theme-accent);">
        <Clock class="h-4 w-4" />
        <span>MOD HISTORY & LEDGER</span>
      </div>
      <h2 class="text-xl font-bold mb-1" style="color: var(--theme-text-primary);">Recently Added Mods</h2>
      <p class="text-xs max-w-2xl" style="color: var(--theme-text-muted);">
        Track all mods by R4's "First Seen" date.
      </p>
    </div>
  </div>

  <!-- Ledger Table Area -->
  <div class="flex-1 flex flex-col p-6 overflow-hidden">
    <div class="rounded-lg border flex flex-col h-full shadow-sm overflow-hidden" style="background-color: var(--theme-card); border-color: var(--theme-border);">
      
      <!-- Table Toolbar -->
      <div class="px-4 py-3 border-b flex items-center justify-between shrink-0 gap-4" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-3 flex-1">
          <span class="text-xs font-bold uppercase tracking-wider shrink-0" style="color: var(--theme-text-primary);">Ledger Entries</span>
          
          <div class="relative max-w-md flex-1 ml-2">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search All Mods..."
              class="w-full pl-9 pr-8 py-1.5 rounded border text-xs focus:outline-none font-sans"
              style="background-color: var(--theme-bg); border-color: var(--theme-border); color: var(--theme-text-primary);"
            />
            {#if searchQuery !== ''}
              <button
                type="button"
                onclick={() => searchQuery = ''}
                class="absolute right-2 top-1/2 -translate-y-1/2 p-1 transition cursor-pointer hover:brightness-125"
                style="color: var(--theme-text-muted);"
                title="Clear search"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            {/if}
          </div>
        </div>

        <div class="flex items-center gap-3 shrink-0">
          <select
            bind:value={newModsHours}
            onchange={handleTimeframeChange}
            class="py-1.5 px-2 border rounded text-xs font-bold focus:outline-none cursor-pointer"
            style="background-color: var(--theme-bg); border-color: var(--theme-border); color: var(--theme-text-primary);"
          >
            <option value={1}>Last 1 Hour</option>
            <option value={6}>Last 6 Hours</option>
            <option value={24}>Last 24 Hours</option>
            <option value={72}>Last 3 Days</option>
            <option value={168}>Last 7 Days</option>
            <option value={720}>Last 30 Days</option>
            <option value={999999}>All Time</option>
          </select>

          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded border" style="background-color: var(--theme-bg); border-color: var(--theme-border); color: var(--theme-text-muted);">
            {searchQuery ? groupedMods.reduce((acc, g) => acc + g.mods.length, 0) : recentMods.length}
          </span>
        </div>
      </div>

      <!-- Table Body (Grouped) -->
      <div class="flex-1 overflow-y-auto p-2 space-y-4" style="background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
        {#if groupedMods.length === 0}
          <div class="p-10 flex flex-col items-center justify-center text-center h-full" style="color: var(--theme-text-muted);">
            <Clock class="h-8 w-8 mb-3 opacity-20" />
            <p class="text-xs font-mono">
              {searchQuery ? 'No mods match your search query.' : 'No new mods detected in the selected timeframe.'}
            </p>
          </div>
        {:else}
          {#each groupedMods as group (group.timestamp)}
            <div class="space-y-0">
              <!-- Group Header -->
              <div class="flex items-center gap-2 px-2 py-1.5 mb-1 border-b" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent);">
                <CalendarDays class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
                <span class="text-[11px] font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">
                  Added {formatGroupHeader(group.timestamp)}
                </span>
                <span class="text-[10px] font-mono ml-1" style="color: var(--theme-text-muted);">
                  ({group.mods.length} {group.mods.length === 1 ? 'mod' : 'mods'})
                </span>
              </div>

              <!-- Group Items (Seamless List) -->
              {#each group.mods as entry (entry.path)}
                {@const isHighlighted = highlightedModName === entry.name}
                <div
                  use:registerModNode={entry.name}
                  oncontextmenu={(e) => openContextMenu(e, entry)}
                  class="grid grid-cols-[140px_1fr] gap-4 px-2 py-1.5 border-b transition-all duration-200 items-center cursor-context-menu density-row ml-2
                    {isHighlighted ? 'ring-1 z-10 relative' : 'hover:bg-[var(--theme-surface)]'}"
                  style="border-color: var(--theme-border); {isHighlighted ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: var(--theme-accent); box-shadow: 0 0 15px color-mix(in srgb, var(--theme-accent) 35%, transparent);' : ''}"
                >
                  <div>
                    <!-- Monochromatic Badge -->
                    <span class="inline-flex items-center gap-1.5 text-[9px] font-mono font-bold uppercase px-1.5 py-0.5 rounded border" style="background-color: color-mix(in srgb, var(--theme-text-muted) 15%, transparent); color: var(--theme-text-muted); border-color: color-mix(in srgb, var(--theme-text-muted) 30%, transparent);">
                      {#if entry.mod_type === 'archive'}
                        <Archive class="h-3 w-3" />
                      {:else if entry.mod_type === 'cet'}
                        <Cpu class="h-3 w-3" />
                      {:else if entry.mod_type === 'red4ext'}
                        <Puzzle class="h-3 w-3" />
                      {:else if entry.mod_type === 'redscript'}
                        <FileCode2 class="h-3 w-3" />
                      {:else if entry.mod_type === 'r6tweaks'}
                        <Sliders class="h-3 w-3" />
                      {/if}
                      {MOD_TYPE_LABELS[entry.mod_type] || entry.mod_type}
                    </span>
                  </div>
                  <div class="text-xs font-mono truncate" style="color: var(--theme-text-primary);" title={entry.name}>
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