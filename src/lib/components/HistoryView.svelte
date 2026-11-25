<script lang="ts">
  import { onMount, untrack, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import type { LedgerEntry, ScanResult, MultiLinkCluster } from '$lib/types';
  import {
    FolderSearch, ExternalLink, Copy, Check, Search, X,
    Archive, Cpu, Puzzle, FileCode2, Sliders, Power, ArrowDown, ArrowUp, Clock, ChevronDown,
    Link, Unlink, MessageSquare, MoreVertical, ChevronRight, Calendar
  } from 'lucide-svelte';
  import NoteModal from '$lib/components/NoteModal.svelte';

  let {
    gamePath = '',
    newModsHours = $bindable(6),
    targetHighlightMod = null,
    onNavigateToMod,
    currentTab = 'history',
    scanResult = null
  }: {
    gamePath: string;
    newModsHours?: number;
    targetHighlightMod?: string | null;
    onNavigateToMod: (tab: string, modName: string) => void;
    currentTab?: string;
    scanResult?: ScanResult | null;
  } = $props();

  let recentMods = $state<LedgerEntry[]>([]);
  let multiLinks = $state<MultiLinkCluster[]>([]);
  let notesConfig = $state<Record<string, string>>({});
  let copiedFeedback = $state(false);
  let searchQuery = $state('');
  
  let dateViewMode = $state<'first_seen' | 'os_created' | 'os_modified'>('first_seen');
  let sortBy = $state<'name' | 'mod_type' | 'first_seen' | 'os_created' | 'os_modified'>('first_seen');
  let sortDesc = $state(true);

  let highlightedModName = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;
  const modNodeMap = new Map<string, HTMLElement>();

  let selectedMods = $state<Set<string>>(new Set());
  let lastSelectedPath = $state<string | null>(null);

  let expandedMultiLinks = $state<Record<string, boolean>>({});
  let isNoteModalOpen = $state(false);
  let targetNoteModName = $state('');

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

  // Multi-Link Modal State
  let isLinkModalOpen = $state(false);
  let linkDraftName = $state('');
  let linkDraftMembers = $state<LedgerEntry[]>([]);

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
      untrack(() => { 
        loadRecentMods(); 
        loadMultiLinks();
        loadNotes();
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

  let displayMods = $derived.by(() => {
    let arr = [...recentMods];
    const query = searchQuery.toLowerCase().trim();
    if (query) {
      arr = arr.filter(m => m.name.toLowerCase().includes(query));
    }
    
    arr.sort((a, b) => {
      if (sortBy === 'name') {
        return sortDesc ? b.name.localeCompare(a.name) : a.name.localeCompare(b.name);
      }
      if (sortBy === 'mod_type') {
        return sortDesc ? b.mod_type.localeCompare(a.mod_type) : a.mod_type.localeCompare(b.mod_type);
      }
      
      let valA = a[sortBy] || 0;
      let valB = b[sortBy] || 0;
      if (valA === valB) return a.name.localeCompare(b.name);
      return sortDesc ? valB - valA : valA - valB;
    });

    return arr.map(mod => ({ ...mod, displayTime: new Date(mod[sortBy === 'name' || sortBy === 'mod_type' ? 'first_seen' : sortBy] || 0) }));
  });

  function formatHistoryDate(d: Date) {
    if (d.getTime() === 0) return 'Unknown';
    const datePart = d.toLocaleDateString(undefined, { month: 'short', day: '2-digit', year: 'numeric' });
    const timePart = d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    return `${datePart} ${timePart}`;
  }

  async function loadRecentMods() {
    if (!gamePath) return;
    try {
      recentMods = await invoke<LedgerEntry[]>('get_recent_mods', { gamePath, hours: 999999 });
    } catch (err) { console.error(`Failed to load recent mods: ${err}`); }
  }

  async function loadMultiLinks() {
    if (!gamePath) return;
    try {
      multiLinks = await invoke<MultiLinkCluster[]>('load_multilinks_config');
    } catch (err) { console.error(`Failed to load multilinks: ${err}`); }
  }

  async function loadNotes() {
    if (!gamePath) return;
    try {
      notesConfig = await invoke<Record<string, string>>('load_notes_config');
    } catch (err) { console.error(`Failed to load notes: ${err}`); }
  }

  onMount(() => { 
    newModsHours = 999999;
    const savedDateView = localStorage.getItem('cp2077_history_date_view');
    if (savedDateView === 'first_seen' || savedDateView === 'os_created' || savedDateView === 'os_modified') {
      dateViewMode = savedDateView;
      sortBy = savedDateView;
    }
    loadRecentMods(); 
    loadMultiLinks();
    loadNotes();
  });

  function getClusterForMod(modName: string): MultiLinkCluster | undefined {
    return multiLinks.find(c => c.members.some(m => m.name === modName));
  }

  function toggleMultiLink(fileName: string) {
    expandedMultiLinks[fileName] = !expandedMultiLinks[fileName];
  }

  function handlePointerDown(e: PointerEvent, entry: LedgerEntry) {
    if (e.button !== 0) return;
    const isCtrl = e.ctrlKey || e.metaKey;
    const isShift = e.shiftKey;

    if (isShift && lastSelectedPath) {
      const startIdx = displayMods.findIndex(m => m.path === lastSelectedPath);
      const endIdx = displayMods.findIndex(m => m.path === entry.path);
      if (startIdx !== -1 && endIdx !== -1) {
        const min = Math.min(startIdx, endIdx);
        const max = Math.max(startIdx, endIdx);
        if (!isCtrl) selectedMods.clear();
        for (let i = min; i <= max; i++) {
          selectedMods.add(displayMods[i].path);
        }
      }
    } else if (isCtrl) {
      if (selectedMods.has(entry.path)) selectedMods.delete(entry.path);
      else selectedMods.add(entry.path);
      lastSelectedPath = entry.path;
    } else {
      selectedMods.clear();
      selectedMods.add(entry.path);
      lastSelectedPath = entry.path;
    }
    selectedMods = new Set(selectedMods);
  }

  async function handleToggle(entry: LedgerEntry) {
    const nextState = !entry.enabled;
    const cluster = getClusterForMod(entry.name);

    if (cluster) {
      // Transactional Multi-Link Toggle
      try {
        await invoke('toggle_multilink_cluster', { gamePath, cluster, enable: nextState });
        // Update local state for all members
        const memberNames = new Set(cluster.members.map(m => m.name));
        recentMods = recentMods.map(m => {
          if (memberNames.has(m.name)) {
            return { ...m, enabled: nextState };
          }
          return m;
        });
      } catch (err) {
        console.error(err);
      }
    } else {
      // Standard Single Toggle
      entry.enabled = nextState;
      recentMods = [...recentMods];
      try {
        if (entry.mod_type === 'archive') {
          await invoke('toggle_mod_state', { gamePath, modName: entry.name, enable: nextState });
        } else {
          await invoke('toggle_plugin_state', { gamePath, modType: entry.mod_type, modName: entry.name, enable: nextState });
        }
      } catch (err) {
        console.error(err);
        entry.enabled = !nextState;
        recentMods = [...recentMods];
      }
    }
  }

  function openContextMenu(event: MouseEvent, entry: LedgerEntry) {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;
    
    if (!selectedMods.has(entry.path)) {
      selectedMods.clear();
      selectedMods.add(entry.path);
      selectedMods = new Set(selectedMods);
      lastSelectedPath = entry.path;
    }

    const menuWidth = 200;
    const menuHeight = selectedMods.size > 1 ? 200 : 160;
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

  async function handleContextMenuToggle() {
    if (!contextMenu.entry) return;
    const targetState = !contextMenu.entry.enabled;
    
    if (selectedMods.size > 1 && selectedMods.has(contextMenu.entry.path)) {
      const itemsToToggle = displayMods.filter(m => selectedMods.has(m.path) && m.enabled !== targetState);
      for (const item of itemsToToggle) {
        item.enabled = targetState;
      }
      recentMods = [...recentMods];
      
      for (const item of itemsToToggle) {
        try {
          if (item.mod_type === 'archive') {
            await invoke('toggle_mod_state', { gamePath, modName: item.name, enable: targetState });
          } else {
            await invoke('toggle_plugin_state', { gamePath, modType: item.mod_type, modName: item.name, enable: targetState });
          }
        } catch (err) { console.error(err); }
      }
    } else {
      handleToggle(contextMenu.entry);
    }
    closeContextMenu();
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

  async function handleNoteConfirm(note: string) {
    if (!targetNoteModName) return;
    const targetName = targetNoteModName;
    
    if (note.trim() === '') {
      delete notesConfig[targetName];
    } else {
      notesConfig[targetName] = note.trim();
    }
    notesConfig = { ...notesConfig };
    try {
      await invoke('save_notes_config', { notes: notesConfig });
    } catch (err) {
      console.error('Failed to save notes:', err);
    }
  }

  // Multi-Link Logic
  function openLinkModalFromSelection() {
    const selected = displayMods.filter(m => selectedMods.has(m.path));
    // Filter out mods that are already in a cluster
    linkDraftMembers = selected.filter(m => !getClusterForMod(m.name));
    
    if (linkDraftMembers.length < 2) {
      // If they selected mods that are already linked, we can't create a new link with them.
      closeContextMenu();
      return;
    }
    
    linkDraftName = '';
    isLinkModalOpen = true;
    closeContextMenu();
  }

  async function confirmMultiLink() {
    if (!linkDraftName.trim() || linkDraftMembers.length < 2) return;
    const newCluster: MultiLinkCluster = {
      id: crypto.randomUUID(),
      name: linkDraftName.trim(),
      members: linkDraftMembers.map(m => ({ mod_type: m.mod_type, name: m.name }))
    };
    multiLinks = [...multiLinks, newCluster];
    try {
      await invoke('save_multilinks_config', { clusters: multiLinks });
    } catch (e) { console.error(e); }
    isLinkModalOpen = false;
    selectedMods.clear();
  }

  async function handleUnlink(clusterId: string, modName: string) {
    const clusterIdx = multiLinks.findIndex(c => c.id === clusterId);
    if (clusterIdx === -1) return;
    
    let cluster = multiLinks[clusterIdx];
    cluster.members = cluster.members.filter(m => m.name !== modName);
    
    if (cluster.members.length < 2) {
      multiLinks.splice(clusterIdx, 1);
    } else {
      multiLinks[clusterIdx] = cluster;
    }
    
    multiLinks = [...multiLinks];
    try {
      await invoke('save_multilinks_config', { clusters: multiLinks });
    } catch (e) { console.error(e); }
    closeContextMenu();
  }

  function handleSort(column: 'name' | 'mod_type' | 'first_seen' | 'os_created' | 'os_modified') {
    if (sortBy === column) {
      sortDesc = !sortDesc;
    } else {
      sortBy = column;
      sortDesc = true;
    }
  }
</script>

<svelte:window 
  onclick={() => { closeContextMenu(); }} 
  oncontextmenu={(e) => e.preventDefault()} 
/>

<!-- Context Menu -->
{#if contextMenu.visible && contextMenu.entry}
  {@const activeCluster = getClusterForMod(contextMenu.entry.name)}
  <div
    class="fixed z-50 w-56 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center gap-2" style="border-color: var(--theme-border); background-color: var(--theme-surface);">
      <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);" title={contextMenu.entry.name}>
        {#if selectedMods.size > 1}
          {selectedMods.size} Mods Selected
        {:else}
          {contextMenu.entry.name}
        {/if}
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
        onclick={handleContextMenuToggle}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Power class="h-3.5 w-3.5" style="color: {contextMenu.entry.enabled ? 'var(--color-warning)' : 'var(--theme-accent)'};" />
        {#if selectedMods.size > 1}
          <span>{contextMenu.entry.enabled ? 'Disable ' + selectedMods.size + ' Mods' : 'Enable ' + selectedMods.size + ' Mods'}</span>
        {:else}
          <span>{contextMenu.entry.enabled ? 'Disable Mod' : 'Enable Mod'}</span>
        {/if}
      </button>

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

      <button
        type="button"
        onclick={() => { 
          targetNoteModName = contextMenu.entry?.name || ''; 
          closeContextMenu(); 
          isNoteModalOpen = true; 
        }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
        <span>{notesConfig[contextMenu.entry?.name || ''] ? 'Edit Note...' : 'Add Note...'}</span>
      </button>

      {#if activeCluster}
        <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>
        <button
          type="button"
          onclick={() => handleUnlink(activeCluster.id, contextMenu.entry!.name)}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Unlink class="h-3.5 w-3.5" style="color: var(--color-warning);" />
          <span class="truncate">Unlink from {activeCluster.name}</span>
        </button>
      {:else if selectedMods.size > 1}
        <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>
        <button
          type="button"
          onclick={openLinkModalFromSelection}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Link class="h-3.5 w-3.5" style="color: var(--color-info);" />
          <span>Create Multi-Link...</span>
        </button>
      {/if}

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

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

<!-- Multi-Link Creation Modal -->
{#if isLinkModalOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={() => isLinkModalOpen = false}
    role="presentation"
  >
    <div
      class="w-full max-w-md rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b flex items-center justify-between shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5">
          <Link class="h-4 w-4" style="color: var(--color-info);" />
          <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Create Multi-Link</span>
        </div>
        <button
          type="button"
          onclick={() => isLinkModalOpen = false}
          class="p-1 rounded transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          <X class="h-4 w-4 hover:text-[var(--theme-text-primary)]" />
        </button>
      </div>

      <div class="p-5 space-y-4">
        <p class="text-xs leading-relaxed" style="color: var(--theme-text-muted);">
          Linking these <strong style="color: var(--theme-text-primary);">{linkDraftMembers.length}</strong> mods will ensure they are always enabled or disabled together across all framework tabs.
        </p>
        
        <div class="space-y-2">
          <label class="text-xs font-semibold" style="color: var(--theme-text-primary);">Cluster Name</label>
          <input
            type="text"
            bind:value={linkDraftName}
            placeholder="e.g., My Custom Outfit Cluster"
            class="w-full px-3 py-2 border rounded text-xs focus:outline-none font-sans"
            style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
          />
        </div>

        <div class="space-y-2">
          <label class="text-xs font-semibold" style="color: var(--theme-text-primary);">Included Mods</label>
          <div class="max-h-40 overflow-y-auto border rounded p-2 space-y-1" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
            {#each linkDraftMembers as member}
              <div class="text-[11px] font-mono truncate flex items-center gap-2" style="color: var(--theme-text-primary);">
                <div class="h-1.5 w-1.5 rounded-full shrink-0" style="background-color: var(--color-info);"></div>
                {member.name}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="px-5 py-3 border-t flex items-center justify-end gap-2.5 shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <button
          type="button"
          onclick={() => isLinkModalOpen = false}
          class="px-3.5 py-1.5 rounded text-xs transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={confirmMultiLink}
          disabled={!linkDraftName.trim()}
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-110"
          style="background-color: var(--color-info); color: #000000;"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>Create Link</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<div class="flex flex-col h-full select-none p-6" oncontextmenu={(e) => e.preventDefault()}>
  
  <div class="flex items-center justify-between gap-3 mb-3 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter Mod History..."
          class="w-full pl-9 pr-8 py-1.5 rounded text-xs focus:outline-none border font-sans"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
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
  </div>

  <!-- Native Sorting Header Row -->
  <div class="flex items-center justify-between px-2 py-2 border-b shrink-0 text-[10px] font-bold uppercase tracking-wider" style="border-color: var(--theme-border); color: var(--theme-text-muted); background-color: color-mix(in srgb, var(--theme-surface) 50%, transparent);">
    <div class="flex items-center gap-3 min-w-0 flex-1 pl-1">
      <div class="w-6 shrink-0"></div> <!-- Checkbox Spacer -->
      
      <button type="button" onclick={() => handleSort('name')} class="flex items-center gap-1 hover:text-[var(--theme-text-primary)] transition cursor-pointer min-w-0">
        <span>Name</span>
        {#if sortBy === 'name'}
          {#if sortDesc}<ArrowDown class="h-3 w-3" style="color: var(--theme-accent);" />{:else}<ArrowUp class="h-3 w-3" style="color: var(--theme-accent);" />{/if}
        {/if}
      </button>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <button type="button" onclick={() => handleSort('mod_type')} class="w-24 flex items-center gap-1 hover:text-[var(--theme-text-primary)] transition cursor-pointer shrink-0">
        <span>Mod Type</span>
        {#if sortBy === 'mod_type'}
          {#if sortDesc}<ArrowDown class="h-3 w-3" style="color: var(--theme-accent);" />{:else}<ArrowUp class="h-3 w-3" style="color: var(--theme-accent);" />{/if}
        {/if}
      </button>

      <!-- Unified Date Column Header -->
      <div class="w-48 flex items-center gap-2 shrink-0">
        <Calendar class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
        <select 
          bind:value={dateViewMode} 
          onchange={(e) => {
            const val = e.currentTarget.value;
            localStorage.setItem('cp2077_history_date_view', val);
            if (['first_seen', 'os_created', 'os_modified'].includes(sortBy)) {
              sortBy = val as any;
            }
          }}
          class="bg-transparent focus:outline-none cursor-pointer hover:text-[var(--theme-text-primary)] transition-colors"
        >
          <option value="first_seen" style="background-color: var(--theme-surface);">R4 First-Seen</option>
          <option value="os_created" style="background-color: var(--theme-surface);">OS Created</option>
          <option value="os_modified" style="background-color: var(--theme-surface);">OS Modified</option>
        </select>
        <button type="button" onclick={() => handleSort(dateViewMode)} class="flex items-center hover:text-[var(--theme-text-primary)] transition cursor-pointer">
          {#if ['first_seen', 'os_created', 'os_modified'].includes(sortBy)}
            {#if sortDesc}<ArrowDown class="h-3 w-3" style="color: var(--theme-accent);" />{:else}<ArrowUp class="h-3 w-3" style="color: var(--theme-accent);" />{/if}
          {:else}
            <ArrowDown class="h-3 w-3 opacity-0" /> <!-- Spacer to prevent layout shift -->
          {/if}
        </button>
      </div>

      <div class="w-6 shrink-0"></div> <!-- Note Spacer -->
      <div class="w-10 shrink-0"></div> <!-- Multi-Link Spacer -->
      <div class="w-6 shrink-0"></div> <!-- Context Menu Spacer -->
    </div>
  </div>

  <div class="flex-1 overflow-y-auto space-y-0 pr-1">
    {#if displayMods.length === 0}
      <div class="p-6 text-center text-xs border rounded mt-2" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);">
        {searchQuery ? 'No mods match your search query.' : 'No mods detected in the ledger.'}
      </div>
    {:else}
      {#each displayMods as item (item.path)}
        {@const isSelected = selectedMods.has(item.path)}
        {@const isHighlighted = highlightedModName === item.name}
        {@const activeCluster = getClusterForMod(item.name)}
        {@const isMultiLinkExpanded = !!expandedMultiLinks[item.name]}
        <div class="flex flex-col">
          <div
            use:registerModNode={item.name}
            onpointerdown={(e) => handlePointerDown(e, item)}
            oncontextmenu={(e) => openContextMenu(e, item)}
            class="flex items-center justify-between px-2 py-1.5 border-b transition-all duration-200 density-row cursor-context-menu
              {isHighlighted ? 'ring-1 z-10 relative bg-[color-mix(in_srgb,var(--theme-accent)_15%,transparent)] border-[var(--theme-accent)] shadow-[0_0_15px_color-mix(in_srgb,var(--theme-accent)_35%,transparent)]' : 
              (isSelected ? 'bg-[color-mix(in_srgb,var(--theme-accent)_10%,transparent)] border-[color-mix(in_srgb,var(--theme-accent)_60%,transparent)]' : 
              'bg-transparent border-[var(--theme-border)] hover:bg-[color-mix(in_srgb,var(--theme-accent)_12%,transparent)]')}
              {item.enabled ? 'opacity-100' : 'opacity-50'}"
          >
            <!-- HYBRID SLOT ARCHITECTURE: LEFT ZONE -->
            <div class="flex items-center gap-3 min-w-0 flex-1 pl-1">
              
              <!-- Slot 1: Checkbox (w-6) -->
              <div class="w-6 flex justify-center shrink-0">
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); handleToggle(item); }}
                  aria-label={item.enabled ? "Disable mod " + item.name : "Enable mod " + item.name}
                  class="w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors cursor-pointer"
                  style="border-color: {item.enabled ? 'var(--theme-accent)' : 'var(--theme-border)'}; background-color: {item.enabled ? 'var(--theme-accent)' : 'transparent'};"
                >
                  {#if item.enabled}
                    <Check class="h-2.5 w-2.5" style="color: var(--theme-accent-contrast);" strokeWidth={3} />
                  {/if}
                </button>
              </div>

              <!-- Slot 2: Mod File Name -->
              <span class="font-mono text-[11px] truncate" style="color: {item.enabled ? 'var(--theme-text-primary)' : 'var(--theme-text-muted)'};">
                {item.name}
              </span>
            </div>

            <!-- HYBRID SLOT ARCHITECTURE: RIGHT ZONE -->
            <div class="flex items-center gap-1 shrink-0">
              
              <!-- Slot 1: Mod Type Badge (w-24) -->
              <div class="w-24 shrink-0">
                <span class="inline-flex items-center gap-1.5 text-[9px] font-mono font-bold uppercase px-1.5 py-0.5 rounded border" style="background-color: color-mix(in srgb, var(--theme-text-muted) 15%, transparent); color: var(--theme-text-muted); border-color: color-mix(in srgb, var(--theme-text-muted) 30%, transparent);">
                  {#if item.mod_type === 'archive'}
                    <Archive class="h-3 w-3" />
                  {:else if item.mod_type === 'cet'}
                    <Cpu class="h-3 w-3" />
                  {:else if item.mod_type === 'red4ext'}
                    <Puzzle class="h-3 w-3" />
                  {:else if item.mod_type === 'redscript'}
                    <FileCode2 class="h-3 w-3" />
                  {:else if item.mod_type === 'r6tweaks'}
                    <Sliders class="h-3 w-3" />
                  {/if}
                  {MOD_TYPE_LABELS[item.mod_type] || item.mod_type}
                </span>
              </div>

              <!-- Slot 2: Unified Date (w-48) -->
              <div class="w-48 shrink-0 text-[10px] font-mono" style="color: var(--theme-text-muted);">
                {formatHistoryDate(new Date(item[dateViewMode] || 0))}
              </div>

              <!-- Slot 3: Note Icon (w-6) -->
              <div class="w-6 flex justify-center shrink-0">
                {#if notesConfig[item.name]}
                  <button 
                    type="button"
                    onclick={(e) => { e.stopPropagation(); targetNoteModName = item.name; isNoteModalOpen = true; }}
                    class="hover:scale-110 transition-transform cursor-pointer"
                    title={notesConfig[item.name]}
                  >
                    <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
                  </button>
                {/if}
              </div>

              <!-- Slot 4: Multi-Link Icon + Chevron (w-10) -->
              <div class="w-10 flex items-center justify-center gap-0.5 shrink-0">
                {#if activeCluster}
                  <Link class="h-3.5 w-3.5" style="color: var(--color-warning);" title="Part of Multi-Link Cluster: {activeCluster.name}" />
                  <button
                    type="button"
                    onclick={(e) => { e.stopPropagation(); toggleMultiLink(item.name); }}
                    class="p-0.5 rounded transition cursor-pointer hover:bg-[var(--theme-card)]"
                    style="color: var(--color-warning);"
                    title="Toggle Multi-Link details"
                  >
                    {#if isMultiLinkExpanded}
                      <ChevronDown class="h-3.5 w-3.5" />
                    {:else}
                      <ChevronRight class="h-3.5 w-3.5" />
                    {/if}
                  </button>
                {/if}
              </div>

              <!-- Slot 5: Context Menu (w-6) -->
              <div class="w-6 flex justify-end shrink-0">
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); openContextMenu(e, item); }}
                  class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-surface)]"
                  style="color: var(--theme-text-muted);"
                  title="More options"
                >
                  <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
                </button>
              </div>
            </div>
          </div>

          <!-- Inline Drawer: Multi-Link -->
          {#if isMultiLinkExpanded && activeCluster}
            <div class="px-4 py-2 border-t space-y-2 text-xs" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-card) 30%, transparent);">
              <div>
                <span class="text-[10px] font-semibold uppercase tracking-wider" style="color: var(--color-warning);">Multi-Link Cluster: {activeCluster.name}</span>
                <div class="mt-1 flex flex-wrap gap-1">
                  {#each activeCluster.members as member}
                    <button type="button" onclick={(e) => { e.stopPropagation(); if(onNavigateToMod) onNavigateToMod(member.mod_type === 'archive' ? 'archive' : member.mod_type, member.name); }} class="px-1.5 py-0.2 rounded border text-[10px] font-mono cursor-pointer hover:brightness-125 transition-all" style="background-color: color-mix(in srgb, var(--color-warning) 15%, transparent); border-color: color-mix(in srgb, var(--color-warning) 40%, transparent); color: var(--color-warning);" title="Jump to mod">
                      {member.name}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<NoteModal
  bind:isOpen={isNoteModalOpen}
  modName={targetNoteModName}
  initialNote={notesConfig[targetNoteModName] || ''}
  onConfirm={handleNoteConfirm}
  onCancel={() => isNoteModalOpen = false}
/>
