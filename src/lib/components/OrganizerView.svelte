<script lang="ts">
  import type { ArchiveItem, LabelsConfig, CategoryLabel } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { 
    Plus, 
    Pencil, 
    Trash2, 
    GripVertical, 
    FileArchive, 
    LayoutGrid, 
    Tag, 
    ListTree, 
    FolderSearch, 
    FolderInput, 
    X, 
    ChevronDown, 
    ChevronRight, 
    Folder, 
    FolderOpen,
    Archive,
    RotateCcw,
    Package,
    Search
  } from 'lucide-svelte';
  import CategoryModal from './CategoryModal.svelte';
  import DialogModal from './DialogModal.svelte';
  import MoveCategoryModal from './MoveCategoryModal.svelte';

  let {
    archives = [],
    labelsConfig = $bindable({ categories: [], assignments: {}, orders: {} }),
    gamePath = '',
    showCategoryLabelsInLoadOrder = $bindable(true),
    onLabelsToggle,
    showUncategorizedDrawer = true,
    searchQuery = '',
    onShowInLoadOrder,
    onRouteToHistory
  }: {
    archives: ArchiveItem[];
    labelsConfig: LabelsConfig;
    gamePath: string;
    showCategoryLabelsInLoadOrder?: boolean;
    onLabelsToggle?: (val: boolean) => void;
    showUncategorizedDrawer?: boolean;
    searchQuery?: string;
    onShowInLoadOrder?: (modName: string) => void;
    onRouteToHistory?: (modName: string) => void;
  } = $props();

  let isCatModalOpen = $state(false);
  let catModalMode = $state<'create' | 'edit'>('create');
  let catModalData = $state({ id: '', name: '', color: '' });

  let isDeleteModalOpen = $state(false);
  let catToDelete = $state<CategoryLabel | null>(null);

  let isMoveCatModalOpen = $state(false);

  // --- Collapse State ---
  let collapsedCategories = $state<Record<string, boolean>>({});
  
  let activeCategories = $derived(labelsConfig.categories.filter(c => !c.stashed));
  let stashedCategories = $derived(labelsConfig.categories.filter(c => c.stashed));

  let totalActiveCategories = $derived(activeCategories.length);
  let collapsedCount = $derived(activeCategories.filter(c => collapsedCategories[c.id]).length);
  let isAllCollapsed = $derived(totalActiveCategories > 0 && collapsedCount === totalActiveCategories);

  function toggleAllCategories() {
    if (isAllCollapsed) {
      collapsedCategories = {};
    } else {
      const newState: Record<string, boolean> = {};
      for (const cat of activeCategories) {
        newState[cat.id] = true;
      }
      collapsedCategories = newState;
    }
  }

  function toggleCategoryCollapse(catId: string) {
    collapsedCategories[catId] = !collapsedCategories[catId];
  }

  // --- Search Engine Logic ---
  let isSearching = $derived(searchQuery.trim().length > 0);
  let normalizedSearch = $derived(searchQuery.trim().toLowerCase());

  function doesModMatch(fileName: string): boolean {
    if (!isSearching) return false;
    return fileName.toLowerCase().includes(normalizedSearch);
  }

  function doesCategoryMatch(cat: CategoryLabel, mods: ArchiveItem[]): boolean {
    if (!isSearching) return true;
    if (cat.name.toLowerCase().includes(normalizedSearch)) return true;
    return mods.some(m => m.file_name.toLowerCase().includes(normalizedSearch));
  }

  $effect(() => {
    if (isSearching) {
      const newCollapsed = { ...collapsedCategories };
      for (const cat of activeCategories) {
        const mods = sortedCategorizedMods.get(cat.id) || [];
        if (doesCategoryMatch(cat, mods)) {
          newCollapsed[cat.id] = false; // Auto-expand matching categories
        }
      }
      collapsedCategories = newCollapsed;
    }
  });

  // Derived state for rendering
  let actualMods = $derived(archives.filter(a => !a.is_delimiter));
  
  let sortedUncategorizedMods = $derived.by(() => {
    const mods = actualMods.filter(a => !labelsConfig.assignments[a.file_name]);
    const order = labelsConfig.orders?.['uncategorized'] || [];
    mods.sort((a, b) => {
      const idxA = order.indexOf(a.file_name);
      const idxB = order.indexOf(b.file_name);
      if (idxA !== -1 && idxB !== -1) return idxA - idxB;
      if (idxA !== -1) return -1;
      if (idxB !== -1) return 1;
      return a.file_name.localeCompare(b.file_name);
    });
    return mods;
  });

  let sortedCategorizedMods = $derived.by(() => {
    const map = new Map<string, ArchiveItem[]>();
    for (const cat of labelsConfig.categories) {
      map.set(cat.id, []);
    }
    for (const mod of actualMods) {
      const catId = labelsConfig.assignments[mod.file_name];
      if (catId && map.has(catId)) {
        map.get(catId)!.push(mod);
      }
    }
    for (const [catId, mods] of map.entries()) {
      const order = labelsConfig.orders?.[catId] || [];
      mods.sort((a, b) => {
        const idxA = order.indexOf(a.file_name);
        const idxB = order.indexOf(b.file_name);
        if (idxA !== -1 && idxB !== -1) return idxA - idxB;
        if (idxA !== -1) return -1;
        if (idxB !== -1) return 1;
        return a.file_name.localeCompare(b.file_name);
      });
    }
    return map;
  });

  let allVisibleMods = $derived([...sortedUncategorizedMods, ...Array.from(sortedCategorizedMods.values()).flat()]);

  // --- Custom Pointer DnD Engine State ---
  let selectedMods = $state<Set<string>>(new Set());
  let lastSelectedMod = $state<string | null>(null);

  let isPointerDown = $state(false);
  let isDragging = $state(false);
  let startX = $state(0);
  let startY = $state(0);
  let currentX = $state(0);
  let currentY = $state(0);
  
  // Mod Drag State
  let activeDropTarget = $state<string | null>(null);
  let dropTargetMod = $state<string | null>(null);
  let dropPlacement = $state<'before' | 'after' | null>(null);

  // Category Drag State
  let activeCategoryDragId = $state<string | null>(null);
  let categoryDropTargetId = $state<string | null>(null);
  let categoryDropPlacement = $state<'before' | 'after' | null>(null);

  // --- Context Menu State ---
  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    modName: string | null;
  }>({ visible: false, x: 0, y: 0, modName: null });

  async function saveConfig() {
    try {
      await invoke('save_labels_config', { gamePath, config: labelsConfig });
    } catch (err) {
      console.error('Failed to save labels config:', err);
    }
  }

  // --- Category Shelving / Stashing ---
  function toggleStashCategory(cat: CategoryLabel) {
    cat.stashed = !cat.stashed;
    labelsConfig.categories = [...labelsConfig.categories];
    saveConfig();
  }

  // --- Category CRUD ---
  function openCreateModal() {
    catModalMode = 'create';
    catModalData = { id: '', name: '', color: '#06B6D4' };
    isCatModalOpen = true;
  }

  function openEditModal(cat: CategoryLabel) {
    catModalMode = 'edit';
    catModalData = { id: cat.id, name: cat.name, color: cat.color };
    isCatModalOpen = true;
  }

  function handleCatModalConfirm(data: { id: string; name: string; color: string }) {
    if (catModalMode === 'create') {
      labelsConfig.categories = [...labelsConfig.categories, { ...data, stashed: false }];
    } else {
      const idx = labelsConfig.categories.findIndex(c => c.id === data.id);
      if (idx !== -1) {
        labelsConfig.categories[idx] = { ...labelsConfig.categories[idx], ...data };
      }
    }
    saveConfig();
  }

  function confirmDelete(cat: CategoryLabel) {
    catToDelete = cat;
    isDeleteModalOpen = true;
  }

  function executeDelete() {
    if (!catToDelete) return;
    
    labelsConfig.categories = labelsConfig.categories.filter(c => c.id !== catToDelete!.id);
    
    const newAssignments = { ...labelsConfig.assignments };
    for (const [modName, catId] of Object.entries(newAssignments)) {
      if (catId === catToDelete.id) {
        delete newAssignments[modName];
      }
    }
    labelsConfig.assignments = newAssignments;
    
    saveConfig();
    isDeleteModalOpen = false;
    catToDelete = null;
  }

  // --- Context Menu Logic ---
  function openContextMenu(e: MouseEvent, modName: string) {
    e.preventDefault();
    e.stopPropagation();
    
    if (!selectedMods.has(modName)) {
      selectedMods.clear();
      selectedMods.add(modName);
      selectedMods = new Set(selectedMods);
      lastSelectedMod = modName;
    }

    const menuWidth = 200;
    const menuHeight = 160;
    const posX = (e.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (e.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;
    
    contextMenu = { visible: true, x: posX, y: posY, modName };
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.modName = null;
    }
  }

  async function handleContextMenuShowInExplorer() {
    if (!contextMenu.modName || !gamePath) return;
    const fullPath = `${gamePath}\\archive\\pc\\mod\\${contextMenu.modName}`;
    try {
      await revealItemInDir(fullPath);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeContextMenu();
  }

  function handleContextMenuShowInLoadOrder() {
    if (contextMenu.modName && onShowInLoadOrder) {
      onShowInLoadOrder(contextMenu.modName);
    }
    closeContextMenu();
  }

  function handleContextMenuRemoveFromCategory() {
    const newAssignments = { ...labelsConfig.assignments };
    for (const mod of selectedMods) {
      delete newAssignments[mod];
    }
    labelsConfig.assignments = newAssignments;
    saveConfig();
    closeContextMenu();
  }

  function handleMoveToCategory(actionOrId: string) {
    if (actionOrId === '[CREATE_NEW]') {
      isMoveCatModalOpen = false;
      openCreateModal();
      return;
    }

    if (actionOrId === '[REMOVE]') {
      handleContextMenuRemoveFromCategory();
      isMoveCatModalOpen = false;
      return;
    }

    const newAssignments = { ...labelsConfig.assignments };
    for (const mod of selectedMods) {
      newAssignments[mod] = actionOrId;
    }
    labelsConfig.assignments = newAssignments;
    saveConfig();
    isMoveCatModalOpen = false;
  }

  // --- Custom Pointer DnD Engine Logic ---
  function handleCategoryPointerDown(e: PointerEvent, catId: string) {
    if (e.button !== 0) return;
    e.stopPropagation();
    isPointerDown = true;
    activeCategoryDragId = catId;
    startX = e.clientX;
    startY = e.clientY;
    currentX = e.clientX;
    currentY = e.clientY;
  }

  function handlePointerDown(e: PointerEvent, modName: string) {
    if (e.button !== 0) return;
    
    const isCtrl = e.ctrlKey || e.metaKey;
    const isShift = e.shiftKey;

    if (isShift && lastSelectedMod) {
      const startIdx = allVisibleMods.findIndex(m => m.file_name === lastSelectedMod);
      const endIdx = allVisibleMods.findIndex(m => m.file_name === modName);
      if (startIdx !== -1 && endIdx !== -1) {
        const min = Math.min(startIdx, endIdx);
        const max = Math.max(startIdx, endIdx);
        if (!isCtrl) selectedMods.clear();
        for (let i = min; i <= max; i++) {
          selectedMods.add(allVisibleMods[i].file_name);
        }
      }
    } else if (isCtrl) {
      if (selectedMods.has(modName)) {
        selectedMods.delete(modName);
      } else {
        selectedMods.add(modName);
      }
      lastSelectedMod = modName;
    } else {
      if (!selectedMods.has(modName)) {
        selectedMods.clear();
        selectedMods.add(modName);
      }
      lastSelectedMod = modName;
    }
    selectedMods = new Set(selectedMods);

    isPointerDown = true;
    startX = e.clientX;
    startY = e.clientY;
    currentX = e.clientX;
    currentY = e.clientY;
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isPointerDown) return;

    if (!isDragging) {
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (Math.sqrt(dx * dx + dy * dy) > 5) {
        isDragging = true;
      }
    }

    if (isDragging) {
      currentX = e.clientX;
      currentY = e.clientY;

      if (activeCategoryDragId) {
        // 1. Check if hovering over The Stash drawer
        const stashEl = document.getElementById('stash-drawer');
        let isOverStash = false;
        if (stashEl) {
          const stashRect = stashEl.getBoundingClientRect();
          if (
            e.clientX >= stashRect.left &&
            e.clientX <= stashRect.right &&
            e.clientY >= stashRect.top &&
            e.clientY <= stashRect.bottom
          ) {
            isOverStash = true;
          }
        }

        if (isOverStash) {
          categoryDropTargetId = 'stash';
          categoryDropPlacement = null;
        } else {
          // 2. 1D Distance-based Active Category Reordering
          const catElements = Array.from(document.querySelectorAll('.organizer-category-header'));
          let closestCat: Element | null = null;
          let minDistance = Infinity;
          
          for (const el of catElements) {
            const rect = el.getBoundingClientRect();
            const midY = rect.top + rect.height / 2;
            const dist = Math.abs(e.clientY - midY);
            if (dist < minDistance) {
              minDistance = dist;
              closestCat = el;
            }
          }
          
          if (closestCat) {
            const targetId = closestCat.getAttribute('data-category-id');
            if (targetId && targetId !== activeCategoryDragId) {
              categoryDropTargetId = targetId;
              const rect = closestCat.getBoundingClientRect();
              categoryDropPlacement = e.clientY < (rect.top + rect.height / 2) ? 'before' : 'after';
            } else {
              categoryDropTargetId = null;
              categoryDropPlacement = null;
            }
          }
        }
      } else {
        const el = document.elementFromPoint(e.clientX, e.clientY);
        const targetContainer = el?.closest('[data-category-id]');
        activeDropTarget = targetContainer ? targetContainer.getAttribute('data-category-id') : null;

        const modEl = el?.closest('[data-mod-name]');
        if (modEl && activeDropTarget) {
          const rect = modEl.getBoundingClientRect();
          const mid = rect.top + rect.height / 2;
          dropPlacement = e.clientY < mid ? 'before' : 'after';
          dropTargetMod = modEl.getAttribute('data-mod-name');
        } else {
          dropTargetMod = null;
          dropPlacement = null;
        }
      }
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isPointerDown) return;

    if (isDragging) {
      if (activeCategoryDragId) {
        if (categoryDropTargetId === 'stash') {
          // Shelve category to The Stash
          const cat = labelsConfig.categories.find(c => c.id === activeCategoryDragId);
          if (cat) {
            cat.stashed = true;
            labelsConfig.categories = [...labelsConfig.categories];
            saveConfig();
          }
        } else if (categoryDropTargetId && categoryDropPlacement) {
          // Restore / Reorder category in active list
          const cats = [...labelsConfig.categories];
          const dragIdx = cats.findIndex(c => c.id === activeCategoryDragId);
          const targetIdx = cats.findIndex(c => c.id === categoryDropTargetId);
          
          if (dragIdx !== -1 && targetIdx !== -1) {
            const [draggedCat] = cats.splice(dragIdx, 1);
            draggedCat.stashed = false; // Automatically un-stash if dragged into active list
            const newTargetIdx = cats.findIndex(c => c.id === categoryDropTargetId);
            const insertIdx = categoryDropPlacement === 'before' ? newTargetIdx : newTargetIdx + 1;
            cats.splice(insertIdx, 0, draggedCat);
            
            labelsConfig.categories = cats;
            saveConfig();
          }
        }
      } else if (activeDropTarget && !activeCategoryDragId) {
        const newAssignments = { ...labelsConfig.assignments };
        const newOrders = { ...(labelsConfig.orders || {}) };

        for (const modName of selectedMods) {
          if (activeDropTarget === 'uncategorized') {
            delete newAssignments[modName];
          } else {
            newAssignments[modName] = activeDropTarget;
          }
        }

        let currentVisualOrder: string[] = [];
        if (activeDropTarget === 'uncategorized') {
          currentVisualOrder = sortedUncategorizedMods.map(m => m.file_name);
        } else {
          currentVisualOrder = (sortedCategorizedMods.get(activeDropTarget) || []).map(m => m.file_name);
        }

        currentVisualOrder = currentVisualOrder.filter(m => !selectedMods.has(m));

        if (dropTargetMod && !selectedMods.has(dropTargetMod)) {
          const targetIdx = currentVisualOrder.indexOf(dropTargetMod);
          if (targetIdx !== -1) {
            const insertIdx = dropPlacement === 'before' ? targetIdx : targetIdx + 1;
            currentVisualOrder.splice(insertIdx, 0, ...Array.from(selectedMods));
          } else {
            currentVisualOrder.push(...Array.from(selectedMods));
          }
        } else {
          currentVisualOrder.push(...Array.from(selectedMods));
        }

        newOrders[activeDropTarget] = currentVisualOrder;

        labelsConfig.assignments = newAssignments;
        labelsConfig.orders = newOrders;
        saveConfig();
      }
    } else {
      if (!e.ctrlKey && !e.metaKey && !e.shiftKey && !activeCategoryDragId) {
        if (lastSelectedMod) {
          selectedMods.clear();
          selectedMods.add(lastSelectedMod);
          selectedMods = new Set(selectedMods);
        }
      }
    }

    isPointerDown = false;
    isDragging = false;
    activeDropTarget = null;
    dropTargetMod = null;
    dropPlacement = null;
    activeCategoryDragId = null;
    categoryDropTargetId = null;
    categoryDropPlacement = null;
  }

  function clearSelection(e: PointerEvent) {
    if (e.target === e.currentTarget) {
      selectedMods.clear();
      selectedMods = new Set();
      lastSelectedMod = null;
    }
  }
</script>

<svelte:window onpointermove={handlePointerMove} onpointerup={handlePointerUp} onclick={closeContextMenu} />

<!-- Mod Drag Ghost -->
{#if isDragging && !activeCategoryDragId}
  <div
    id="dnd-ghost"
    class="fixed pointer-events-none z-50 px-3 py-1.5 rounded-lg bg-nvidia-card border border-cyan-400 shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
    style="left: {currentX}px; top: {currentY}px;"
  >
    <GripVertical class="h-3.5 w-3.5 text-cyan-400" />
    <FileArchive class="h-3.5 w-3.5 text-cyan-400" />
    <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">
      {#if selectedMods.size > 1}
        {selectedMods.size} Mods Selected
      {:else}
        {Array.from(selectedMods)[0]}
      {/if}
    </span>
  </div>
{/if}

<!-- Category Drag Ghost -->
{#if isDragging && activeCategoryDragId}
  {@const draggedCat = labelsConfig.categories.find(c => c.id === activeCategoryDragId)}
  {#if draggedCat}
    <div
      class="fixed pointer-events-none z-50 px-3 py-2 rounded-lg bg-nvidia-card border shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
      style="left: {currentX}px; top: {currentY}px; border-color: {draggedCat.color};"
    >
      <GripVertical class="h-3.5 w-3.5 text-nvidia-text-muted" />
      <div class="h-2.5 w-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {draggedCat.color};"></div>
      <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">{draggedCat.name}</span>
    </div>
  {/if}
{/if}

<!-- Context Menu -->
{#if contextMenu.visible && contextMenu.modName}
  <div
    class="fixed z-50 w-56 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center gap-2">
      <FileArchive class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
      <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate">
        {#if selectedMods.size > 1}
          {selectedMods.size} Mods Selected
        {:else}
          {contextMenu.modName}
        {/if}
      </span>
    </div>

    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleContextMenuShowInLoadOrder}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <ListTree class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Load Order</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <FolderSearch class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Explorer</span>
      </button>

      <div class="h-px bg-nvidia-border/50 my-1 mx-1"></div>

      <button
        type="button"
        onclick={() => { closeContextMenu(); isMoveCatModalOpen = true; }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <FolderInput class="h-3.5 w-3.5 text-cyan-400" />
        <span>Move to Category...</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuRemoveFromCategory}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-red-500/10 text-red-400 transition text-left cursor-pointer group"
      >
        <X class="h-3.5 w-3.5 text-red-400" />
        <span>Remove from Category</span>
      </button>
    </div>
  </div>
{/if}

<div class="flex h-full gap-4 overflow-hidden select-none" onpointerdown={clearSelection}>
  
  <!-- Left: Vertical Active Category Stack -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden" onpointerdown={clearSelection}>
    <div class="flex items-center justify-between mb-4 shrink-0">
      <div class="flex items-center gap-2">
        <LayoutGrid class="h-4 w-4 text-cyan-400" />
        <h3 class="text-sm font-bold uppercase tracking-wider text-nvidia-text-primary">Virtual Categories</h3>
      </div>
      
      <div class="flex items-center gap-4">
        <!-- Show Labels in Load Order Toggle -->
        <div class="flex items-center gap-2 bg-nvidia-surface/40 px-3 py-1.5 rounded border border-nvidia-border/60">
          <Tag class="h-3.5 w-3.5 text-nvidia-text-muted" />
          <span class="text-[11px] font-semibold text-nvidia-text-primary uppercase tracking-wider">Labels in Load Order</span>
          <button
            type="button"
            onclick={() => {
              showCategoryLabelsInLoadOrder = !showCategoryLabelsInLoadOrder;
              if (onLabelsToggle) onLabelsToggle(showCategoryLabelsInLoadOrder);
            }}
            class="w-7 h-4 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer ml-1 {showCategoryLabelsInLoadOrder ? 'bg-cyan-500 border border-cyan-400' : 'bg-zinc-900/90 border border-zinc-800'}"
          >
            <div class="h-2.5 w-2.5 rounded-full transition-transform transform {showCategoryLabelsInLoadOrder ? 'translate-x-3 bg-black shadow-xs' : 'translate-x-0 bg-zinc-500'}"></div>
          </button>
        </div>

        <button
          type="button"
          onclick={toggleAllCategories}
          disabled={totalActiveCategories === 0}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary font-medium transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isAllCollapsed}
            <FolderOpen class="h-3.5 w-3.5 text-cyan-400" />
            <span>Expand All</span>
          {:else}
            <Folder class="h-3.5 w-3.5 text-cyan-400" />
            <span>Collapse All</span>
          {/if}
        </button>

        <button
          type="button"
          onclick={openCreateModal}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-cyan-500/15 hover:bg-cyan-500/25 border border-cyan-500/30 text-xs font-bold text-cyan-400 transition cursor-pointer shadow-sm"
        >
          <Plus class="h-3.5 w-3.5" />
          <span>Add Category</span>
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto pr-2 pb-4" onpointerdown={clearSelection}>
      {#if activeCategories.length === 0}
        <div class="h-full flex flex-col items-center justify-center text-nvidia-text-muted border-2 border-dashed border-nvidia-border/60 rounded-xl bg-nvidia-surface/20">
          <LayoutGrid class="h-8 w-8 mb-3 opacity-30" />
          <p class="text-xs font-medium">No active virtual categories.</p>
          <p class="text-[10px] mt-1 opacity-70">Click "Add Category" or restore shelved categories from The Stash.</p>
        </div>
      {:else}
        <div class="flex flex-col space-y-2" onpointerdown={clearSelection}>
          {#each activeCategories as cat (cat.id)}
            {@const mods = sortedCategorizedMods.get(cat.id) || []}
            {@const matchesSearch = doesCategoryMatch(cat, mods)}
            
            <div class="relative flex flex-col transition-opacity duration-200 {isSearching && !matchesSearch ? 'opacity-25' : 'opacity-100'}">
              
              <!-- Drop Indicator Before Category -->
              {#if isDragging && activeCategoryDragId && categoryDropTargetId === cat.id && categoryDropPlacement === 'before'}
                <div class="absolute -top-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
              {/if}

              <!-- Category Header -->
              <div
                class="organizer-category-header group rounded-lg border transition-all duration-300 border-nvidia-border {activeCategoryDragId === cat.id ? 'opacity-30 border-dashed' : ''}"
                data-category-id={cat.id}
                style="background-color: color-mix(in srgb, {cat.color} 15%, transparent);"
              >
                <div class="flex items-center justify-between px-3 gap-2 density-row relative overflow-hidden">
                  <div class="absolute left-0 top-0 bottom-0 w-1" style="background-color: {cat.color};"></div>
                  
                  <div class="flex items-center gap-3 min-w-0 flex-1 pl-1">
                    <div
                      role="button"
                      tabindex="0"
                      onpointerdown={(e) => handleCategoryPointerDown(e, cat.id)}
                      class="cursor-grab active:cursor-grabbing p-1 text-nvidia-text-muted hover:text-nvidia-text-primary shrink-0 rounded hover:bg-nvidia-surface transition"
                      title="Drag to reorder category"
                    >
                      <GripVertical class="h-4 w-4" />
                    </div>

                    <button
                      type="button"
                      onclick={() => toggleCategoryCollapse(cat.id)}
                      class="flex items-center gap-1.5 text-nvidia-text-primary hover:text-cyan-400 transition cursor-pointer min-w-0"
                    >
                      {#if collapsedCategories[cat.id]}
                        <ChevronRight class="h-4 w-4 text-nvidia-text-muted shrink-0" />
                        <Folder class="h-4 w-4 text-nvidia-text-muted shrink-0" />
                      {:else}
                        <ChevronDown class="h-4 w-4 text-nvidia-text-muted shrink-0" />
                        <FolderOpen class="h-4 w-4 text-nvidia-text-muted shrink-0" />
                      {/if}
                      <span class="font-bold uppercase tracking-wider truncate">{cat.name}</span>
                    </button>

                    <span class="text-[10px] font-mono text-nvidia-text-muted shrink-0 opacity-80">
                      ({mods.length} {mods.length === 1 ? 'mod' : 'mods'})
                    </span>
                  </div>

                  <div class="flex items-center gap-1.5 shrink-0">
                    <button 
                      type="button"
                      onclick={() => toggleStashCategory(cat)} 
                      class="p-1 rounded text-nvidia-text-muted hover:text-amber-400 hover:bg-amber-500/10 transition cursor-pointer" 
                      title="Move Category to Stash (Also suppresses label in load order)"
                    >
                      <Archive class="h-3.5 w-3.5" />
                    </button>
                    <button 
                      type="button"
                      onclick={() => openEditModal(cat)} 
                      class="p-1 rounded text-nvidia-text-muted hover:text-cyan-400 hover:bg-nvidia-surface/80 transition cursor-pointer" 
                      title="Edit Category"
                    >
                      <Pencil class="h-3.5 w-3.5" />
                    </button>
                    <button 
                      type="button"
                      onclick={() => confirmDelete(cat)} 
                      class="p-1 rounded text-nvidia-text-muted hover:text-red-400 hover:bg-red-500/10 transition cursor-pointer" 
                      title="Delete Category"
                    >
                      <Trash2 class="h-3.5 w-3.5" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Drop Indicator After Category -->
              {#if isDragging && activeCategoryDragId && categoryDropTargetId === cat.id && categoryDropPlacement === 'after'}
                <div class="absolute -bottom-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
              {/if}

              <!-- Category Body (Mods) -->
              {#if !collapsedCategories[cat.id]}
                <div class="flex flex-col space-y-1 mt-1 mb-2" data-category-id={cat.id} onpointerdown={clearSelection}>
                  {#if mods.length === 0}
                    <div class="ml-3 pl-4 py-2 text-[10px] font-mono text-nvidia-text-muted/50 italic pointer-events-none border-l border-nvidia-border/45">
                      Drag mods here
                    </div>
                  {:else}
                    {#each mods as mod}
                      {@const isTargetMatch = doesModMatch(mod.file_name)}
                      <div class="relative flex flex-col ml-3">
                        <div class="absolute -left-2 -top-1 bottom-0 w-[1px] bg-nvidia-border/45 z-0"></div>
                        <div class="absolute -left-2 top-1/2 w-2 h-[1px] bg-nvidia-border/45 z-0"></div>
                        
                        {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'before' && !selectedMods.has(mod.file_name)}
                          <div class="absolute -top-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
                        {/if}

                        <div
                          onpointerdown={(e) => handlePointerDown(e, mod.file_name)}
                          oncontextmenu={(e) => openContextMenu(e, mod.file_name)}
                          ondblclick={() => onShowInLoadOrder && onShowInLoadOrder(mod.file_name)}
                          class="flex items-center gap-2 px-2 py-1.5 rounded border transition density-row cursor-pointer z-10 
                            {selectedMods.has(mod.file_name) ? 'border-cyan-500/60 bg-cyan-500/15 opacity-50' : 
                            (isTargetMatch ? 'border-cyan-400 ring-1 ring-cyan-400 bg-cyan-950/40' : 'border-nvidia-border/50 bg-nvidia-surface hover:border-nvidia-border')}"
                          data-mod-name={mod.file_name}
                          title="Double-click to locate in Load Order"
                        >
                          <GripVertical class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
                          <FileArchive class="h-3.5 w-3.5 {isTargetMatch ? 'text-cyan-400' : 'text-nvidia-text-muted'} shrink-0" />
                          <span class="text-[11px] font-mono truncate {isTargetMatch ? 'text-cyan-300 font-bold' : (mod.enabled ? 'text-nvidia-text-primary' : 'opacity-50 line-through')}">
                            {mod.file_name}
                          </span>
                        </div>

                        {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'after' && !selectedMods.has(mod.file_name)}
                          <div class="absolute -bottom-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
                        {/if}
                      </div>
                    {/each}
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Right: The Stash Drawer -->
  {#if showUncategorizedDrawer}
    <div 
      id="stash-drawer"
      class="w-84 flex flex-col rounded-xl border transition-all duration-200 bg-nvidia-surface/40 overflow-hidden shrink-0 shadow-sm 
        {((activeDropTarget === 'uncategorized' && !activeCategoryDragId) || (activeCategoryDragId && categoryDropTargetId === 'stash')) ? 'border-amber-400 ring-1 ring-amber-400 bg-amber-950/20' : 'border-nvidia-border'}"
      data-category-id="uncategorized"
    >
      <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/80 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2">
          <Archive class="h-4 w-4 text-amber-400" />
          <h3 class="text-sm font-bold uppercase tracking-wider text-nvidia-text-primary">The Stash</h3>
        </div>
        <span class="text-[10px] font-mono text-nvidia-text-muted bg-nvidia-bg px-2 py-0.5 rounded border border-nvidia-border">
          {sortedUncategorizedMods.length} loose • {stashedCategories.length} shelved
        </span>
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-4 bg-nvidia-bg/30" onpointerdown={clearSelection}>
        
        <!-- Shelved Categories Section -->
        {#if stashedCategories.length > 0}
          <div class="space-y-1.5">
            <div class="text-[10px] font-bold uppercase tracking-wider text-amber-400/90 flex items-center gap-1.5 px-1">
              <Package class="h-3 w-3" />
              <span>Shelved Categories</span>
            </div>
            
            <div class="space-y-1">
              {#each stashedCategories as cat (cat.id)}
                {@const catMods = sortedCategorizedMods.get(cat.id) || []}
                <div 
                  class="rounded-lg border border-nvidia-border/70 bg-nvidia-surface/70 p-2 flex items-center justify-between gap-2 shadow-xs group"
                  style="border-left: 3px solid {cat.color};"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <div
                      role="button"
                      tabindex="0"
                      onpointerdown={(e) => handleCategoryPointerDown(e, cat.id)}
                      class="cursor-grab active:cursor-grabbing p-1 text-nvidia-text-muted hover:text-nvidia-text-primary shrink-0 rounded hover:bg-nvidia-surface transition"
                      title="Drag category back to main list"
                    >
                      <GripVertical class="h-3.5 w-3.5" />
                    </div>
                    <span class="text-xs font-bold text-nvidia-text-primary truncate">{cat.name}</span>
                    <span class="text-[10px] font-mono text-nvidia-text-muted shrink-0">
                      ({catMods.length} {catMods.length === 1 ? 'mod' : 'mods'})
                    </span>
                  </div>

                  <button
                    type="button"
                    onclick={() => toggleStashCategory(cat)}
                    class="flex items-center gap-1 px-2 py-1 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border text-[10px] font-bold text-cyan-400 transition cursor-pointer shrink-0"
                    title="Restore category to main view"
                  >
                    <RotateCcw class="h-3 w-3" />
                    <span>Restore</span>
                  </button>
                </div>
              {/each}
            </div>
          </div>

          <div class="h-px bg-nvidia-border/50"></div>
        {/if}

        <!-- Loose / Unassigned Mods Section -->
        <div class="space-y-1.5">
          <div class="text-[10px] font-bold uppercase tracking-wider text-nvidia-text-muted flex items-center justify-between px-1">
            <span>Unassigned Mods</span>
            <span class="font-mono text-[9px]">{sortedUncategorizedMods.length}</span>
          </div>

          {#if sortedUncategorizedMods.length === 0}
            <div class="p-6 text-center text-xs text-nvidia-text-muted pointer-events-none italic">
              All mods are categorized!
            </div>
          {:else}
            <div class="space-y-1">
              {#each sortedUncategorizedMods as mod (mod.file_name)}
                {@const isTargetMatch = doesModMatch(mod.file_name)}
                <div class="relative flex flex-col">
                  {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'before' && !selectedMods.has(mod.file_name)}
                    <div class="absolute -top-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
                  {/if}

                  <div
                    onpointerdown={(e) => handlePointerDown(e, mod.file_name)}
                    oncontextmenu={(e) => openContextMenu(e, mod.file_name)}
                    ondblclick={() => onShowInLoadOrder && onShowInLoadOrder(mod.file_name)}
                    class="flex items-center gap-2 px-2 py-1.5 rounded border transition density-row cursor-pointer 
                      {selectedMods.has(mod.file_name) ? 'border-cyan-500/60 bg-cyan-500/15 opacity-50' : 
                      (isTargetMatch ? 'border-cyan-400 ring-1 ring-cyan-400 bg-cyan-950/40' : 'border-nvidia-border/50 bg-nvidia-surface hover:border-nvidia-border')}"
                    data-mod-name={mod.file_name}
                    title="Double-click to locate in Load Order"
                  >
                    <GripVertical class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
                    <FileArchive class="h-3.5 w-3.5 {isTargetMatch ? 'text-cyan-400' : 'text-nvidia-text-muted'} shrink-0" />
                    <span class="text-[11px] font-mono truncate {isTargetMatch ? 'text-cyan-300 font-bold' : (mod.enabled ? 'text-nvidia-text-primary' : 'opacity-50 line-through')}">
                      {mod.file_name}
                    </span>
                  </div>

                  {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'after' && !selectedMods.has(mod.file_name)}
                    <div class="absolute -bottom-1 left-0 right-0 h-[2.5px] bg-cyan-400 z-30 shadow-[0_0_12px_rgba(34,211,238,0.8)]"></div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>

      </div>
    </div>
  {/if}

</div>

<CategoryModal
  bind:isOpen={isCatModalOpen}
  mode={catModalMode}
  initialData={catModalData}
  onConfirm={handleCatModalConfirm}
  onCancel={() => isCatModalOpen = false}
/>

<DialogModal
  bind:isOpen={isDeleteModalOpen}
  title="Delete Category"
  message="Are you sure you want to delete the '{catToDelete?.name}' category? Mods inside will be returned to the Unassigned list. This does not delete any mod files."
  mode="confirm"
  confirmText="Delete Category"
  isDanger={true}
  onConfirm={executeDelete}
  onCancel={() => isDeleteModalOpen = false}
/>

<MoveCategoryModal
  bind:isOpen={isMoveCatModalOpen}
  categories={labelsConfig.categories}
  hasExistingLabel={selectedMods.size > 0 && Array.from(selectedMods).some(m => labelsConfig.assignments[m])}
  onConfirm={handleMoveToCategory}
  onCancel={() => isMoveCatModalOpen = false}
/>