<script lang="ts">
  function focusOnMount(node: HTMLElement) {
    node.focus();
  }
  import type { ArchiveItem, ArchiveScanReport, CategoryItem, LoadOrderItem, XlItem } from '$lib/types';
  import {
    GripVertical,
    ChevronDown,
    ChevronRight,
    CheckCircle2,
    Search,
    Crown,
    ShieldAlert,
    FolderPlus,
    Folder,
    FolderOpen,
    Trash2,
    Eye,
    EyeOff,
    AlertTriangle,
    HelpCircle,
    FileCode,
    Info,
    FolderSearch,
    Copy,
    Power,
    Check
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick, untrack } from 'svelte';

  let {
    archives = $bindable([]),
    gamePath = '',
    scanReport = null
  }: {
    archives: ArchiveItem[];
    gamePath: string;
    scanReport?: ArchiveScanReport | null;
  } = $props();

  let searchQuery = $state('');
  let expandedRows = $state<Record<string, boolean>>({});
  let loadOrderList = $state<LoadOrderItem[]>([]);
  let initialized = $state(false);
  let scrollContainer = $state<HTMLElement | null>(null);
  let showXlHelp = $state(false);

  // Context Menu State
  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    archive: ArchiveItem | null;
    targetType: 'archive' | 'xl';
  }>({
    visible: false,
    x: 0,
    y: 0,
    archive: null,
    targetType: 'archive'
  });

  let copiedFeedback = $state(false);

  const modNodeMap = new Map<string, HTMLElement>();

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

  let showConflictSummary = $state(
    localStorage.getItem('cp2077_show_conflict_summary') !== 'false'
  );
  let highlightedModName = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;

  let activeDragIndex = $state<number | null>(null);
  let dropTargetIndex = $state<number | null>(null);
  let dropPlacement = $state<'before' | 'after' | null>(null);
  let cursorX = $state(0);
  let cursorY = $state(0);

  let scrollSpeed = 0;
  let animationFrameId: number | null = null;

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function toggleSummaryDrawer() {
    showConflictSummary = !showConflictSummary;
    localStorage.setItem('cp2077_show_conflict_summary', showConflictSummary.toString());
  }

  $effect(() => {
    const currentArchives = archives;
    const currentGamePath = gamePath;

    if (currentArchives && currentGamePath) {
      untrack(() => {
        if (!initialized) {
          syncLoadOrderWithCategories(currentArchives, currentGamePath);
        } else {
          reconcileLoadOrderWithArchives(currentArchives);
        }
      });
    }
  });

  async function syncLoadOrderWithCategories(sourceArchives: ArchiveItem[], targetGamePath: string) {
    try {
      const raw = await invoke<string>('load_categories_config', { gamePath: targetGamePath });
      const savedCategories: { id: string; name: string; enabled: boolean; collapsed: boolean; archiveNames: string[] }[] = JSON.parse(raw);

      const unassignedArchives = [...sourceArchives];
      const items: LoadOrderItem[] = [];

      for (const cat of savedCategories) {
        items.push({
          type: 'category',
          category: {
            id: cat.id,
            name: cat.name,
            enabled: cat.enabled,
            collapsed: cat.collapsed,
            isEditing: false
          }
        });

        for (const modName of cat.archiveNames) {
          const idx = unassignedArchives.findIndex(a => a.file_name === modName);
          if (idx !== -1) {
            items.push({ type: 'archive', archive: unassignedArchives.splice(idx, 1)[0] });
          }
        }
      }

      for (const rem of unassignedArchives) {
        items.push({ type: 'archive', archive: rem });
      }

      loadOrderList = items;
      initialized = true;
    } catch {
      loadOrderList = sourceArchives.map(a => ({ type: 'archive', archive: a }));
      initialized = true;
    }
  }

  function reconcileLoadOrderWithArchives(sourceArchives: ArchiveItem[]) {
    const freshMap = new Map(sourceArchives.map(a => [a.file_name, a]));
    const updated: LoadOrderItem[] = [];
    const seen = new Set<string>();

    for (const item of loadOrderList) {
      if (item.type === 'category') {
        updated.push(item);
      } else {
        const fresh = freshMap.get(item.archive.file_name);
        if (fresh) {
          updated.push({ type: 'archive', archive: fresh });
          seen.add(item.archive.file_name);
        }
      }
    }

    for (const archive of sourceArchives) {
      if (!seen.has(archive.file_name)) {
        updated.push({ type: 'archive', archive });
      }
    }

    loadOrderList = updated;
  }

  async function persistState() {
    const archiveNames: string[] = [];
    const categoryLayout: { id: string; name: string; enabled: boolean; collapsed: boolean; archiveNames: string[] }[] = [];
    let currentCat: typeof categoryLayout[0] | null = null;

    for (const item of loadOrderList) {
      if (item.type === 'category') {
        currentCat = {
          id: item.category.id,
          name: item.category.name,
          enabled: item.category.enabled,
          collapsed: item.category.collapsed,
          archiveNames: []
        };
        categoryLayout.push(currentCat);
      } else {
        archiveNames.push(item.archive.file_name);
        if (currentCat) {
          currentCat.archiveNames.push(item.archive.file_name);
        }
      }
    }

    try {
      const report = await invoke<ArchiveScanReport>('save_load_order', {
        gamePath,
        loadOrder: archiveNames
      });
      await invoke('save_categories_config', {
        gamePath,
        configJson: JSON.stringify(categoryLayout)
      });
      if (report && Array.isArray(report.archives)) {
        archives = report.archives;
      }
    } catch (err) {
      console.error('Failed to persist state:', err);
    }
  }

  function addCategory() {
    const newCat: CategoryItem = {
      id: 'cat_' + Date.now(),
      name: 'New Category',
      enabled: true,
      collapsed: false,
      isEditing: true
    };
    loadOrderList = [{ type: 'category', category: newCat }, ...loadOrderList];
    persistState();
  }

  function removeCategory(catId: string) {
    loadOrderList = loadOrderList.filter(item => !(item.type === 'category' && item.category.id === catId));
    persistState();
  }

  async function toggleCategory(cat: CategoryItem) {
    const next = !cat.enabled;
    cat.enabled = next;

    const catIndex = loadOrderList.findIndex(i => i.type === 'category' && i.category.id === cat.id);
    if (catIndex === -1) return;

    for (let i = catIndex + 1; i < loadOrderList.length; i++) {
      const item = loadOrderList[i];
      if (item.type === 'category') break;
      if (item.archive.enabled !== next) {
        item.archive.enabled = next;
        await invoke('toggle_mod_state', {
          gamePath,
          modName: item.archive.file_name,
          enable: next
        });
      }
    }
    persistState();
  }

  async function toggleMod(archive: ArchiveItem) {
    const next = !archive.enabled;
    archive.enabled = next;
    try {
      await invoke('toggle_mod_state', {
        gamePath,
        modName: archive.file_name,
        enable: next
      });
      persistState();
    } catch (err) {
      console.error('Failed to toggle mod:', err);
    }
  }

  async function focusModInMainList(modName: string) {
    let uncollapsed = false;
    let parentCat: CategoryItem | null = null;

    for (const entry of loadOrderList) {
      if (entry.type === 'category') {
        parentCat = entry.category;
      } else if (entry.archive.file_name === modName) {
        if (parentCat && parentCat.collapsed) {
          parentCat.collapsed = false;
          uncollapsed = true;
        }
        break;
      }
    }

    if (uncollapsed) {
      persistState();
    }

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
    } else {
      console.warn('Could not locate DOM node for mod:', modName);
    }
  }

  function processAutoScroll() {
    if (scrollSpeed !== 0 && scrollContainer) {
      scrollContainer.scrollTop += scrollSpeed;
      animationFrameId = requestAnimationFrame(processAutoScroll);
    } else {
      animationFrameId = null;
    }
  }

  function startDrag(event: PointerEvent, index: number) {
    if (searchQuery !== '') return;
    closeContextMenu();
    activeDragIndex = index;
    dropTargetIndex = index;
    dropPlacement = null;
    cursorX = event.clientX;
    cursorY = event.clientY;
    scrollSpeed = 0;
  }

  function onPointerMove(event: PointerEvent) {
    if (activeDragIndex === null) return;
    cursorX = event.clientX;
    cursorY = event.clientY;

    if (!scrollContainer) return;
    const rect = scrollContainer.getBoundingClientRect();
    const topEdge = rect.top + 70;
    const bottomEdge = rect.bottom - 70;

    if (event.clientY < topEdge) {
      const distance = topEdge - event.clientY;
      scrollSpeed = -Math.min(18, Math.max(3, distance / 3));
      if (!animationFrameId) {
        animationFrameId = requestAnimationFrame(processAutoScroll);
      }
    } else if (event.clientY > bottomEdge) {
      const distance = event.clientY - bottomEdge;
      scrollSpeed = Math.min(18, Math.max(3, distance / 3));
      if (!animationFrameId) {
        animationFrameId = requestAnimationFrame(processAutoScroll);
      }
    } else {
      scrollSpeed = 0;
    }
  }

  function onRowPointerMove(event: PointerEvent, index: number) {
    if (activeDragIndex === null) return;
    dropTargetIndex = index;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const midPoint = rect.top + rect.height / 2;
    dropPlacement = event.clientY < midPoint ? 'before' : 'after';
  }

  function completeDrop() {
    scrollSpeed = 0;
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }

    if (activeDragIndex !== null && dropTargetIndex !== null && dropPlacement !== null) {
      let targetIndex = dropTargetIndex;
      if (dropPlacement === 'after') {
        targetIndex += 1;
      }
      if (activeDragIndex < targetIndex) {
        targetIndex -= 1;
      }

      if (activeDragIndex !== targetIndex) {
        const updated = [...loadOrderList];
        const [movedItem] = updated.splice(activeDragIndex, 1);
        updated.splice(targetIndex, 0, movedItem);
        loadOrderList = updated;
        persistState();
      }
    }
    activeDragIndex = null;
    dropTargetIndex = null;
    dropPlacement = null;
  }

  function getCategoryStats(catIndex: number) {
    let total = 0;
    let active = 0;
    for (let i = catIndex + 1; i < loadOrderList.length; i++) {
      const entry = loadOrderList[i];
      if (entry.type === 'category') break;
      total += 1;
      if (entry.archive.enabled) active += 1;
    }
    return { total, active };
  }

  // --- Context Menu Handlers ---
  function openContextMenu(event: MouseEvent, archive: ArchiveItem, targetType: 'archive' | 'xl' = 'archive') {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    // Viewport boundary guard (prevent menu overflowing outside screen)
    const menuWidth = 230;
    const menuHeight = 145;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    contextMenu = {
      visible: true,
      x: posX,
      y: posY,
      archive,
      targetType
    };
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.archive = null;
      copiedFeedback = false;
    }
  }

  function handleContextMenuToggle() {
    if (contextMenu.archive) {
      toggleMod(contextMenu.archive);
    }
    closeContextMenu();
  }

  async function handleContextMenuShowInExplorer() {
    if (!contextMenu.archive || !gamePath) return;

    let targetFilename = contextMenu.archive.file_name;
    if (contextMenu.targetType === 'xl' && contextMenu.archive.associated_xl) {
      targetFilename = contextMenu.archive.associated_xl.file_name;
    }

    const fullPath = `${gamePath}\\archive\\pc\\mod\\${targetFilename}`;
    try {
      await revealItemInDir(fullPath);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeContextMenu();
  }

  async function handleContextMenuCopyName() {
    if (!contextMenu.archive) return;

    let targetFilename = contextMenu.archive.file_name;
    if (contextMenu.targetType === 'xl' && contextMenu.archive.associated_xl) {
      targetFilename = contextMenu.archive.associated_xl.file_name;
    }

    try {
      await navigator.clipboard.writeText(targetFilename);
      copiedFeedback = true;
      setTimeout(() => {
        closeContextMenu();
      }, 400);
    } catch (err) {
      console.error('Failed to copy file name:', err);
      closeContextMenu();
    }
  }

  let visibleItems = $derived.by(() => {
    let currentCategoryCollapsed = false;
    const result: { item: LoadOrderItem; originalIndex: number; archiveRank: number }[] = [];
    let rank = 0;

    for (let i = 0; i < loadOrderList.length; i++) {
      const entry = loadOrderList[i];

      if (entry.type === 'category') {
        currentCategoryCollapsed = entry.category.collapsed;
        if (searchQuery === '' || entry.category.name.toLowerCase().includes(searchQuery.toLowerCase())) {
          result.push({ item: entry, originalIndex: i, archiveRank: 0 });
        }
      } else {
        rank += 1;
        if (!currentCategoryCollapsed || searchQuery !== '') {
          if (searchQuery === '' || entry.archive.name.toLowerCase().includes(searchQuery.toLowerCase())) {
            result.push({ item: entry, originalIndex: i, archiveRank: rank });
          }
        }
      }
    }
    return result;
  });

  let conflictingArchives = $derived(
    archives.filter(a => a.wins.length > 0 || a.loses.length > 0)
  );

  let draggedEntry = $derived(
    activeDragIndex !== null ? loadOrderList[activeDragIndex] : null
  );

  let unassociatedXlFiles = $derived(
    (scanReport?.unassociated_xl ?? []).filter(item => {
      if (!searchQuery.trim()) return true;
      return item.file_name.toLowerCase().includes(searchQuery.toLowerCase());
    })
  );
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={completeDrop}
  onclick={closeContextMenu}
/>

{#if activeDragIndex !== null && draggedEntry}
  <div
    class="fixed pointer-events-none z-50 px-3 py-1.5 rounded bg-[#1e2328] border border-[#76b900] shadow-2xl shadow-black/95 flex items-center gap-2.5 backdrop-blur-sm -translate-x-4 -translate-y-6"
    style="left: {cursorX}px; top: {cursorY}px;"
  >
    <GripVertical class="h-3.5 w-3.5 text-[#76b900]" />
    {#if draggedEntry.type === 'category'}
      <Folder class="h-3.5 w-3.5 text-[#76b900]" />
      <span class="text-xs font-bold text-white uppercase tracking-wider">{draggedEntry.category.name}</span>
    {:else}
      <span class="text-xs font-mono font-semibold text-white">{draggedEntry.archive.file_name}</span>
    {/if}
  </div>
{/if}

<!-- Custom Context Menu -->
{#if contextMenu.visible && contextMenu.archive}
  {@const activeTargetName = (contextMenu.targetType === 'xl' && contextMenu.archive.associated_xl) ? contextMenu.archive.associated_xl.file_name : contextMenu.archive.file_name}
  <div
    class="fixed z-50 w-56 rounded-md border border-[#303841] bg-[#161a1e] py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <!-- Header with Monospace Filename -->
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-card/40 flex items-center justify-between gap-2">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.targetType === 'xl'}
          <FileCode class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
        {:else}
          <div class="h-2 w-2 rounded-full shrink-0 {contextMenu.archive.enabled ? 'bg-nvidia-accent' : 'bg-red-500'}"></div>
        {/if}
        <span class="font-mono text-[11px] font-bold text-white truncate" title={activeTargetName}>
          {activeTargetName}
        </span>
      </div>
      <span class="text-[9px] font-mono uppercase px-1 py-0.2 rounded border {contextMenu.archive.enabled ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-red-500/15 border-red-500/40 text-red-400'} shrink-0">
        {contextMenu.archive.enabled ? 'Active' : 'Disabled'}
      </span>
    </div>

    <!-- Actions -->
    <div class="p-1 space-y-0.5">
      <!-- Show in Explorer -->
      <button
        type="button"
        onclick={handleContextMenuShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-gray-200 hover:text-white transition text-left cursor-pointer group"
      >
        <FolderSearch class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Explorer</span>
      </button>

      <!-- Enable / Disable (Mirrors Switch) -->
      <button
        type="button"
        onclick={handleContextMenuToggle}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-gray-200 hover:text-white transition text-left cursor-pointer group"
      >
        <Power class="h-3.5 w-3.5 {contextMenu.archive.enabled ? 'text-amber-400' : 'text-nvidia-accent'}" />
        <span>{contextMenu.archive.enabled ? 'Disable Mod' : 'Enable Mod'}</span>
      </button>

      <!-- Copy File Name -->
      <button
        type="button"
        onclick={handleContextMenuCopyName}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-gray-200 hover:text-white transition text-left cursor-pointer group"
      >
        <div class="flex items-center gap-2.5">
          {#if copiedFeedback}
            <Check class="h-3.5 w-3.5 text-nvidia-accent" />
            <span class="text-nvidia-accent font-medium">Copied!</span>
          {:else}
            <Copy class="h-3.5 w-3.5 text-gray-400 group-hover:text-white" />
            <span>Copy File Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="space-y-3 select-none flex flex-col h-full">
  <div class="flex items-center justify-between gap-4 shrink-0">
    <div class="relative flex-1 max-w-md">
      <Search class="h-4 w-4 absolute left-3 top-1/2 -translate-y-1/2 text-nvidia-text-muted" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Filter mods or categories..."
        class="w-full pl-9 pr-4 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-white placeholder-nvidia-text-muted focus:outline-none focus:border-nvidia-accent"
      />
    </div>

    <div class="flex items-center gap-2.5">
      <button
        onclick={addCategory}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-gray-200 font-medium transition"
      >
        <FolderPlus class="h-3.5 w-3.5 text-nvidia-accent" />
        <span>Add Category</span>
      </button>

      <button
        onclick={toggleSummaryDrawer}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded border text-xs font-medium transition {showConflictSummary ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-nvidia-surface hover:bg-nvidia-card border-nvidia-border text-nvidia-text-muted hover:text-white'}"
        title="Toggle Conflicting Mod Summary Column"
      >
        {#if showConflictSummary}
          <Eye class="h-3.5 w-3.5" />
        {:else}
          <EyeOff class="h-3.5 w-3.5" />
        {/if}
        <span>Conflicting Mod Summary</span>
      </button>

      <span class="text-xs font-mono text-gray-400 pl-2">Mods: {archives.length}</span>
    </div>
  </div>

  <div class="flex gap-3 flex-1 overflow-hidden min-h-0">
    <div
      bind:this={scrollContainer}
      class="space-y-1 overflow-y-auto pr-1 flex-1 max-h-[calc(100vh-170px)]"
    >
      {#if visibleItems.length === 0}
        <div class="p-6 text-center text-xs text-nvidia-text-muted border border-nvidia-border rounded bg-nvidia-surface">
          No matches found.
        </div>
      {:else}
        {#each visibleItems as { item, originalIndex, archiveRank } (item.type === 'category' ? item.category.id : item.archive.file_name)}
          {@const isSource = activeDragIndex === originalIndex}
          {@const showLineBefore = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'before' && activeDragIndex !== originalIndex && activeDragIndex !== originalIndex - 1}
          {@const showLineAfter = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'after' && activeDragIndex !== originalIndex && activeDragIndex !== originalIndex + 1}

          <div class="relative flex flex-col">
            {#if showLineBefore}
              <div class="absolute -top-1 left-0 right-0 h-0.5 bg-nvidia-accent z-30 shadow-[0_0_8px_rgba(118,185,0,0.8)] flex items-center">
                <div class="h-2 w-2 rounded-full bg-nvidia-accent -ml-1"></div>
              </div>
            {/if}

            {#if item.type === 'category'}
              {@const stats = getCategoryStats(originalIndex)}
              <div
                role="group"
                aria-label="Mod Category: {item.category.name}"
                onpointermove={(e) => onRowPointerMove(e, originalIndex)}
                class="rounded-lg border border-nvidia-border bg-gradient-to-r from-nvidia-card via-[#161a1e] to-nvidia-surface transition-colors px-3 py-1.5 flex items-center justify-between h-10 mt-3 relative overflow-hidden shadow-sm {isSource ? 'opacity-20 border-dashed border-nvidia-accent' : ''}"
              >
                <div class="absolute left-0 top-0 bottom-0 w-1 bg-nvidia-accent"></div>

                <div class="flex items-center gap-2.5 flex-1 min-w-0 pl-1">
                  <div
                    role="button"
                    tabindex="0"
                    aria-label="Drag entire category"
                    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                    onpointerdown={(e) => { e.stopPropagation(); startDrag(e, originalIndex); }}
                    class="cursor-grab active:cursor-grabbing text-nvidia-text-muted hover:text-white shrink-0 p-1"
                    title="Drag entire category"
                  >
                    <GripVertical class="h-3.5 w-3.5" />
                  </div>

                  <button
                    onclick={() => { item.category.collapsed = !item.category.collapsed; persistState(); }}
                    class="text-nvidia-accent hover:text-nvidia-accent-hover transition shrink-0 p-1 rounded hover:bg-nvidia-surface/60"
                    title={item.category.collapsed ? "Expand category" : "Collapse category"}
                  >
                    {#if item.category.collapsed}
                      <Folder class="h-4 w-4" />
                    {:else}
                      <FolderOpen class="h-4 w-4" />
                    {/if}
                  </button>

                  <button
                    onclick={() => toggleCategory(item.category)}
                    class="w-7 h-4 rounded-full transition relative p-0.5 shrink-0 {item.category.enabled ? 'bg-nvidia-accent' : 'bg-nvidia-border'}"
                    title="Batch toggle all mods in category"
                  >
                    <div class="h-3 w-3 rounded-full bg-black transition transform {item.category.enabled ? 'translate-x-3' : 'translate-x-0'}"></div>
                  </button>

                  {#if item.category.isEditing}
                    <input
                      type="text"
                      bind:value={item.category.name}
                      onblur={() => { item.category.isEditing = false; persistState(); }}
                      onkeydown={(e) => { if (e.key === 'Enter') { item.category.isEditing = false; persistState(); }}}
                      class="bg-nvidia-bg border border-nvidia-accent rounded px-2 py-0.5 text-xs text-white font-bold uppercase tracking-wider focus:outline-none"
                      use:focusOnMount
                    />
                  {:else}
                    <button
                      onclick={() => item.category.isEditing = true}
                      class="text-xs font-bold text-gray-100 uppercase tracking-wider hover:text-nvidia-accent transition truncate flex items-center gap-2"
                    >
                      <span>{item.category.name}</span>
                    </button>
                  {/if}

                  <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nvidia-surface border border-nvidia-border text-nvidia-text-muted shrink-0">
                    {stats.active}/{stats.total} Active
                  </span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <button
                    onclick={() => removeCategory(item.category.id)}
                    class="p-1 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-red-400 transition"
                    title="Delete category container"
                  >
                    <Trash2 class="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>

            {:else}
              {@const archive = item.archive}
              {@const isExpanded = !!expandedRows[archive.file_name]}
              {@const isHighlighted = highlightedModName === archive.file_name}

              <div
                role="group"
                aria-label="Archive mod item: {archive.file_name}"
                use:registerModNode={archive.file_name}
                onpointermove={(e) => onRowPointerMove(e, originalIndex)}
                oncontextmenu={(e) => openContextMenu(e, archive, 'archive')}
                class="rounded border transition-all duration-300 {isHighlighted ? 'border-[#76b900] ring-2 ring-[#76b900] bg-[#76b900]/20 shadow-[0_0_15px_rgba(118,185,0,0.35)] scale-[1.008]' : 'border-nvidia-border/70 bg-nvidia-surface hover:border-nvidia-border'} {archive.enabled ? 'opacity-100' : 'opacity-40'} {isSource ? 'opacity-20 border-dashed border-nvidia-accent/50' : ''}"
              >
                <div class="flex items-center justify-between px-3 py-1.5 gap-2 h-9">
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <div
                      role="button"
                      tabindex="0"
                      aria-label="Drag to adjust load order"
                      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                      onpointerdown={(e) => { e.stopPropagation(); startDrag(e, originalIndex); }}
                      class="cursor-grab active:cursor-grabbing p-1 text-nvidia-text-muted hover:text-nvidia-accent shrink-0 rounded hover:bg-nvidia-card transition"
                      title="Drag to adjust load order"
                    >
                      <GripVertical class="h-3.5 w-3.5" />
                    </div>

                    <button
                      onclick={() => toggleMod(archive)}
                      aria-label={archive.enabled ? "Disable mod " + archive.file_name : "Enable mod " + archive.file_name}
                      class="w-7 h-4 rounded-full transition relative p-0.5 shrink-0 {archive.enabled ? 'bg-nvidia-accent' : 'bg-nvidia-border'}"
                    >
                      <div class="h-3 w-3 rounded-full bg-black transition transform {archive.enabled ? 'translate-x-3': 'translate-x-0'}"></div>
                    </button>

                    <span class="text-[10px] font-mono px-1 py-0.2 rounded bg-nvidia-card text-nvidia-text-muted border border-nvidia-border shrink-0">
                      #{archiveRank}
                    </span>

                    <span class="text-xs font-mono font-medium text-white truncate max-w-md">
                      {archive.file_name}
                    </span>

                    <!-- Flat Inline XL Badge with Isolated Context Menu Trigger -->
                    {#if archive.associated_xl}
                      <span
                        role="button"
                        tabindex="0"
                        oncontextmenu={(e) => openContextMenu(e, archive, 'xl')}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                        class="text-[10px] font-mono font-bold px-1.5 py-0.2 rounded border transition-colors shrink-0 cursor-context-menu {archive.enabled ? 'bg-cyan-950/80 text-cyan-400 border-cyan-700/60 hover:border-cyan-500' : 'bg-nvidia-card text-nvidia-text-muted border-nvidia-border opacity-50'}"
                        title="Companion: {archive.associated_xl.file_name} ({formatBytes(archive.associated_xl.size_bytes)}) - Right-click for options"
                      >
                        XL
                      </span>
                    {/if}

                    <div class="flex items-center gap-1.5 text-[11px] text-nvidia-text-muted shrink-0">
                      <span>({archive.file_count} assets)</span>
                      <span>•</span>
                      <span>{formatBytes(archive.size_bytes)}</span>
                    </div>
                  </div>

                  <div class="flex items-center gap-2 shrink-0">
                    {#if archive.wins.length > 0}
                      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-nvidia-accent/10 border border-nvidia-accent/30 text-nvidia-accent text-[11px] font-medium">
                        <Crown class="h-3 w-3" />
                        <span>Winning ({archive.wins.length})</span>
                      </div>
                    {/if}

                    {#if archive.loses.length > 0}
                      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-amber-500/10 border border-amber-500/30 text-amber-400 text-[11px] font-medium">
                        <ShieldAlert class="h-3 w-3" />
                        <span>Overridden ({archive.loses.length})</span>
                      </div>
                    {/if}

                    {#if archive.wins.length === 0 && archive.loses.length === 0}
                      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-nvidia-card border border-nvidia-border text-nvidia-text-muted text-[11px]">
                        <CheckCircle2 class="h-3 w-3 text-nvidia-accent" />
                        <span>Clean</span>
                      </div>
                    {/if}

                    {#if archive.has_conflicts}
                      <button
                        onclick={() => { expandedRows[archive.file_name] = !expandedRows[archive.file_name]; }}
                        class="p-1 rounded hover:bg-nvidia-card text-nvidia-text-muted hover:text-white transition"
                      >
                        {#if isExpanded}
                          <ChevronDown class="h-3.5 w-3.5" />
                        {:else}
                          <ChevronRight class="h-3.5 w-3.5" />
                        {/if}
                      </button>
                    {:else}
                      <div class="w-5.5"></div>
                    {/if}
                  </div>
                </div>

                {#if isExpanded && archive.has_conflicts}
                  <div class="px-4 py-2 border-t border-nvidia-border/60 bg-nvidia-card/30 space-y-2 text-xs">
                    {#if archive.wins.length > 0}
                      <div>
                        <span class="text-[10px] font-semibold text-nvidia-accent uppercase tracking-wider">Overwrites Lower Mods:</span>
                        <div class="mt-1 flex flex-wrap gap-1">
                          {#each archive.wins as target}
                            <span class="px-1.5 py-0.2 rounded bg-nvidia-surface border border-nvidia-border text-[10px] font-mono text-gray-300">
                              {target}
                            </span>
                          {/each}
                        </div>
                      </div>
                    {/if}

                    {#if archive.loses.length > 0}
                      <div>
                        <span class="text-[10px] font-semibold text-amber-400 uppercase tracking-wider">Loses To Higher Mods:</span>
                        <div class="mt-1 flex flex-wrap gap-1">
                          {#each archive.loses as target}
                            <span class="px-1.5 py-0.2 rounded bg-amber-500/10 border border-amber-500/30 text-[10px] font-mono text-amber-300">
                              {target}
                            </span>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}

            {#if showLineAfter}
              <div class="absolute -bottom-1 left-0 right-0 h-0.5 bg-nvidia-accent z-30 shadow-[0_0_8px_rgba(118,185,0,0.8)] flex items-center">
                <div class="h-2 w-2 rounded-full bg-nvidia-accent -ml-1"></div>
              </div>
            {/if}
          </div>
        {/each}
      {/if}

      <!-- Dedicated Bottom Section: Unassociated .xl Files -->
      {#if unassociatedXlFiles.length > 0}
        <div class="mt-6 pt-4 border-t border-nvidia-border/60 space-y-2">
          <!-- Section Header with Help Trigger -->
          <div class="rounded-lg border border-cyan-900/40 bg-gradient-to-r from-[#0e171f] via-[#101923] to-nvidia-surface px-3 py-2 flex items-center justify-between">
            <div class="flex items-center gap-2 min-w-0">
              <FileCode class="h-4 w-4 text-cyan-400 shrink-0" />
              <span class="text-xs font-bold text-cyan-300 uppercase tracking-wider">
                Unassociated .xl Files
              </span>
              <span class="text-[10px] text-gray-400 hidden sm:inline">
                (Active in game, without a systemically generated mod association)
              </span>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <button
                type="button"
                onclick={() => showXlHelp = !showXlHelp}
                class="flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium border transition-colors {showXlHelp ? 'bg-cyan-950 text-cyan-300 border-cyan-600' : 'bg-nvidia-surface hover:bg-nvidia-card text-cyan-400 border-cyan-900/60'}"
                title="Click for association instructions"
              >
                <HelpCircle class="h-3.5 w-3.5" />
                <span>Help</span>
              </button>

              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nvidia-surface border border-nvidia-border text-cyan-400">
                {unassociatedXlFiles.length} loose
              </span>
            </div>
          </div>

          <!-- Explanatory Help Card -->
          {#if showXlHelp}
            <div class="p-3 rounded-lg border border-cyan-800/50 bg-cyan-950/20 text-xs text-gray-300 space-y-1.5">
              <div class="flex items-center gap-2 text-cyan-300 font-semibold">
                <Info class="h-4 w-4 shrink-0" />
                <span>How to link an unassociated .xl file to its parent .archive mod:</span>
              </div>
              <p class="text-[11px] text-gray-400 pl-6 leading-relaxed">
                1. Open your mod folder in Windows Explorer: <span class="font-mono text-gray-200">{gamePath ? gamePath + '\\archive\\pc\\mod' : '\\archive\\pc\\mod'}</span>.
                <br />
                2. Rename the <span class="font-mono text-cyan-300">.xl</span> file to match its parent archive name exactly:
                <br />
                &nbsp;&nbsp;&nbsp;• Standard: <span class="font-mono text-gray-200">&lt;ModName&gt;.archive.xl</span> or <span class="font-mono text-gray-200">&lt;ModName&gt;.xl</span>
                <br />
                3. Click <span class="font-semibold text-white">Run Conflict Scan</span> at the top of the app. The file will automatically link to the archive, display the <span class="font-mono text-cyan-400 font-bold">[XL]</span> badge, and synchronize its toggle state.
              </p>
            </div>
          {/if}

          <!-- Read-only List of Unassociated Files -->
          <div class="space-y-1">
            {#each unassociatedXlFiles as xl (xl.file_name)}
              <div class="rounded border border-cyan-950/60 bg-nvidia-surface/80 px-3 py-1.5 flex items-center justify-between h-8">
                <div class="flex items-center gap-2.5 min-w-0">
                  <FileCode class="h-3.5 w-3.5 text-cyan-500 shrink-0" />
                  <span class="text-xs font-mono text-cyan-200 truncate">{xl.file_name}</span>
                  <span class="text-[11px] font-mono text-nvidia-text-muted shrink-0">
                    {formatBytes(xl.size_bytes)}
                  </span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-cyan-950/60 border border-cyan-800/40 text-cyan-300">
                    Active / Untethered
                  </span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    {#if showConflictSummary}
      <aside class="w-80 rounded-lg border border-nvidia-border bg-nvidia-surface flex flex-col overflow-hidden shrink-0 shadow-xl transition-all duration-200">
        <div class="p-3 border-b border-nvidia-border bg-nvidia-card/50 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <AlertTriangle class="h-4 w-4 text-amber-400" />
            <span class="text-xs font-bold text-white uppercase tracking-wider">Conflicting Summary</span>
          </div>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nvidia-surface border border-nvidia-border text-gray-300">
            {conflictingArchives.length} Contested
          </span>
        </div>

        <div class="p-2 space-y-1 overflow-y-auto flex-1 max-h-[calc(100vh-220px)]">
          {#if conflictingArchives.length === 0}
            <div class="p-6 text-center text-xs text-nvidia-text-muted">
              No load order conflicts detected.
            </div>
          {:else}
            {#each conflictingArchives as archive (archive.file_name)}
              {@const isLosing = archive.loses.length > 0}
              <button
                type="button"
                onclick={() => focusModInMainList(archive.file_name)}
                class="w-full text-left rounded p-2 flex items-center justify-between gap-2 border transition cursor-pointer {isLosing ? 'border-amber-500/30 bg-amber-500/5 hover:bg-amber-500/15' : 'border-nvidia-accent/30 bg-nvidia-accent/5 hover:bg-nvidia-accent/15'} hover:border-nvidia-accent group"
                title="Click to locate in main load order"
              >
                <div class="flex items-center gap-2 min-w-0 flex-1">
                  <div class="h-2 w-2 rounded-full shrink-0 {isLosing ? 'bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.6)]' : 'bg-nvidia-accent shadow-[0_0_6px_rgba(118,185,0,0.6)]'}"></div>

                  <span class="text-xs font-mono text-gray-200 group-hover:text-white truncate">
                    {archive.file_name}
                  </span>
                </div>

                <div class="shrink-0">
                  {#if isLosing}
                    <span class="text-[10px] font-mono font-semibold px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-400 border border-amber-500/30">
                      -{archive.loses.length}
                    </span>
                  {:else}
                    <span class="text-[10px] font-mono font-semibold px-1.5 py-0.5 rounded bg-nvidia-accent/20 text-nvidia-accent border border-nvidia-accent/30">
                      +{archive.wins.length}
                    </span>
                  {/if}
                </div>
              </button>
            {/each}
          {/if}
        </div>

        <div class="p-2.5 border-t border-nvidia-border bg-nvidia-card/30 text-[11px] text-nvidia-text-muted flex items-center justify-center">
          <span>Click any mod to locate in main load order</span>
        </div>
      </aside>
    {/if}
  </div>
</div>