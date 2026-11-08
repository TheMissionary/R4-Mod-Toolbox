<script lang="ts">
  import type { ArchiveItem, LabelsConfig, CategoryLabel } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { 
    Plus, Pencil, Trash2, GripVertical, FileArchive, LayoutGrid, Tag, ListTree, 
    FolderSearch, FolderInput, X, ChevronDown, ChevronRight, Folder, FolderOpen,
    Archive, RotateCcw, Package, Search, MessageSquare, Columns, MoreVertical
  } from 'lucide-svelte';
  import CategoryModal from './CategoryModal.svelte';
  import DialogModal from './DialogModal.svelte';
  import MoveCategoryModal from './MoveCategoryModal.svelte';
  import NoteModal from './NoteModal.svelte';
  import { loadConfigFromDisk, saveConfigToDisk } from '$lib/theme';

  let {
    archives = [],
    labelsConfig = $bindable({ categories: [], assignments: {}, orders: {} }),
    notesConfig = $bindable({}),
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
    notesConfig: Record<string, string>;
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
  
  let isNoteModalOpen = $state(false);
  let targetNoteModName = $state('');

  let organizerColumns = $state<'auto' | '1' | '2' | '3'>('auto');

  // Inverted state: Categories are collapsed by default. 
  // If their ID is in this object as true, they are expanded.
  let expandedCategories = $state<Record<string, boolean>>({});
  
  let activeCategories = $derived(labelsConfig.categories.filter(c => !c.stashed));
  let stashedCategories = $derived(labelsConfig.categories.filter(c => c.stashed));

  let totalActiveCategories = $derived(activeCategories.length);
  let expandedCount = $derived(activeCategories.filter(c => expandedCategories[c.id]).length);
  let isAllExpanded = $derived(totalActiveCategories > 0 && expandedCount === totalActiveCategories);

  $effect(() => {
    if (gamePath) {
      loadConfigFromDisk().then(config => {
        if (config.organizerColumns) {
          organizerColumns = config.organizerColumns as 'auto' | '1' | '2' | '3';
        }
      }).catch(err => console.error('Failed to load organizer columns config:', err));
    }
  });

  async function handleColumnsChange() {
    try {
      const config = await loadConfigFromDisk();
      config.organizerColumns = organizerColumns;
      await saveConfigToDisk(config);
    } catch (err) {
      console.error('Failed to save column preference:', err);
    }
  }

  function toggleAllCategories() {
    if (isAllExpanded) {
      expandedCategories = {};
    } else {
      const newState: Record<string, boolean> = {};
      for (const cat of activeCategories) {
        newState[cat.id] = true;
      }
      expandedCategories = newState;
    }
  }

  function toggleCategoryExpansion(catId: string) {
    expandedCategories[catId] = !expandedCategories[catId];
  }

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
      const newExpanded = { ...expandedCategories };
      for (const cat of activeCategories) {
        const mods = sortedCategorizedMods.get(cat.id) || [];
        if (doesCategoryMatch(cat, mods)) {
          newExpanded[cat.id] = true;
        }
      }
      expandedCategories = newExpanded;
    }
  });

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

  let selectedMods = $state<Set<string>>(new Set());
  let lastSelectedMod = $state<string | null>(null);

  let isPointerDown = $state(false);
  let isDragging = $state(false);
  let startX = $state(0);
  let startY = $state(0);
  let currentX = $state(0);
  let currentY = $state(0);
  
  let activeDropTarget = $state<string | null>(null);
  let dropTargetMod = $state<string | null>(null);
  let dropPlacement = $state<'before' | 'after' | null>(null);

  let activeCategoryDragId = $state<string | null>(null);
  let categoryDropTargetId = $state<string | null>(null);
  let categoryDropPlacement = $state<'before' | 'after' | null>(null);

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    modName: string | null;
  }>({ visible: false, x: 0, y: 0, modName: null });

  let categoryContextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    cat: CategoryLabel | null;
  }>({ visible: false, x: 0, y: 0, cat: null });

  async function saveConfig() {
    try {
      await invoke('save_labels_config', { gamePath, config: labelsConfig });
    } catch (err) {
      console.error('Failed to save labels config:', err);
    }
  }

  function toggleStashCategory(cat: CategoryLabel) {
    cat.stashed = !cat.stashed;
    labelsConfig.categories = [...labelsConfig.categories];
    saveConfig();
  }

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
    const newId = data.id || crypto.randomUUID();
    if (catModalMode === 'create') {
      labelsConfig.categories = [...labelsConfig.categories, { ...data, id: newId, stashed: false }];
      if (selectedMods.size > 0) {
        const newAssignments = { ...labelsConfig.assignments };
        const newOrders = { ...(labelsConfig.orders || {}) };
        
        // Explicitly sort alphabetically on initial creation
        const sortedSelected = Array.from(selectedMods).sort((a, b) => a.localeCompare(b));
        
        for (const mod of sortedSelected) {
          newAssignments[mod] = newId;
        }
        newOrders[newId] = sortedSelected;
        
        labelsConfig.assignments = newAssignments;
        labelsConfig.orders = newOrders;
        
        selectedMods.clear();
        selectedMods = new Set();
        lastSelectedMod = null;
      }
    } else {
      const idx = labelsConfig.categories.findIndex(c => c.id === data.id);
      if (idx !== -1) {
        labelsConfig.categories[idx] = { ...labelsConfig.categories[idx], ...data };
      }
    }
    saveConfig();
    isCatModalOpen = false;
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
    const menuHeight = 200;
    const posX = (e.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : e.clientX;
    const posY = (e.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : e.clientY;
    contextMenu = { visible: true, x: posX, y: posY, modName };
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.modName = null;
    }
  }

  function openCategoryContextMenu(e: MouseEvent, cat: CategoryLabel) {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 180;
    const menuHeight = 140;
    const posX = (e.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : e.clientX;
    const posY = (e.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : e.clientY;
    categoryContextMenu = { visible: true, x: posX, y: posY, cat };
  }

  function closeCategoryContextMenu() {
    if (categoryContextMenu.visible) {
      categoryContextMenu.visible = false;
      categoryContextMenu.cat = null;
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
    const newOrders = { ...(labelsConfig.orders || {}) };
    const targetOrder = newOrders[actionOrId] || [];
    
    // Explicitly sort alphabetically on initial move
    const sortedSelected = Array.from(selectedMods).sort((a, b) => a.localeCompare(b));
    
    for (const mod of sortedSelected) {
      newAssignments[mod] = actionOrId;
      if (!targetOrder.includes(mod)) {
        targetOrder.push(mod);
      }
    }
    newOrders[actionOrId] = targetOrder;
    
    labelsConfig.assignments = newAssignments;
    labelsConfig.orders = newOrders;
    saveConfig();
    isMoveCatModalOpen = false;
  }

  async function handleNoteConfirm(note: string) {
    if (!targetNoteModName) return;
    const modName = targetNoteModName;
    if (note.trim() === '') {
      delete notesConfig[modName];
    } else {
      notesConfig[modName] = note.trim();
    }
    notesConfig = { ...notesConfig };
    try {
      await invoke('save_notes_config', { notes: notesConfig });
    } catch (err) {
      console.error('Failed to save notes:', err);
    }
  }

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
          // 2D Pointer Hit Detection for Categories (Absolute Insertion Index Normalization)
          const el = document.elementFromPoint(e.clientX, e.clientY);
          const closestCat = el?.closest('.organizer-category-header');
          if (closestCat) {
            const targetId = closestCat.getAttribute('data-category-id');
            if (targetId) {
              const rect = closestCat.getBoundingClientRect();
              const isTopHalf = e.clientY < (rect.top + rect.height / 2);
              
              const targetIdx = activeCategories.findIndex(c => c.id === targetId);
              if (targetIdx !== -1) {
                let insertIdx = isTopHalf ? targetIdx : targetIdx + 1;
                const dragIdx = activeCategories.findIndex(c => c.id === activeCategoryDragId);
                
                // Prevent dropping on itself
                if (insertIdx === dragIdx || insertIdx === dragIdx + 1) {
                  categoryDropTargetId = null;
                  categoryDropPlacement = null;
                } else {
                  // Normalize to always use 'before' unless it's the very end of the list
                  if (insertIdx < activeCategories.length) {
                    categoryDropTargetId = activeCategories[insertIdx].id;
                    categoryDropPlacement = 'before';
                  } else {
                    categoryDropTargetId = activeCategories[activeCategories.length - 1].id;
                    categoryDropPlacement = 'after';
                  }
                }
              }
            }
          } else {
            categoryDropTargetId = null;
            categoryDropPlacement = null;
          }
        }
      } else {
        const el = document.elementFromPoint(e.clientX, e.clientY);
        const targetContainer = el?.closest('[data-category-id]');
        activeDropTarget = targetContainer ? targetContainer.getAttribute('data-category-id') : null;
        
        // 2D Pointer Hit Detection for Mods (Absolute Insertion Index Normalization)
        const modEl = el?.closest('[data-mod-name]');
        if (modEl && activeDropTarget) {
          const rect = modEl.getBoundingClientRect();
          const isTopHalf = e.clientY < (rect.top + rect.height / 2);
          const hoveredMod = modEl.getAttribute('data-mod-name');
          
          let currentVisualOrder = activeDropTarget === 'uncategorized' 
            ? sortedUncategorizedMods.map(m => m.file_name)
            : (sortedCategorizedMods.get(activeDropTarget) || []).map(m => m.file_name);
            
          const targetIdx = currentVisualOrder.indexOf(hoveredMod || '');
          if (targetIdx !== -1) {
            let insertIdx = isTopHalf ? targetIdx : targetIdx + 1;
            
            if (insertIdx < currentVisualOrder.length) {
              dropTargetMod = currentVisualOrder[insertIdx];
              dropPlacement = 'before';
            } else {
              dropTargetMod = currentVisualOrder[currentVisualOrder.length - 1];
              dropPlacement = 'after';
            }
          }
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
          const cat = labelsConfig.categories.find(c => c.id === activeCategoryDragId);
          if (cat) {
            cat.stashed = true;
            labelsConfig.categories = [...labelsConfig.categories];
            saveConfig();
          }
        } else if (categoryDropTargetId && categoryDropPlacement) {
          const cats = [...labelsConfig.categories];
          const dragIdx = cats.findIndex(c => c.id === activeCategoryDragId);
          const targetIdx = cats.findIndex(c => c.id === categoryDropTargetId);
          if (dragIdx !== -1 && targetIdx !== -1) {
            const [draggedCat] = cats.splice(dragIdx, 1);
            draggedCat.stashed = false;
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

<svelte:window onpointermove={handlePointerMove} onpointerup={handlePointerUp} onclick={() => { closeContextMenu(); closeCategoryContextMenu(); }} />

<!-- Mod Drag Ghost -->
{#if isDragging && !activeCategoryDragId}
  <div
    id="dnd-ghost"
    class="fixed pointer-events-none z-50 px-3 py-1.5 rounded-lg border shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
    style="left: {currentX}px; top: {currentY}px; background-color: var(--theme-card); border-color: var(--theme-accent);"
  >
    <GripVertical class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
    <FileArchive class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
    <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">
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
      class="fixed pointer-events-none z-50 px-3 py-2 rounded-lg border shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
      style="left: {currentX}px; top: {currentY}px; background-color: var(--theme-card); border-color: {draggedCat.color};"
    >
      <GripVertical class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
      <div class="h-2.5 w-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {draggedCat.color};"></div>
      <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">{draggedCat.name}</span>
    </div>
  {/if}
{/if}

<!-- Mod Context Menu -->
{#if contextMenu.visible && contextMenu.modName}
  <div
    class="fixed z-50 w-56 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center gap-2" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
      <FileArchive class="h-3.5 w-3.5 shrink-0" style="color: var(--color-info);" />
      <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);">
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
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <ListTree class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in Load Order</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <FolderSearch class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in Explorer</span>
      </button>

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

      <button
        type="button"
        onclick={() => { targetNoteModName = contextMenu.modName || ''; closeContextMenu(); isNoteModalOpen = true; }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
        <span>{notesConfig[contextMenu.modName] ? 'Edit Note...' : 'Add Note...'}</span>
      </button>

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

      <button
        type="button"
        onclick={() => { closeContextMenu(); isMoveCatModalOpen = true; }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <FolderInput class="h-3.5 w-3.5" style="color: var(--color-info);" />
        <span>Move to Category...</span>
      </button>

      <button
        type="button"
        onclick={handleContextMenuRemoveFromCategory}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--color-danger);"
      >
        <X class="h-3.5 w-3.5" style="color: var(--color-danger);" />
        <span>Remove from Category</span>
      </button>
    </div>
  </div>
{/if}

<!-- Category Context Menu -->
{#if categoryContextMenu.visible && categoryContextMenu.cat}
  <div
    class="fixed z-50 w-48 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {categoryContextMenu.x}px; top: {categoryContextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center gap-2" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
      <div class="h-2.5 w-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {categoryContextMenu.cat.color};"></div>
      <span class="font-bold truncate uppercase tracking-wider" style="color: var(--theme-text-primary);">{categoryContextMenu.cat.name}</span>
    </div>
    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={() => { toggleStashCategory(categoryContextMenu.cat!); closeCategoryContextMenu(); }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Archive class="h-3.5 w-3.5" style="color: var(--color-warning);" />
        <span>Move to Stash</span>
      </button>
      <button
        type="button"
        onclick={() => { openEditModal(categoryContextMenu.cat!); closeCategoryContextMenu(); }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Pencil class="h-3.5 w-3.5" style="color: var(--color-info);" />
        <span>Edit Category</span>
      </button>
      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>
      <button
        type="button"
        onclick={() => { confirmDelete(categoryContextMenu.cat!); closeCategoryContextMenu(); }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--color-danger);"
      >
        <Trash2 class="h-3.5 w-3.5" style="color: var(--color-danger);" />
        <span>Delete Category</span>
      </button>
    </div>
  </div>
{/if}

<div class="flex h-full gap-4 overflow-hidden select-none" onpointerdown={clearSelection}>
  
  <!-- Left: Split View (Category Tray & Active Canvas) -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden" onpointerdown={clearSelection}>
    
    <!-- Header Controls -->
    <div class="flex items-center justify-between mb-4 shrink-0">
      <div class="flex items-center gap-2">
        <LayoutGrid class="h-4 w-4" style="color: var(--color-info);" />
        <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Virtual Categories</h3>
      </div>
      
      <div class="flex items-center gap-4">
        <!-- Column Control Dropdown -->
        <div class="flex items-center gap-2 px-2 py-1 rounded border" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: color-mix(in srgb, var(--theme-border) 60%, transparent);">
          <Columns class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
          <select
            bind:value={organizerColumns}
            onchange={handleColumnsChange}
            class="py-0.5 px-1 bg-transparent text-[11px] font-semibold focus:outline-none cursor-pointer uppercase tracking-wider"
            style="color: var(--theme-text-primary);"
          >
            <option value="auto" style="background-color: var(--theme-surface); color: var(--theme-text-primary);">Auto Cols</option>
            <option value="1" style="background-color: var(--theme-surface); color: var(--theme-text-primary);">1 Column</option>
            <option value="2" style="background-color: var(--theme-surface); color: var(--theme-text-primary);">2 Columns</option>
            <option value="3" style="background-color: var(--theme-surface); color: var(--theme-text-primary);">3 Columns</option>
          </select>
        </div>

        <!-- Show Labels in Load Order Toggle -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded border" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: color-mix(in srgb, var(--theme-border) 60%, transparent);">
          <Tag class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
          <span class="text-[11px] font-semibold uppercase tracking-wider" style="color: var(--theme-text-primary);">Labels in Load Order</span>
          <button
            type="button"
            onclick={() => {
              showCategoryLabelsInLoadOrder = !showCategoryLabelsInLoadOrder;
              if (onLabelsToggle) onLabelsToggle(showCategoryLabelsInLoadOrder);
            }}
            class="w-7 h-4 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer ml-1 border"
            style="{showCategoryLabelsInLoadOrder ? 'background-color: var(--color-info); border-color: var(--color-info);' : 'background-color: color-mix(in srgb, var(--theme-bg) 90%, transparent); border-color: var(--theme-border);'}"
          >
            <div class="h-2.5 w-2.5 rounded-full transition-transform transform {showCategoryLabelsInLoadOrder ? 'translate-x-3 shadow-xs' : 'translate-x-0'}" style="background-color: {showCategoryLabelsInLoadOrder ? '#000000' : 'var(--theme-text-muted)'};"></div>
          </button>
        </div>

        <button
          type="button"
          onclick={toggleAllCategories}
          disabled={totalActiveCategories === 0}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded border text-xs font-medium transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-125"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        >
          {#if isAllExpanded}
            <Folder class="h-3.5 w-3.5" style="color: var(--color-info);" />
            <span>Collapse All</span>
          {:else}
            <FolderOpen class="h-3.5 w-3.5" style="color: var(--color-info);" />
            <span>Expand All</span>
          {/if}
        </button>

        <button
          type="button"
          onclick={openCreateModal}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded border text-xs font-bold transition cursor-pointer shadow-sm hover:brightness-125"
          style="background-color: color-mix(in srgb, var(--color-info) 15%, transparent); border-color: color-mix(in srgb, var(--color-info) 30%, transparent); color: var(--color-info);"
        >
          <Plus class="h-3.5 w-3.5" />
          <span>Add Category</span>
        </button>
      </div>
    </div>

    {#if activeCategories.length === 0}
      <div class="flex-1 flex flex-col items-center justify-center border-2 border-dashed rounded-xl" style="color: var(--theme-text-muted); border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 20%, transparent);">
        <LayoutGrid class="h-8 w-8 mb-3 opacity-30" />
        <p class="text-xs font-medium">No active virtual categories.</p>
        <p class="text-[10px] mt-1 opacity-70">Click "Add Category" or restore shelved categories from The Stash.</p>
      </div>
    {:else}
      <!-- Category Tray (Top Section) -->
      {@const columnsClass = organizerColumns === '1' ? 'columns-1' : 
                             organizerColumns === '2' ? 'columns-2' : 
                             organizerColumns === '3' ? 'columns-3' : 
                             'columns-1 lg:columns-2 2xl:columns-3'}
      
      <div class="shrink-0 max-h-[40%] overflow-y-auto pr-2 pb-4 pt-2 border-b mb-4" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent);" onpointerdown={clearSelection}>
        <div class="{columnsClass} gap-3">
          {#each activeCategories as cat (cat.id)}
            {@const mods = sortedCategorizedMods.get(cat.id) || []}
            {@const matchesSearch = doesCategoryMatch(cat, mods)}
            {@const isHovered = isDragging && !activeCategoryDragId && activeDropTarget === cat.id}
            {@const isExpanded = !!expandedCategories[cat.id]}
            
            <div class="break-inside-avoid mb-3 relative flex flex-col transition-opacity duration-200 {isSearching && !matchesSearch ? 'opacity-25' : 'opacity-100'}">
              
              {#if isDragging && activeCategoryDragId && categoryDropTargetId === cat.id && categoryDropPlacement === 'before'}
                <div class="absolute top-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
              {/if}

              <div
                class="organizer-category-header group rounded-lg border transition-all duration-300 {activeCategoryDragId === cat.id ? 'opacity-30 border-dashed' : ''} {isHovered ? 'ring-1' : ''} {isExpanded ? 'ring-1 shadow-md' : ''}"
                data-category-id={cat.id}
                style="background-color: color-mix(in srgb, {cat.color} {isHovered || isExpanded ? '25%' : '15%'}, transparent); border-color: {isHovered || isExpanded ? cat.color : 'var(--theme-border)'}; {isHovered || isExpanded ? `box-shadow: 0 0 15px color-mix(in srgb, ${cat.color} 40%, transparent);` : ''}"
                oncontextmenu={(e) => openCategoryContextMenu(e, cat)}
              >
                <div class="flex items-center justify-between px-3 gap-2 density-row relative overflow-hidden">
                  <div class="absolute left-0 top-0 bottom-0 w-1" style="background-color: {cat.color};"></div>
                  
                  <div class="flex items-center gap-3 min-w-0 flex-1 pl-1">
                    <div
                      role="button"
                      tabindex="0"
                      onpointerdown={(e) => handleCategoryPointerDown(e, cat.id)}
                      class="cursor-grab active:cursor-grabbing p-1 rounded transition shrink-0 hover:bg-[var(--theme-surface)]"
                      style="color: var(--theme-text-muted);"
                      title="Drag to reorder category"
                    >
                      <GripVertical class="h-4 w-4" />
                    </div>

                    <button
                      type="button"
                      onclick={() => toggleCategoryExpansion(cat.id)}
                      class="flex items-center gap-1.5 transition cursor-pointer min-w-0 hover:brightness-125"
                      style="color: var(--theme-text-primary);"
                    >
                      {#if isExpanded}
                        <ChevronDown class="h-4 w-4 shrink-0" style="color: var(--theme-text-muted);" />
                      {:else}
                        <ChevronRight class="h-4 w-4 shrink-0" style="color: var(--theme-text-muted);" />
                      {/if}
                      <span class="font-bold uppercase tracking-wider truncate">{cat.name}</span>
                    </button>
                  </div>

                  <div class="flex items-center gap-1.5 shrink-0">
                    <span class="text-[10px] font-mono shrink-0 opacity-80" style="color: var(--theme-text-muted);">
                      ({mods.length})
                    </span>
                    
                    <button 
                      type="button"
                      onclick={(e) => openCategoryContextMenu(e, cat)} 
                      class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-surface)]" 
                      style="color: var(--theme-text-muted);"
                      title="More options"
                    >
                      <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
                    </button>
                  </div>
                </div>
              </div>

              {#if isDragging && activeCategoryDragId && categoryDropTargetId === cat.id && categoryDropPlacement === 'after'}
                <div class="absolute bottom-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Active Canvas (Bottom Section) -->
      <div class="flex-1 overflow-y-auto pr-2 pb-4 pt-2" onpointerdown={clearSelection}>
        {#each activeCategories.filter(c => expandedCategories[c.id]) as cat (cat.id)}
          {@const mods = sortedCategorizedMods.get(cat.id) || []}
          <div class="mb-6" data-category-id={cat.id}>
            
            <!-- Canvas Section Header -->
            <div class="flex items-center gap-2 mb-3 px-1">
              <div class="h-3 w-3 rounded-full shadow-sm" style="background-color: {cat.color};"></div>
              <h4 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">{cat.name}</h4>
            </div>

            {#if mods.length === 0}
              <div class="ml-3 pl-4 py-2 text-[10px] font-mono italic pointer-events-none border-l" style="color: color-mix(in srgb, var(--theme-text-muted) 50%, transparent); border-color: color-mix(in srgb, var(--theme-border) 45%, transparent);">
                Drag mods here
              </div>
            {:else}
              <!-- CSS Columns for Vertical Flow -->
              <div class="{columnsClass} gap-3">
                {#each mods as mod}
                  {@const isTargetMatch = doesModMatch(mod.file_name)}
                  {@const isSelected = selectedMods.has(mod.file_name)}
                  <div class="break-inside-avoid mb-3 relative flex flex-col">
                    
                    {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'before' && !isSelected}
                      <div class="absolute top-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
                    {/if}

                    <div
                      onpointerdown={(e) => handlePointerDown(e, mod.file_name)}
                      oncontextmenu={(e) => openContextMenu(e, mod.file_name)}
                      ondblclick={() => onShowInLoadOrder && onShowInLoadOrder(mod.file_name)}
                      class="flex items-center justify-between px-2 py-1.5 rounded border transition density-row cursor-pointer z-10 
                        {isSelected ? 'ring-1 z-10 relative' : (isTargetMatch ? 'ring-1' : 'hover:brightness-125')}"
                      style="{isSelected ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: var(--theme-accent); box-shadow: 0 0 15px color-mix(in srgb, var(--theme-accent) 35%, transparent);' : (isTargetMatch ? 'background-color: color-mix(in srgb, var(--color-info) 20%, transparent); border-color: var(--color-info); box-shadow: 0 0 0 1px var(--color-info);' : 'background-color: var(--theme-surface); border-color: color-mix(in srgb, var(--theme-border) 50%, transparent);')}"
                      data-mod-name={mod.file_name}
                      title="Double-click to locate in Load Order"
                    >
                      <div class="flex items-center gap-2 min-w-0">
                        <GripVertical class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                        <FileArchive class="h-3.5 w-3.5 shrink-0" style="color: {isTargetMatch ? 'var(--color-info)' : 'var(--theme-text-muted)'};" />
                        <span class="text-[11px] font-mono truncate {isTargetMatch ? 'font-bold' : (mod.enabled ? '' : 'opacity-50 line-through')}" style="color: {isTargetMatch ? 'var(--color-info)' : 'var(--theme-text-primary)'};">
                          {mod.file_name}
                        </span>
                      </div>
                      
                      {#if notesConfig[mod.file_name]}
                        <button 
                          type="button"
                          onclick={(e) => { e.stopPropagation(); targetNoteModName = mod.file_name; isNoteModalOpen = true; }}
                          class="shrink-0 ml-2 hover:scale-110 transition-transform cursor-pointer" 
                          title={notesConfig[mod.file_name]}
                        >
                          <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
                        </button>
                      {/if}
                    </div>

                    {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'after' && !isSelected}
                      <div class="absolute bottom-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Right: The Stash Drawer -->
  {#if showUncategorizedDrawer}
    <div 
      id="stash-drawer"
      class="w-84 flex flex-col rounded-xl border transition-all duration-200 overflow-hidden shrink-0 shadow-sm 
        {((activeDropTarget === 'uncategorized' && !activeCategoryDragId) || (activeCategoryDragId && categoryDropTargetId === 'stash')) ? 'ring-1' : ''}"
      style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: {((activeDropTarget === 'uncategorized' && !activeCategoryDragId) || (activeCategoryDragId && categoryDropTargetId === 'stash')) ? 'var(--color-warning)' : 'var(--theme-border)'}; {((activeDropTarget === 'uncategorized' && !activeCategoryDragId) || (activeCategoryDragId && categoryDropTargetId === 'stash')) ? 'background-color: color-mix(in srgb, var(--color-warning) 15%, transparent); box-shadow: 0 0 0 1px var(--color-warning);' : ''}"
      data-category-id="uncategorized"
    >
      <div class="px-4 py-3 border-b flex items-center justify-between shrink-0" style="background-color: color-mix(in srgb, var(--theme-surface) 80%, transparent); border-color: color-mix(in srgb, var(--theme-border) 60%, transparent);">
        <div class="flex items-center gap-2">
          <Archive class="h-4 w-4" style="color: var(--color-warning);" />
          <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">The Stash</h3>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded border" style="background-color: var(--theme-bg); border-color: var(--theme-border); color: var(--theme-text-muted);">
          {sortedUncategorizedMods.length} loose • {stashedCategories.length} shelved
        </span>
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-4" style="background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);" onpointerdown={clearSelection}>
        
        <!-- Shelved Categories Section -->
        {#if stashedCategories.length > 0}
          <div class="space-y-1.5">
            <div class="text-[10px] font-bold uppercase tracking-wider flex items-center gap-1.5 px-1" style="color: color-mix(in srgb, var(--color-warning) 90%, transparent);">
              <Package class="h-3 w-3" />
              <span>Shelved Categories</span>
            </div>
            
            <div class="space-y-1">
              {#each stashedCategories as cat (cat.id)}
                {@const catMods = sortedCategorizedMods.get(cat.id) || []}
                <div 
                  class="rounded-lg border p-2 flex items-center justify-between gap-2 shadow-xs group"
                  style="background-color: color-mix(in srgb, var(--theme-surface) 70%, transparent); border-color: color-mix(in srgb, var(--theme-border) 70%, transparent); border-left: 3px solid {cat.color};"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <div
                      role="button"
                      tabindex="0"
                      onpointerdown={(e) => handleCategoryPointerDown(e, cat.id)}
                      class="cursor-grab active:cursor-grabbing p-1 rounded transition shrink-0 hover:bg-[var(--theme-surface)]"
                      style="color: var(--theme-text-muted);"
                      title="Drag category back to main list"
                    >
                      <GripVertical class="h-3.5 w-3.5" />
                    </div>
                    <span class="text-xs font-bold truncate" style="color: var(--theme-text-primary);">{cat.name}</span>
                    <span class="text-[10px] font-mono shrink-0" style="color: var(--theme-text-muted);">
                      ({catMods.length})
                    </span>
                  </div>

                  <button
                    type="button"
                    onclick={() => toggleStashCategory(cat)}
                    class="flex items-center gap-1 px-2 py-1 rounded border text-[10px] font-bold transition cursor-pointer shrink-0 hover:bg-[var(--theme-surface)]"
                    style="background-color: var(--theme-card); border-color: var(--theme-border); color: var(--color-info);"
                    title="Restore category to main view"
                  >
                    <RotateCcw class="h-3 w-3" />
                    <span>Restore</span>
                  </button>
                </div>
              {/each}
            </div>
          </div>

          <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>
        {/if}

        <!-- Loose / Unassigned Mods Section -->
        <div class="space-y-1.5">
          <div class="text-[10px] font-bold uppercase tracking-wider flex items-center justify-between px-1" style="color: var(--theme-text-muted);">
            <span>Unassigned Mods</span>
            <span class="font-mono text-[9px]">{sortedUncategorizedMods.length}</span>
          </div>

          {#if sortedUncategorizedMods.length === 0}
            <div class="p-6 text-center text-xs pointer-events-none italic" style="color: var(--theme-text-muted);">
              All mods are categorized!
            </div>
          {:else}
            <div class="space-y-1">
              {#each sortedUncategorizedMods as mod (mod.file_name)}
                {@const isTargetMatch = doesModMatch(mod.file_name)}
                {@const isSelected = selectedMods.has(mod.file_name)}
                <div class="relative flex flex-col">
                  {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'before' && !isSelected}
                    <div class="absolute top-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
                  {/if}

                  <div
                    onpointerdown={(e) => handlePointerDown(e, mod.file_name)}
                    oncontextmenu={(e) => openContextMenu(e, mod.file_name)}
                    ondblclick={() => onShowInLoadOrder && onShowInLoadOrder(mod.file_name)}
                    class="flex items-center justify-between px-2 py-1.5 rounded border transition density-row cursor-pointer 
                      {isSelected ? 'ring-1 z-10 relative' : (isTargetMatch ? 'ring-1' : 'hover:brightness-125')}"
                    style="{isSelected ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: var(--theme-accent); box-shadow: 0 0 15px color-mix(in srgb, var(--theme-accent) 35%, transparent);' : (isTargetMatch ? 'background-color: color-mix(in srgb, var(--color-info) 20%, transparent); border-color: var(--color-info); box-shadow: 0 0 0 1px var(--color-info);' : 'background-color: var(--theme-surface); border-color: color-mix(in srgb, var(--theme-border) 50%, transparent);')}"
                    data-mod-name={mod.file_name}
                    title="Double-click to locate in Load Order"
                  >
                    <div class="flex items-center gap-2 min-w-0">
                      <GripVertical class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                      <FileArchive class="h-3.5 w-3.5 shrink-0" style="color: {isTargetMatch ? 'var(--color-info)' : 'var(--theme-text-muted)'};" />
                      <span class="text-[11px] font-mono truncate {isTargetMatch ? 'font-bold' : (mod.enabled ? '' : 'opacity-50 line-through')}" style="color: {isTargetMatch ? 'var(--color-info)' : 'var(--theme-text-primary)'};">
                        {mod.file_name}
                      </span>
                    </div>

                    {#if notesConfig[mod.file_name]}
                      <button 
                        type="button"
                        onclick={(e) => { e.stopPropagation(); targetNoteModName = mod.file_name; isNoteModalOpen = true; }}
                        class="shrink-0 ml-2 hover:scale-110 transition-transform cursor-pointer" 
                        title={notesConfig[mod.file_name]}
                      >
                        <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
                      </button>
                    {/if}
                  </div>

                  {#if isDragging && !activeCategoryDragId && dropTargetMod === mod.file_name && dropPlacement === 'after' && !isSelected}
                    <div class="absolute bottom-0 left-0 right-0 h-[2.5px] z-30 pointer-events-none" style="background-color: var(--theme-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--theme-accent) 80%, transparent);"></div>
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

<NoteModal
  bind:isOpen={isNoteModalOpen}
  modName={targetNoteModName}
  initialNote={notesConfig[targetNoteModName] || ''}
  onConfirm={handleNoteConfirm}
  onCancel={() => isNoteModalOpen = false}
/>