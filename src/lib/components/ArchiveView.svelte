<script lang="ts">
  import type { ArchiveItem, ArchiveScanReport, XlItem } from '$lib/types';
  import CategoryDelimiterRow from '$lib/components/CategoryDelimiterRow.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import MoveCategoryModal from '$lib/components/MoveCategoryModal.svelte';
  import LinkXlModal from '$lib/components/LinkXlModal.svelte';
  import {
    GripVertical,
    ChevronDown,
    ChevronRight,
    Search,
    X,
    FolderPlus,
    Folder,
    FolderOpen,
    Eye,
    EyeOff,
    AlertTriangle,
    HelpCircle,
    FileCode,
    Info,
    FolderSearch,
    Copy,
    Power,
    Check,
    Pencil,
    Trash2,
    MoreVertical,
    FolderInput,
    Link2,
    Unlink
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick } from 'svelte';

  let {
    archives = $bindable([]),
    gamePath = '',
    scanReport = $bindable(null),
    onScanRequested,
    onStateChanged
  }: {
    archives: ArchiveItem[];
    gamePath: string;
    scanReport?: ArchiveScanReport | null;
    onScanRequested?: () => void;
    onStateChanged?: () => void;
  } = $props();

  let searchQuery = $state('');
  let expandedRows = $state<Record<string, boolean>>({});
  
  let localArchives = $state<ArchiveItem[]>([]);
  let scrollContainer = $state<HTMLElement | null>(null);
  let showXlHelp = $state(false);

  // Multi-Select State
  let selectedMods = $state<Set<string>>(new Set());
  let lastSelectedFileName = $state<string | null>(null);
  let isMoveModalOpen = $state(false);

  // XL Link State
  let isLinkModalOpen = $state(false);
  let targetXlToLink = $state<string | null>(null);

  let dialogState = $state<{
    isOpen: boolean;
    title: string;
    message: string;
    mode: 'prompt' | 'confirm';
    initialValue: string;
    confirmText: string;
    isDanger: boolean;
    onConfirm: (val: string) => void;
  }>({
    isOpen: false,
    title: '',
    message: '',
    mode: 'prompt',
    initialValue: '',
    confirmText: 'Confirm',
    isDanger: false,
    onConfirm: () => {}
  });

  let collapsedCategories = $state<Record<string, boolean>>({});
  let preDragCollapseState = $state<Record<string, boolean> | null>(null);

  let totalCategories = $derived(localArchives.filter(a => a.is_delimiter).length);
  let collapsedCount = $derived(Object.keys(collapsedCategories).filter(k => collapsedCategories[k]).length);
  let isAllCollapsed = $derived(totalCategories > 0 && collapsedCount === totalCategories);

  function toggleAllCategories() {
    if (isAllCollapsed) {
      collapsedCategories = {};
    } else {
      const newState: Record<string, boolean> = {};
      for (const a of localArchives) {
        if (a.is_delimiter) {
          newState[a.file_name] = true;
        }
      }
      collapsedCategories = newState;
    }
  }

  function toggleCategoryCollapse(categoryFileName: string) {
    collapsedCategories[categoryFileName] = !collapsedCategories[categoryFileName];
  }

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    archive: ArchiveItem | null;
    unassociatedXl: XlItem | null;
    targetType: 'archive' | 'xl' | 'unassociated_xl';
  }>({
    visible: false,
    x: 0,
    y: 0,
    archive: null,
    unassociatedXl: null,
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
  let dragBlockSize = $state<number>(1);
  let dropTargetIndex = $state<number | null>(null);
  let dropPlacement = $state<'before' | 'after' | null>(null);
  let cursorX = $state(0);
  let cursorY = $state(0);

  let scrollSpeed = 0;
  let animationFrameId: number | null = null;

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '0 B';
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
    if (archives && archives.length >= 0) {
      localArchives = [...archives];
    }
  });

  async function persistState() {
    const archiveNames = localArchives.map(a => a.file_name);
    try {
      const report = await invoke<ArchiveScanReport>('save_load_order', {
        gamePath,
        loadOrder: archiveNames
      });
      if (report && Array.isArray(report.archives)) {
        archives = report.archives;
      }
    } catch (err) {
      console.error('Failed to persist state:', err);
    }
  }

  function addCategory() {
    dialogState = {
      isOpen: true,
      title: 'Create Category',
      message: 'Enter a name for the new category delimiter (symbols like [], (), hyphens allowed):',
      mode: 'prompt',
      initialValue: 'New Category',
      confirmText: 'Create Category',
      isDanger: false,
      onConfirm: async (name: string) => {
        dialogState.isOpen = false;
        if (!name || name.trim() === '') return;
        
        try {
          const report = await invoke<ArchiveScanReport>('create_physical_category', {
            gamePath,
            categoryName: name.trim()
          });
          if (report && Array.isArray(report.archives)) {
            archives = report.archives;
            if (onStateChanged) onStateChanged();
            
            const newFileName = `[CAT] ${name.trim()}.archive`;
            setTimeout(() => {
              focusModInMainList(newFileName);
            }, 100);
          }
        } catch (err) {
          console.error("Failed to create physical category:", err);
        }
      }
    };
  }

  function handleRenameCategory(archive: ArchiveItem) {
    const currentName = archive.category_name || archive.file_name.replace('[CAT] ', '').replace('.archive', '');
    dialogState = {
      isOpen: true,
      title: 'Rename Category',
      message: 'Enter a new name for this category delimiter (symbols like [], (), hyphens allowed):',
      mode: 'prompt',
      initialValue: currentName,
      confirmText: 'Rename',
      isDanger: false,
      onConfirm: async (newName: string) => {
        dialogState.isOpen = false;
        if (!newName || newName.trim() === '' || newName.trim() === currentName) return;

        try {
          const report = await invoke<ArchiveScanReport>('rename_physical_category', {
            gamePath,
            oldFileName: archive.file_name,
            newCategoryName: newName.trim()
          });
          if (report && Array.isArray(report.archives)) {
            const wasCollapsed = !!collapsedCategories[archive.file_name];
            delete collapsedCategories[archive.file_name];
            const newFileName = `[CAT] ${newName.trim()}.archive`;
            if (wasCollapsed) {
              collapsedCategories[newFileName] = true;
            }

            archives = report.archives;
            if (onStateChanged) onStateChanged();
          }
        } catch (err) {
          console.error("Failed to rename physical category:", err);
        }
      }
    };
  }

  function handleDeleteCategory(archive: ArchiveItem) {
    dialogState = {
      isOpen: true,
      title: 'Delete Category',
      message: 'Are you sure you wish to delete this category? The category marker will be removed, but all contained mods will remain preserved in their exact sequential order.',
      mode: 'confirm',
      initialValue: '',
      confirmText: 'Delete Category',
      isDanger: true,
      onConfirm: async () => {
        dialogState.isOpen = false;
        try {
          const report = await invoke<ArchiveScanReport>('delete_physical_category', {
            gamePath,
            categoryFileName: archive.file_name
          });
          if (report && Array.isArray(report.archives)) {
            delete collapsedCategories[archive.file_name];
            archives = report.archives;
            if (onStateChanged) onStateChanged();
          }
        } catch (err) {
          console.error("Failed to delete physical category:", err);
        }
      }
    };
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

      if (archive.is_delimiter) {
        const startIndex = localArchives.findIndex(a => a.file_name === archive.file_name);
        if (startIndex !== -1) {
          for (let i = startIndex + 1; i < localArchives.length; i++) {
            const child = localArchives[i];
            if (child.is_delimiter) break;
            
            if (child.enabled !== next) {
              child.enabled = next;
              await invoke('toggle_mod_state', {
                gamePath,
                modName: child.file_name,
                enable: next
              });
            }
          }
        }
      }

      persistState();
      if (onStateChanged) onStateChanged();
    } catch (err) {
      console.error('Failed to toggle mod/category:', err);
    }
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

    let size = 1;
    if (localArchives[index].is_delimiter) {
      for (let i = index + 1; i < localArchives.length; i++) {
        if (localArchives[i].is_delimiter) break;
        size++;
      }

      preDragCollapseState = { ...collapsedCategories };

      if (size > 1) {
        collapsedCategories = {
          ...collapsedCategories,
          [localArchives[index].file_name]: true
        };
      }
    }
    dragBlockSize = size;
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
    
    if (index >= activeDragIndex && index < activeDragIndex + dragBlockSize) return;

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
        const targetItem = localArchives[dropTargetIndex];
        if (targetItem.is_delimiter && collapsedCategories[targetItem.file_name]) {
          let targetBlockEnd = dropTargetIndex;
          for (let i = dropTargetIndex + 1; i < localArchives.length; i++) {
            if (localArchives[i].is_delimiter) break;
            targetBlockEnd++;
          }
          targetIndex = targetBlockEnd + 1;
        } else {
          targetIndex += 1;
        }
      }
      
      if (activeDragIndex < targetIndex) {
        if (targetIndex > activeDragIndex + dragBlockSize) {
          targetIndex -= dragBlockSize;
        } else {
          targetIndex = activeDragIndex;
        }
      }

      if (activeDragIndex !== targetIndex) {
        const updated = [...localArchives];
        const movedItems = updated.splice(activeDragIndex, dragBlockSize);
        updated.splice(targetIndex, 0, ...movedItems);
        
        localArchives = updated;
        persistState();
      }
    }
    
    if (preDragCollapseState !== null) {
      collapsedCategories = { ...preDragCollapseState };
      preDragCollapseState = null;
    }

    activeDragIndex = null;
    dragBlockSize = 1;
    dropTargetIndex = null;
    dropPlacement = null;
  }

  function handleModClick(event: MouseEvent, archive: ArchiveItem) {
    if (archive.is_delimiter) return;

    const isCtrl = event.ctrlKey || event.metaKey;
    const isShift = event.shiftKey;

    if (isShift && lastSelectedFileName) {
      const visibleMods = visibleItems.filter(v => !v.archive.is_delimiter);
      const startIdx = visibleMods.findIndex(v => v.archive.file_name === lastSelectedFileName);
      const endIdx = visibleMods.findIndex(v => v.archive.file_name === archive.file_name);

      if (startIdx !== -1 && endIdx !== -1) {
        const min = Math.min(startIdx, endIdx);
        const max = Math.max(startIdx, endIdx);
        if (!isCtrl) selectedMods.clear();
        for (let i = min; i <= max; i++) {
          selectedMods.add(visibleMods[i].archive.file_name);
        }
      }
    } else if (isCtrl) {
      if (selectedMods.has(archive.file_name)) {
        selectedMods.delete(archive.file_name);
      } else {
        selectedMods.add(archive.file_name);
      }
      lastSelectedFileName = archive.file_name;
    } else {
      selectedMods.clear();
      selectedMods.add(archive.file_name);
      lastSelectedFileName = archive.file_name;
    }
    selectedMods = new Set(selectedMods);
  }

  function handleMoveToCategory(targetCategoryFileName: string) {
    isMoveModalOpen = false;
    if (selectedMods.size === 0) return;

    const updated = [...localArchives];
    const extracted: ArchiveItem[] = [];
    
    for (let i = updated.length - 1; i >= 0; i--) {
      if (selectedMods.has(updated[i].file_name)) {
        extracted.unshift(updated.splice(i, 1)[0]);
      }
    }

    let insertIndex = 0;
    if (targetCategoryFileName !== '[TOP]') {
      const catIndex = updated.findIndex(a => a.file_name === targetCategoryFileName);
      if (catIndex !== -1) {
        insertIndex = catIndex + 1;
        while (insertIndex < updated.length && !updated[insertIndex].is_delimiter) {
          insertIndex++;
        }
      }
    }

    updated.splice(insertIndex, 0, ...extracted);
    localArchives = updated;
    persistState();
    
    selectedMods.clear();
    selectedMods = new Set();
  }

  async function handleLinkXl(archiveFileName: string) {
    isLinkModalOpen = false;
    if (!targetXlToLink) return;
    try {
      const report = await invoke<ArchiveScanReport>('link_xl_to_archive', {
        gamePath,
        xlName: targetXlToLink,
        archiveName: archiveFileName
      });
      archives = report.archives;
      scanReport = report;
      if (onStateChanged) onStateChanged();
    } catch (err) {
      console.error('Failed to link XL:', err);
    }
    targetXlToLink = null;
  }

  async function handleUnlinkXl() {
    if (!contextMenu.archive || !contextMenu.archive.associated_xls) return;
    try {
      let report: ArchiveScanReport | null = null;
      for (const xl of contextMenu.archive.associated_xls) {
        report = await invoke<ArchiveScanReport>('unlink_xl_from_archive', {
          gamePath,
          xlName: xl.file_name
        });
      }
      if (report) {
        archives = report.archives;
        scanReport = report;
        if (onStateChanged) onStateChanged();
      }
    } catch (err) {
      console.error('Failed to unlink XL:', err);
    }
    closeContextMenu();
  }

  function openContextMenu(event: MouseEvent, target: ArchiveItem | XlItem, targetType: 'archive' | 'xl' | 'unassociated_xl' = 'archive') {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    if (targetType === 'archive' && !('size_bytes' in target && !('file_count' in target))) {
      const archive = target as ArchiveItem;
      if (!archive.is_delimiter) {
        if (!selectedMods.has(archive.file_name)) {
          selectedMods.clear();
          selectedMods.add(archive.file_name);
          selectedMods = new Set(selectedMods);
          lastSelectedFileName = archive.file_name;
        }
      }
    }

    const menuWidth = 230;
    let menuHeight = 145;
    if (targetType === 'unassociated_xl') menuHeight = 145;
    else if (targetType === 'xl') menuHeight = 180;
    else if (targetType === 'archive') {
      const archive = target as ArchiveItem;
      menuHeight = archive.is_delimiter ? 190 : (selectedMods.size > 0 ? 180 : 145);
    }

    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    if (targetType === 'unassociated_xl') {
      contextMenu = { visible: true, x: posX, y: posY, archive: null, unassociatedXl: target as XlItem, targetType };
    } else {
      contextMenu = { visible: true, x: posX, y: posY, archive: target as ArchiveItem, unassociatedXl: null, targetType };
    }
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.archive = null;
      contextMenu.unassociatedXl = null;
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
    if (!gamePath) return;
    let targetFilename = '';
    if (contextMenu.targetType === 'unassociated_xl' && contextMenu.unassociatedXl) {
      targetFilename = contextMenu.unassociatedXl.file_name;
    } else if (contextMenu.archive) {
      targetFilename = contextMenu.archive.file_name;
      if (contextMenu.targetType === 'xl' && contextMenu.archive.associated_xls?.length) {
        targetFilename = contextMenu.archive.associated_xls[0].file_name;
      }
    }
    if (!targetFilename) return;

    const fullPath = `${gamePath}\\archive\\pc\\mod\\${targetFilename}`;
    try {
      await revealItemInDir(fullPath);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeContextMenu();
  }

  async function handleContextMenuCopyName() {
    let targetFilename = '';
    if (contextMenu.targetType === 'unassociated_xl' && contextMenu.unassociatedXl) {
      targetFilename = contextMenu.unassociatedXl.file_name;
    } else if (contextMenu.archive) {
      targetFilename = contextMenu.archive.file_name;
      if (contextMenu.targetType === 'xl' && contextMenu.archive.associated_xls?.length) {
        targetFilename = contextMenu.archive.associated_xls.map(x => x.file_name).join(', ');
      }
    }
    if (!targetFilename) return;

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

  let availableCategories = $derived.by(() => {
    const cats = localArchives.filter(a => a.is_delimiter).map(a => ({
      name: a.category_name || a.file_name.replace('[CAT] ', '').replace('.archive', ''),
      file_name: a.file_name
    }));
    return [{ name: 'Top (Uncategorized)', file_name: '[TOP]' }, ...cats];
  });

  let availableArchivesForLink = $derived.by(() => {
    return localArchives.filter(a => !a.is_delimiter).map(a => ({
      name: a.name,
      file_name: a.file_name
    }));
  });

  let categoryCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    let currentCat: string | null = null;
    let currentCount = 0;

    for (const item of localArchives) {
      if (item.is_delimiter) {
        if (currentCat) {
          counts.set(currentCat, currentCount);
        }
        currentCat = item.file_name;
        currentCount = 0;
      } else if (currentCat) {
        currentCount++;
      }
    }
    if (currentCat) {
      counts.set(currentCat, currentCount);
    }
    return counts;
  });

  let previewConflictState = $derived.by(() => {
    if (activeDragIndex === null || dropTargetIndex === null || dropPlacement === null) {
      return null;
    }

    let targetIndex = dropTargetIndex;
    
    if (dropPlacement === 'after') {
      const targetItem = localArchives[dropTargetIndex];
      if (targetItem.is_delimiter && collapsedCategories[targetItem.file_name]) {
        let targetBlockEnd = dropTargetIndex;
        for (let i = dropTargetIndex + 1; i < localArchives.length; i++) {
          if (localArchives[i].is_delimiter) break;
          targetBlockEnd++;
        }
        targetIndex = targetBlockEnd + 1;
      } else {
        targetIndex += 1;
      }
    }
    
    if (activeDragIndex < targetIndex) {
      if (targetIndex > activeDragIndex + dragBlockSize) {
        targetIndex -= dragBlockSize;
      } else {
        targetIndex = activeDragIndex;
      }
    }

    if (activeDragIndex === targetIndex) {
      return null;
    }

    const updated = [...localArchives];
    const movedItems = updated.splice(activeDragIndex, dragBlockSize);
    updated.splice(targetIndex, 0, ...movedItems);

    const indexMap = new Map<string, number>();
    for (let i = 0; i < updated.length; i++) {
      indexMap.set(updated[i].file_name, i);
    }

    const stateMap = new Map<string, { wins: string[], loses: string[] }>();
    
    for (let i = 0; i < updated.length; i++) {
      const item = updated[i];
      if (item.has_conflicts) {
        const newWins: string[] = [];
        const newLoses: string[] = [];
        
        for (const rival of item.conflicts_with) {
          const rivalIndex = indexMap.get(rival);
          if (rivalIndex !== undefined) {
            if (i < rivalIndex) {
              newWins.push(rival);
            } else {
              newLoses.push(rival);
            }
          }
        }
        stateMap.set(item.file_name, { wins: newWins, loses: newLoses });
      }
    }

    return stateMap;
  });

  let visibleItems = $derived.by(() => {
    const result: { archive: ArchiveItem; originalIndex: number; archiveRank: number; inCategory: boolean }[] = [];
    let rank = 0;
    let currentCategoryCollapsed = false;
    let inCategory = false;

    for (let i = 0; i < localArchives.length; i++) {
      const archive = localArchives[i];

      if (archive.is_delimiter) {
        currentCategoryCollapsed = !!collapsedCategories[archive.file_name];
        inCategory = true;
        if (searchQuery === '' || archive.name.toLowerCase().includes(searchQuery.toLowerCase())) {
          result.push({ 
            archive, 
            originalIndex: i, 
            archiveRank: 0,
            inCategory: false
          });
        }
      } else {
        rank += 1;
        if (!currentCategoryCollapsed || searchQuery !== '') {
          if (searchQuery === '' || archive.name.toLowerCase().includes(searchQuery.toLowerCase())) {
            result.push({ 
              archive, 
              originalIndex: i, 
              archiveRank: rank,
              inCategory
            });
          }
        }
      }
    }
    return result;
  });

  let conflictingArchives = $derived.by(() => {
    return localArchives
      .filter(a => a.has_conflicts)
      .map(a => {
        if (previewConflictState && previewConflictState.has(a.file_name)) {
          const preview = previewConflictState.get(a.file_name)!;
          return { ...a, wins: preview.wins, loses: preview.loses };
        }
        return a;
      })
      .filter(a => a.wins.length > 0 || a.loses.length > 0);
  });

  let draggedEntry = $derived(
    activeDragIndex !== null ? localArchives[activeDragIndex] : null
  );

  let unassociatedXlFiles = $derived(
    (scanReport?.unassociated_xl ?? []).filter(item => {
      if (!searchQuery.trim()) return true;
      return item.file_name.toLowerCase().includes(searchQuery.toLowerCase());
    })
  );

  let actualMods = $derived(archives.filter(a => !a.is_delimiter));
  let activeCount = $derived(actualMods.filter(a => a.enabled).length);
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={completeDrop}
  onclick={closeContextMenu}
/>

{#if activeDragIndex !== null && draggedEntry}
  <div
    class="fixed pointer-events-none z-50 px-3 py-1.5 rounded-lg bg-nvidia-card border border-nvidia-accent shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
    style="left: {cursorX}px; top: {cursorY}px;"
  >
    <GripVertical class="h-3.5 w-3.5 text-nvidia-accent" />
    {#if draggedEntry.is_delimiter}
      <Folder class="h-3.5 w-3.5 text-nvidia-accent" />
      <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">
        {draggedEntry.category_name || draggedEntry.file_name.replace('[CAT] ', '').replace('.archive', '')}
        {#if dragBlockSize > 1}
          <span class="text-nvidia-accent ml-1 font-mono">({dragBlockSize - 1} mods)</span>
        {/if}
      </span>
    {:else}
      <span class="text-xs font-mono font-semibold text-nvidia-text-primary">{draggedEntry.file_name}</span>
    {/if}
  </div>
{/if}

<!-- Custom Context Menu -->
{#if contextMenu.visible && (contextMenu.archive || contextMenu.unassociatedXl)}
  {@const activeTargetName = contextMenu.targetType === 'unassociated_xl' 
    ? contextMenu.unassociatedXl!.file_name 
    : (contextMenu.targetType === 'xl' && contextMenu.archive!.associated_xls?.length)
      ? (contextMenu.archive!.associated_xls.length === 1 ? contextMenu.archive!.associated_xls[0].file_name : `${contextMenu.archive!.associated_xls.length} Companion Files`)
      : contextMenu.archive!.file_name}
  {@const isActive = contextMenu.targetType === 'unassociated_xl' ? contextMenu.unassociatedXl!.enabled : contextMenu.archive!.enabled}
  
  <div
    class="fixed z-50 w-56 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center justify-between gap-2">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.targetType === 'xl' || contextMenu.targetType === 'unassociated_xl'}
          <FileCode class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
        {:else if contextMenu.archive!.is_delimiter}
          <Folder class="h-3.5 w-3.5 text-nvidia-accent shrink-0" />
        {:else}
          <div class="h-2 w-2 rounded-full shrink-0 {isActive ? 'bg-nvidia-accent' : 'bg-red-500'}"></div>
        {/if}
        <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={activeTargetName}>
          {activeTargetName}
        </span>
      </div>
      <span class="text-[9px] font-mono uppercase px-1 py-0.2 rounded border {isActive ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-red-500/15 border-red-500/40 text-red-400'} shrink-0">
        {isActive ? 'Active' : 'Disabled'}
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

      {#if contextMenu.targetType === 'unassociated_xl'}
        <button
          type="button"
          onclick={() => { targetXlToLink = contextMenu.unassociatedXl!.file_name; isLinkModalOpen = true; closeContextMenu(); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <Link2 class="h-3.5 w-3.5 text-cyan-400" />
          <span>Link to Archive...</span>
        </button>
      {:else}
        <button
          type="button"
          onclick={handleContextMenuToggle}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <Power class="h-3.5 w-3.5 {isActive ? 'text-amber-400' : 'text-nvidia-accent'}" />
          <span>{isActive ? (contextMenu.archive!.is_delimiter ? 'Disable Category' : 'Disable Mod') : (contextMenu.archive!.is_delimiter ? 'Enable Category' : 'Enable Mod')}</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'archive' && contextMenu.archive!.is_delimiter}
        <button
          type="button"
          onclick={() => { const item = contextMenu.archive; closeContextMenu(); if (item) handleRenameCategory(item); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <Pencil class="h-3.5 w-3.5 text-nvidia-accent" />
          <span>Rename Category</span>
        </button>

        <button
          type="button"
          onclick={() => { const item = contextMenu.archive; closeContextMenu(); if (item) handleDeleteCategory(item); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-red-500/10 text-red-400 transition text-left cursor-pointer group"
        >
          <Trash2 class="h-3.5 w-3.5 text-red-400" />
          <span>Delete Category</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'archive' && !contextMenu.archive!.is_delimiter && selectedMods.size > 0}
        <button
          type="button"
          onclick={() => { closeContextMenu(); isMoveModalOpen = true; }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <FolderInput class="h-3.5 w-3.5 text-nvidia-accent" />
          <span>Move {selectedMods.size} Mods to Category...</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'xl'}
        <button
          type="button"
          onclick={handleUnlinkXl}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <Unlink class="h-3.5 w-3.5 text-amber-400" />
          <span>Unlink .xl</span>
        </button>
      {/if}

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
            <span>Copy File Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="space-y-3 select-none flex flex-col h-full" oncontextmenu={(e) => e.preventDefault()}>
  <div class="flex items-center justify-between gap-4 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="h-4 w-4 absolute left-3 top-1/2 -translate-y-1/2 text-nvidia-text-muted" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter mods or categories..."
          class="w-full pl-9 pr-8 py-1.5 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder-nvidia-text-muted focus:outline-none focus:border-nvidia-accent"
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
        Active: <span class="text-nvidia-accent font-semibold">{activeCount}</span> of {actualMods.length}
      </div>
    </div>

    <div class="flex items-center gap-2.5">
      <button
        onclick={toggleAllCategories}
        disabled={totalCategories === 0}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary font-medium transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if isAllCollapsed}
          <FolderOpen class="h-3.5 w-3.5 text-nvidia-accent" />
          <span>Expand All</span>
        {:else}
          <Folder class="h-3.5 w-3.5 text-nvidia-accent" />
          <span>Collapse All</span>
        {/if}
      </button>

      <button
        onclick={addCategory}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border text-xs text-nvidia-text-primary font-medium transition cursor-pointer"
      >
        <FolderPlus class="h-3.5 w-3.5 text-nvidia-accent" />
        <span>Add Category</span>
      </button>

      <button
        onclick={toggleSummaryDrawer}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded border text-xs font-medium transition cursor-pointer {showConflictSummary ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-nvidia-surface hover:bg-nvidia-card border-nvidia-border text-nvidia-text-muted hover:text-nvidia-text-primary'}"
        title="Toggle Conflict Summary Column"
      >
        {#if showConflictSummary}
          <Eye class="h-3.5 w-3.5" />
        {:else}
          <EyeOff class="h-3.5 w-3.5" />
        {/if}
        <span>Conflict Summary</span>
      </button>
    </div>
  </div>

  <div class="flex gap-3 flex-1 overflow-hidden min-h-0">
    <div
      bind:this={scrollContainer}
      class="space-y-1 overflow-y-auto pr-1 pt-2 flex-1 max-h-[calc(100vh-170px)]"
    >
      {#if visibleItems.length === 0}
        <div class="p-6 text-center text-xs text-nvidia-text-muted border border-nvidia-border rounded bg-nvidia-surface">
          No matches found.
        </div>
      {:else}
        {#each visibleItems as { archive, originalIndex, archiveRank, inCategory } (archive.file_name)}
          {@const isSource = activeDragIndex !== null && originalIndex >= activeDragIndex && originalIndex < activeDragIndex + dragBlockSize}
          {@const showLineBefore = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'before' && !isSource}
          {@const showLineAfter = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'after' && !isSource}

          <div class="relative flex flex-col {inCategory ? 'ml-3' : ''}">
            {#if inCategory}
              <div class="absolute -left-2 -top-1 bottom-0 w-[1px] bg-nvidia-border/45 z-0"></div>
              <div class="absolute -left-2 top-1/2 w-2 h-[1px] bg-nvidia-border/45 z-0"></div>
            {/if}

            {#if showLineBefore}
              <div class="absolute {originalIndex === 0 ? '-top-1.5' : '-top-1'} left-0 right-0 h-[2.5px] bg-nvidia-accent z-30 shadow-[0_0_12px_var(--theme-accent)] flex items-center">
                <div class="h-2.5 w-2.5 rounded-full bg-nvidia-accent -ml-1.5 shadow-[0_0_8px_var(--theme-accent)]"></div>
              </div>
            {/if}

            {#if archive.is_delimiter}
              <CategoryDelimiterRow
                {archive}
                {originalIndex}
                {isSource}
                collapsed={!!collapsedCategories[archive.file_name]}
                modCount={categoryCounts.get(archive.file_name) ?? 0}
                onToggleCollapse={toggleCategoryCollapse}
                isHighlighted={highlightedModName === archive.file_name}
                onDragStart={startDrag}
                onPointerMove={onRowPointerMove}
                onToggle={toggleMod}
                onContextMenu={openContextMenu}
                onRenameCategory={handleRenameCategory}
                onDeleteCategory={handleDeleteCategory}
                registerNode={registerModNode}
              />
            {:else}
              {@const isExpanded = !!expandedRows[archive.file_name]}
              {@const isHighlighted = highlightedModName === archive.file_name}
              {@const isSelected = selectedMods.has(archive.file_name)}
              {@const activeConflicts = previewConflictState?.get(archive.file_name) || { wins: archive.wins, loses: archive.loses }}

              <div
                role="group"
                aria-label="Archive mod item: {archive.file_name}"
                use:registerModNode={archive.file_name}
                onclick={(e) => handleModClick(e, archive)}
                onpointermove={(e) => onRowPointerMove(e, originalIndex)}
                oncontextmenu={(e) => openContextMenu(e, archive, 'archive')}
                class="group rounded border transition-all duration-300 z-10 {isHighlighted ? 'border-[#76b900] ring-2 ring-[#76b900] bg-[#76b900]/20 shadow-[0_0_15px_rgba(118,185,0,0.35)] scale-[1.008]' : (isSelected ? 'border-nvidia-accent/60 bg-nvidia-accent/10' : 'border-nvidia-border/70 bg-nvidia-surface hover:border-nvidia-border')} {archive.enabled ? 'opacity-100' : 'opacity-40'} {isSource ? 'opacity-20 border-dashed border-nvidia-accent/50' : ''}"
              >
                <!-- Density-aware Row Container (28px) -->
                <div class="flex items-center justify-between px-3 gap-2 density-row">
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

                    <!-- Subtle, Neutral Calmed Switch -->
                    <button
                      onclick={(e) => { e.stopPropagation(); toggleMod(archive); }}
                      aria-label={archive.enabled ? "Disable mod " + archive.file_name : "Enable mod " + archive.file_name}
                      class="w-7 h-4 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer {archive.enabled ? 'bg-zinc-700/80 border border-zinc-600' : 'bg-zinc-900/90 border border-zinc-800'}"
                    >
                      <div class="h-2.5 w-2.5 rounded-full transition-transform transform {archive.enabled ? 'translate-x-3 bg-zinc-100 shadow-xs' : 'translate-x-0 bg-zinc-500'}"></div>
                    </button>

                    <span class="text-[10px] font-mono px-1 py-0.2 rounded bg-nvidia-card text-nvidia-text-muted border border-nvidia-border shrink-0">
                      #{archiveRank}
                    </span>

                    <span class="font-mono font-medium text-nvidia-text-primary truncate max-w-md">
                      {archive.file_name}
                    </span>

                    {#if archive.associated_xls && archive.associated_xls.length > 0}
                      <span
                        role="button"
                        tabindex="0"
                        oncontextmenu={(e) => { e.stopPropagation(); openContextMenu(e, archive, 'xl'); }}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                        class="text-[10px] font-mono font-bold px-1.5 py-0.2 rounded border transition-colors shrink-0 cursor-context-menu {archive.enabled ? 'bg-cyan-950/80 text-cyan-400 border-cyan-700/60 hover:border-cyan-500' : 'bg-nvidia-card text-nvidia-text-muted border-nvidia-border opacity-50'}"
                        title="Companions: {archive.associated_xls.map(x => x.file_name).join(', ')} - Right-click for options"
                      >
                        XL {#if archive.associated_xls.length > 1}({archive.associated_xls.length}){/if}
                      </span>
                    {/if}

                    <div class="flex items-center gap-1.5 text-[11px] text-nvidia-text-muted shrink-0 opacity-80">
                      <span>({archive.file_count} assets)</span>
                      <span>•</span>
                      <span>{formatBytes(archive.size_bytes)}</span>
                    </div>
                  </div>

                  <!-- Fixed-Width Laser-Aligned Conflict Column & Hover Menu -->
                  <div class="flex items-center gap-2 shrink-0 w-24">
                    <div class="w-3 flex items-center justify-center shrink-0">
                      {#if activeConflicts.loses.length > 0}
                        <div class="h-2.5 w-2.5 rounded-full bg-[#ef4444] shadow-[0_0_6px_rgba(239,68,68,0.7)]" title="Overwritten by higher mod"></div>
                      {:else if activeConflicts.wins.length > 0}
                        <div class="h-2.5 w-2.5 rounded-full bg-[#22c55e] shadow-[0_0_6px_rgba(34,197,94,0.7)]" title="Winning overwrites"></div>
                      {:else}
                        <div class="h-2.5 w-2.5 rounded-full bg-[#22c55e]/90" title="Clean (No conflicts)"></div>
                      {/if}
                    </div>

                    <div class="flex items-center min-w-0 flex-1">
                      {#if archive.has_conflicts}
                        {@const conflictCount = activeConflicts.loses.length > 0 ? activeConflicts.loses.length : activeConflicts.wins.length}
                        <button
                          type="button"
                          onclick={(e) => { e.stopPropagation(); expandedRows[archive.file_name] = !expandedRows[archive.file_name]; }}
                          class="flex items-center gap-1 px-1 py-0.5 rounded hover:bg-nvidia-card text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer font-mono text-[11px]"
                          title="Toggle conflict details ({conflictCount} impacted)"
                        >
                          <span class="font-medium">{conflictCount}</span>
                          {#if isExpanded}
                            <ChevronDown class="h-3 w-3" />
                          {:else}
                            <ChevronRight class="h-3 w-3" />
                          {/if}
                        </button>
                      {/if}
                    </div>
                  </div>

                  <!-- Hover Context Menu Button -->
                  <div class="flex items-center justify-end shrink-0 w-6">
                    <button
                      type="button"
                      onclick={(e) => { e.stopPropagation(); openContextMenu(e, archive, 'archive'); }}
                      class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-card transition cursor-pointer opacity-0 group-hover:opacity-100"
                      title="More options"
                    >
                      <MoreVertical class="h-3.5 w-3.5" />
                    </button>
                  </div>
                </div>

                {#if isExpanded && archive.has_conflicts}
                  <div class="px-4 py-2 border-t border-nvidia-border/60 bg-nvidia-card/30 space-y-2 text-xs">
                    {#if activeConflicts.wins.length > 0}
                      <div>
                        <span class="text-[10px] font-semibold text-[#22c55e] uppercase tracking-wider">Overwrites Lower Mods:</span>
                        <div class="mt-1 flex flex-wrap gap-1">
                          {#each activeConflicts.wins as target}
                            <span class="px-1.5 py-0.2 rounded bg-nvidia-surface border border-nvidia-border text-[10px] font-mono text-nvidia-text-primary">
                              {target}
                            </span>
                          {/each}
                        </div>
                      </div>
                    {/if}

                    {#if activeConflicts.loses.length > 0}
                      <div>
                        <span class="text-[10px] font-semibold text-[#ef4444] uppercase tracking-wider">Loses To Higher Mods:</span>
                        <div class="mt-1 flex flex-wrap gap-1">
                          {#each activeConflicts.loses as target}
                            <span class="px-1.5 py-0.2 rounded bg-[#ef4444]/15 border border-[#ef4444]/40 text-[10px] font-mono text-[#ef4444] font-semibold">
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
              <div class="absolute -bottom-1 left-0 right-0 h-[2.5px] bg-nvidia-accent z-30 shadow-[0_0_12px_var(--theme-accent)] flex items-center">
                <div class="h-2.5 w-2.5 rounded-full bg-nvidia-accent -ml-1.5 shadow-[0_0_8px_var(--theme-accent)]"></div>
              </div>
            {/if}
          </div>
        {/each}
      {/if}

      <!-- Dedicated Bottom Section: Unassociated .xl Files -->
      {#if unassociatedXlFiles.length > 0}
        <div class="mt-6 pt-4 border-t border-nvidia-border/60 space-y-2">
          <div class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-2 flex items-center justify-between">
            <div class="flex items-center gap-2 min-w-0">
              <FileCode class="h-4 w-4 text-cyan-400 shrink-0" />
              <span class="text-xs font-bold text-cyan-400 uppercase tracking-wider">
                Unassociated .xl Files
              </span>
              <span class="text-[10px] text-nvidia-text-muted hidden sm:inline">
                (Active in game, without a systemically generated mod association)
              </span>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <button
                type="button"
                onclick={() => showXlHelp = !showXlHelp}
                class="flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium border transition-colors {showXlHelp ? 'bg-cyan-950 text-cyan-300 border-cyan-600' : 'bg-nvidia-surface hover:bg-nvidia-card text-cyan-400 border-cyan-500/40'}"
                title="Click for association instructions"
              >
                <Info class="h-3.5 w-3.5" />
                <span>Info</span>
              </button>

              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nvidia-surface border border-nvidia-border text-cyan-400">
                {unassociatedXlFiles.length} loose
              </span>
            </div>
          </div>

          {#if showXlHelp}
            <div class="p-3 rounded-lg border border-cyan-500/30 bg-cyan-500/5 text-xs text-nvidia-text-primary space-y-1.5">
              <p class="text-[11px] text-nvidia-text-muted pl-6 leading-relaxed">
                <span class="text-nvidia-text-primary font-medium">Not required for the game to function. An organizational quality-of-life feature only. ArchiveXL will load untethered .xl files regardless.</span>
                <br /><br />
                <span class="text-cyan-400 font-semibold">What is Linking?</span><br />
                Linking bonds a loose <span class="font-mono text-nvidia-text-primary">.xl</span> file to a parent <span class="font-mono text-nvidia-text-primary">.archive</span> mod. Once linked, the parent mod will display a blue <span class="font-mono text-cyan-400 font-bold">[XL]</span> badge, and toggling the parent mod will automatically toggle the companion file.
                <br /><br />
                <span class="text-cyan-400 font-semibold">How to Link / Unlink:</span><br />
                • <span class="font-semibold text-nvidia-text-primary">Link:</span> Click the "Active / Click to Link" button on any loose file below, or right-click it and select "Link to Archive...".<br />
                • <span class="font-semibold text-nvidia-text-primary">Unlink:</span> Right-click the blue <span class="font-mono text-cyan-400 font-bold">[XL]</span> badge on a parent mod and select "Unlink .xl".
              </p>
            </div>
          {/if}

          <div class="space-y-1">
            {#each unassociatedXlFiles as xl (xl.file_name)}
              <div
                oncontextmenu={(e) => openContextMenu(e, xl, 'unassociated_xl')}
                class="rounded border border-nvidia-border bg-nvidia-surface/80 hover:bg-nvidia-surface px-3 flex items-center justify-between density-row cursor-context-menu"
              >
                <div class="flex items-center gap-2.5 min-w-0">
                  <FileCode class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
                  <span class="font-mono text-nvidia-text-primary font-medium truncate">{xl.file_name}</span>
                  <span class="text-[11px] font-mono text-nvidia-text-muted shrink-0">
                    {formatBytes(xl.size_bytes)}
                  </span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <button
                    type="button"
                    onclick={(e) => { e.stopPropagation(); targetXlToLink = xl.file_name; isLinkModalOpen = true; }}
                    class="text-[10px] font-mono px-2 py-0.5 rounded bg-cyan-500/15 hover:bg-cyan-500/25 border border-cyan-500/30 hover:border-cyan-400 text-cyan-400 font-semibold transition cursor-pointer"
                  >
                    Active / Click to Link
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Conflict Summary Drawer -->
    {#if showConflictSummary}
      <aside class="w-80 rounded-lg border border-nvidia-border bg-nvidia-surface flex flex-col overflow-hidden shrink-0 shadow-xl transition-all duration-200">
        <div class="p-3 border-b border-nvidia-border bg-nvidia-card/50 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <AlertTriangle class="h-4 w-4 text-[#ef4444]" />
            <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Conflict Summary</span>
          </div>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nvidia-surface border border-nvidia-border text-nvidia-text-primary">
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
              {@const isDragged = draggedEntry?.file_name === archive.file_name}
              <button
                type="button"
                onclick={() => focusModInMainList(archive.file_name)}
                class="w-full text-left rounded px-2.5 flex items-center justify-between gap-2 border transition-all duration-200 cursor-pointer density-row group
                  {isDragged 
                    ? (isLosing 
                        ? 'border-[#ef4444] bg-[#ef4444]/15 shadow-[0_0_12px_rgba(239,68,68,0.3)] scale-[1.02] z-10 relative' 
                        : 'border-[#22c55e] bg-[#22c55e]/15 shadow-[0_0_12px_rgba(34,197,94,0.3)] scale-[1.02] z-10 relative')
                    : 'border-nvidia-border/70 bg-nvidia-surface/80 hover:bg-nvidia-surface hover:border-nvidia-border'}"
                title="Click to locate in main load order"
              >
                <div class="flex items-center gap-2 min-w-0 flex-1">
                  <div class="h-2 w-2 rounded-full shrink-0 {isLosing ? 'bg-[#ef4444] shadow-[0_0_8px_rgba(239,68,68,0.7)]' : 'bg-[#22c55e] shadow-[0_0_8px_rgba(34,197,94,0.7)]'}"></div>

                  <span class="font-mono text-nvidia-text-primary truncate font-medium">
                    {archive.file_name}
                  </span>
                </div>

                <div class="shrink-0">
                  {#if isLosing}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.2 rounded bg-[#ef4444]/15 text-[#ef4444] border border-[#ef4444]/30">
                      -{archive.loses.length}
                    </span>
                  {:else}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.2 rounded bg-[#22c55e]/15 text-[#22c55e] border border-[#22c55e]/30">
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

<DialogModal
  bind:isOpen={dialogState.isOpen}
  title={dialogState.title}
  message={dialogState.message}
  mode={dialogState.mode}
  initialValue={dialogState.initialValue}
  confirmText={dialogState.confirmText}
  isDanger={dialogState.isDanger}
  onConfirm={dialogState.onConfirm}
/>

<MoveCategoryModal
  bind:isOpen={isMoveModalOpen}
  categories={availableCategories}
  onConfirm={handleMoveToCategory}
  onCancel={() => isMoveModalOpen = false}
/>

<LinkXlModal
  bind:isOpen={isLinkModalOpen}
  xlName={targetXlToLink || ''}
  archives={availableArchivesForLink}
  onConfirm={handleLinkXl}
  onCancel={() => isLinkModalOpen = false}
/>
