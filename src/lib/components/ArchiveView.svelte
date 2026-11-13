<script lang="ts">
  import type { ArchiveItem, ArchiveScanReport, XlItem, LabelsConfig } from '$lib/types';
  import FolderDelimiterRow from '$lib/components/FolderDelimiterRow.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import MoveFolderModal from '$lib/components/MoveFolderModal.svelte';
  import LinkXlModal from '$lib/components/LinkXlModal.svelte';
  import OrganizerView from '$lib/components/OrganizerView.svelte';
  import CategoryModal from '$lib/components/CategoryModal.svelte';
  import MoveCategoryModal from '$lib/components/MoveCategoryModal.svelte';
  import NoteModal from '$lib/components/NoteModal.svelte';
  import {
    GripVertical, ChevronDown, ChevronRight, Search, X, FolderPlus, Folder, FolderOpen,
    Eye, EyeOff, AlertTriangle, HelpCircle, FileCode, Info, FolderSearch, Copy, Power,
    Check, Pencil, Trash2, MoreVertical, FolderInput, Link2, Unlink, ArrowUp, LayoutGrid,
    ListTree, Clock, Tag, MessageSquare
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick, untrack } from 'svelte';

  let {
    archives = $bindable([]),
    gamePath = '',
    scanReport = $bindable(null),
    onScanRequested,
    onStateChanged,
    targetHighlightMod = null,
    onRouteToHistory,
    enableOrganizerView = false,
    showCategoryLabelsInLoadOrder = true,
    onLabelsToggle,
    currentTab = 'archive',
    globalSelectedPath = $bindable('Ready')
  }: {
    archives: ArchiveItem[];
    gamePath: string;
    scanReport?: ArchiveScanReport | null;
    onScanRequested?: () => void;
    onStateChanged?: () => void;
    targetHighlightMod?: string | null;
    onRouteToHistory?: (modName: string) => void;
    enableOrganizerView?: boolean;
    showCategoryLabelsInLoadOrder?: boolean;
    onLabelsToggle?: (val: boolean) => void;
    currentTab?: string;
    globalSelectedPath?: string;
  } = $props();

  let searchQuery = $state('');
  let expandedRows = $state<Record<string, boolean>>({});
  
  let localArchives = $state<ArchiveItem[]>([]);
  let scrollContainer = $state<HTMLElement | null>(null);
  let showXlHelp = $state(false);
  let showUnassociatedXl = $state(false);

  let labelsConfig = $state<LabelsConfig>({ categories: [], assignments: {}, orders: {} });
  let notesConfig = $state<Record<string, string>>({});
  let activeViewMode = $state<'load_order' | 'organizer'>('load_order');
  let showUncategorizedDrawer = $state(true);

  let selectedMods = $state<Set<string>>(new Set());
  let lastSelectedFileName = $state<string | null>(null);
  let isMoveModalOpen = $state(false);

  let isCategoryModalOpen = $state(false);
  let categoryModalMode = $state<'create' | 'edit'>('create');
  let categoryModalData = $state({ id: '', name: '', color: '#06B6D4' });
  let isMoveCategoryModalOpen = $state(false);

  let isLinkModalOpen = $state(false);
  let targetXlToLink = $state<string | null>(null);
  
  let isNoteModalOpen = $state(false);
  let targetNoteModName = $state('');

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
    isOpen: false, title: '', message: '', mode: 'prompt', initialValue: '', confirmText: 'Confirm', isDanger: false, onConfirm: () => {}
  });

  let collapsedFolders = $state<Record<string, boolean>>({});
  let preDragCollapseState = $state<Record<string, boolean> | null>(null);

  let totalFolders = $derived(localArchives.filter(a => a.is_delimiter).length);
  let collapsedCount = $derived(Object.keys(collapsedFolders).filter(k => collapsedFolders[k]).length);
  let isAllCollapsed = $derived(totalFolders > 0 && collapsedCount === totalFolders);

  function toggleAllFolders() {
    if (isAllCollapsed) {
      collapsedFolders = {};
    } else {
      const newState: Record<string, boolean> = {};
      for (const a of localArchives) {
        if (a.is_delimiter) {
          newState[a.file_name] = true;
        }
      }
      collapsedFolders = newState;
    }
  }

  function toggleFolderCollapse(folderFileName: string) {
    collapsedFolders[folderFileName] = !collapsedFolders[folderFileName];
  }

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    archive: ArchiveItem | null;
    unassociatedXl: XlItem | null;
    targetType: 'archive' | 'xl' | 'unassociated_xl';
  }>({ visible: false, x: 0, y: 0, archive: null, unassociatedXl: null, targetType: 'archive' });

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
      destroy() { modNodeMap.delete(modName); }
    };
  }

  let showConflictSummary = $state(localStorage.getItem('cp2077_show_conflict_summary') !== 'false');
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

  $effect(() => { if (archives && archives.length >= 0) { localArchives = [...archives]; } });

  $effect(() => {
    if (targetHighlightMod) {
      activeViewMode = 'load_order';
      searchQuery = '';
      focusModInMainList(targetHighlightMod);
    }
  });

  $effect(() => {
    if (gamePath) {
      invoke<LabelsConfig>('load_labels_config', { gamePath }).then(res => { labelsConfig = res; }).catch(err => console.error(err));
      invoke<Record<string, string>>('load_notes_config').then(res => { notesConfig = res; }).catch(err => console.error(err));
    }
  });

  $effect(() => {
    if (!enableOrganizerView && activeViewMode !== 'load_order') {
      activeViewMode = 'load_order';
      searchQuery = '';
    }
  });

  // Tab Exit Effect: Clears Search, Selections, and Highlights
  $effect(() => {
    if (currentTab !== 'archive') {
      untrack(() => {
        if (searchQuery !== '') searchQuery = '';
        if (selectedMods.size > 0) {
          selectedMods.clear();
          selectedMods = new Set();
          lastSelectedFileName = null;
        }
        if (highlightedModName !== null) highlightedModName = null;
      });
    }
  });

  async function persistState() {
    const archiveNames = localArchives.map(a => a.file_name);
    try {
      const report = await invoke<ArchiveScanReport>('save_load_order', { gamePath, loadOrder: archiveNames });
      if (report && Array.isArray(report.archives)) { archives = report.archives; }
    } catch (err) { console.error(err); }
  }

  async function saveLabels() {
    try { await invoke('save_labels_config', { gamePath, config: labelsConfig }); } catch (err) { console.error(err); }
  }

  function handleCategoryModalConfirm(data: { id: string; name: string; color: string }) {
    labelsConfig.categories = [...labelsConfig.categories, { ...data, stashed: false }];
    for (const mod of selectedMods) { labelsConfig.assignments[mod] = data.id; }
    labelsConfig = { ...labelsConfig };
    saveLabels();
    isCategoryModalOpen = false;
  }

  function handleMoveCategoryConfirm(actionOrId: string) {
    if (actionOrId === '[CREATE_NEW]') {
      isMoveCategoryModalOpen = false;
      categoryModalMode = 'create';
      categoryModalData = { id: '', name: '', color: '#06B6D4' };
      isCategoryModalOpen = true;
      return;
    }
    if (actionOrId === '[REMOVE]') {
      for (const mod of selectedMods) { delete labelsConfig.assignments[mod]; }
      labelsConfig = { ...labelsConfig };
      saveLabels();
      isMoveCategoryModalOpen = false;
      return;
    }
    for (const mod of selectedMods) { labelsConfig.assignments[mod] = actionOrId; }
    labelsConfig = { ...labelsConfig };
    saveLabels();
    isMoveCategoryModalOpen = false;
  }

  async function handleNoteConfirm(note: string) {
    if (!targetNoteModName) return;
    const modName = targetNoteModName;
    if (note.trim() === '') { delete notesConfig[modName]; } else { notesConfig[modName] = note.trim(); }
    notesConfig = { ...notesConfig };
    try { await invoke('save_notes_config', { notes: notesConfig }); } catch (err) { console.error(err); }
  }

  function addFolder() {
    dialogState = {
      isOpen: true, title: 'Create Group',
      message: 'Name your new virtual group. Groups are an optional way to visually organize your load order. They appear as empty .txt files in your mod folder and have zero impact on game performance.',
      mode: 'prompt', initialValue: 'New Group', confirmText: 'Create Group', isDanger: false,
      onConfirm: async (name: string) => {
        dialogState.isOpen = false;
        if (!name || name.trim() === '') return;
        try {
          const report = await invoke<ArchiveScanReport>('create_physical_folder', { gamePath, folderName: name.trim() });
          if (report && Array.isArray(report.archives)) {
            archives = report.archives;
            if (onStateChanged) onStateChanged();
            const newFileName = `[DIR] ${name.trim()}.txt`;
            setTimeout(() => { focusModInMainList(newFileName); }, 100);
          }
        } catch (err) { console.error(err); }
      }
    };
  }

  function handleRenameFolder(archive: ArchiveItem) {
    const currentName = archive.category_name || archive.file_name.replace('[DIR] ', '').replace('.txt', '');
    dialogState = {
      isOpen: true, title: 'Rename Group',
      message: 'Enter a new name for this group delimiter:',
      mode: 'prompt', initialValue: currentName, confirmText: 'Rename', isDanger: false,
      onConfirm: async (newName: string) => {
        dialogState.isOpen = false;
        if (!newName || newName.trim() === '' || newName.trim() === currentName) return;
        try {
          const report = await invoke<ArchiveScanReport>('rename_physical_folder', { gamePath, oldFileName: archive.file_name, newFolderName: newName.trim() });
          if (report && Array.isArray(report.archives)) {
            const wasCollapsed = !!collapsedFolders[archive.file_name];
            delete collapsedFolders[archive.file_name];
            const newFileName = `[DIR] ${newName.trim()}.txt`;
            if (wasCollapsed) { collapsedFolders[newFileName] = true; }
            archives = report.archives;
            if (onStateChanged) onStateChanged();
          }
        } catch (err) { console.error(err); }
      }
    };
  }

  function handleDeleteFolder(archive: ArchiveItem) {
    dialogState = {
      isOpen: true, title: 'Delete Group',
      message: 'Are you sure you wish to delete this group? The marker will be removed, but all contained mods will remain preserved in their exact sequential order.',
      mode: 'confirm', initialValue: '', confirmText: 'Delete Group', isDanger: true,
      onConfirm: async () => {
        dialogState.isOpen = false;
        try {
          const report = await invoke<ArchiveScanReport>('delete_physical_folder', { gamePath, folderFileName: archive.file_name });
          if (report && Array.isArray(report.archives)) {
            delete collapsedFolders[archive.file_name];
            archives = report.archives;
            if (onStateChanged) onStateChanged();
          }
        } catch (err) { console.error(err); }
      }
    };
  }

  function confirmBulkAction(enable: boolean) {
    dialogState = {
      isOpen: true,
      title: enable ? 'Enable All Archive Mods' : 'Disable All Archive Mods',
      message: enable 
        ? 'Are you sure you want to enable all archive mods? This will activate every mod in this list.' 
        : 'Are you sure you want to disable all archive mods? This is a nuclear option that will deactivate every mod in this list.',
      mode: 'confirm',
      initialValue: '',
      confirmText: enable ? 'Enable All' : 'Disable All',
      isDanger: !enable,
      onConfirm: async () => {
        dialogState.isOpen = false;
        try {
          await invoke('bulk_toggle_mod_type', { gamePath, modType: 'archive', enable });
          if (onScanRequested) onScanRequested();
        } catch (err) {
          console.error(err);
        }
      }
    };
  }

  let toggleDebounceTimer: number | null = null;

  async function toggleMod(archive: ArchiveItem) {
    const next = !archive.enabled;
    archive.enabled = next;
    if (archive.is_delimiter) {
      const startIndex = localArchives.findIndex(a => a.file_name === archive.file_name);
      if (startIndex !== -1) {
        for (let i = startIndex + 1; i < localArchives.length; i++) {
          const child = localArchives[i];
          if (child.is_delimiter) break;
          child.enabled = next;
        }
      }
    }
    (async () => {
      try {
        await invoke('toggle_mod_state', { gamePath, modName: archive.file_name, enable: next });
        if (archive.is_delimiter) {
          const startIndex = localArchives.findIndex(a => a.file_name === archive.file_name);
          if (startIndex !== -1) {
            for (let i = startIndex + 1; i < localArchives.length; i++) {
              const child = localArchives[i];
              if (child.is_delimiter) break;
              await invoke('toggle_mod_state', { gamePath, modName: child.file_name, enable: next });
            }
          }
        }
        if (toggleDebounceTimer) clearTimeout(toggleDebounceTimer);
        toggleDebounceTimer = window.setTimeout(() => {
          persistState();
          if (onStateChanged) onStateChanged();
        }, 600);
      } catch (err) {
        console.error(err);
        archive.enabled = !next;
      }
    })();
  }

  async function focusModInMainList(modName: string) {
    if (searchQuery !== '') { searchQuery = ''; }
    const targetIndex = localArchives.findIndex(a => a.file_name === modName);
    if (targetIndex !== -1) {
      for (let i = targetIndex - 1; i >= 0; i--) {
        const item = localArchives[i];
        if (item.is_delimiter) {
          if (collapsedFolders[item.file_name]) { collapsedFolders[item.file_name] = false; }
          break;
        }
      }
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
      highlightTimeoutId = window.setTimeout(() => { highlightedModName = null; }, 2500);
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
      preDragCollapseState = { ...collapsedFolders };
      if (size > 1) {
        collapsedFolders = { ...collapsedFolders, [localArchives[index].file_name]: true };
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
      if (!animationFrameId) { animationFrameId = requestAnimationFrame(processAutoScroll); }
    } else if (event.clientY > bottomEdge) {
      const distance = event.clientY - bottomEdge;
      scrollSpeed = Math.min(18, Math.max(3, distance / 3));
      if (!animationFrameId) { animationFrameId = requestAnimationFrame(processAutoScroll); }
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
    if (animationFrameId) { cancelAnimationFrame(animationFrameId); animationFrameId = null; }
    if (activeDragIndex !== null && dropTargetIndex !== null && dropPlacement !== null) {
      let targetIndex = dropTargetIndex;
      if (dropPlacement === 'after') {
        const targetItem = localArchives[dropTargetIndex];
        if (targetItem.is_delimiter && collapsedFolders[targetItem.file_name]) {
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
        if (targetIndex > activeDragIndex + dragBlockSize) { targetIndex -= dragBlockSize; } else { targetIndex = activeDragIndex; }
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
      collapsedFolders = { ...preDragCollapseState };
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
        for (let i = min; i <= max; i++) { selectedMods.add(visibleMods[i].archive.file_name); }
      }
    } else if (isCtrl) {
      if (selectedMods.has(archive.file_name)) { selectedMods.delete(archive.file_name); } else { selectedMods.add(archive.file_name); }
      lastSelectedFileName = archive.file_name;
    } else {
      selectedMods.clear();
      selectedMods.add(archive.file_name);
      lastSelectedFileName = archive.file_name;
    }
    selectedMods = new Set(selectedMods);
  }

  function handleMoveToFolder(targetFolderFileName: string) {
    isMoveModalOpen = false;
    if (targetFolderFileName.startsWith('[CREATE_NEW]')) {
      const prefill = targetFolderFileName.split(':')[1] || 'New Group';
      dialogState = {
        isOpen: true, title: 'Create & Move to Group',
        message: 'Name your new virtual group.',
        mode: 'prompt', initialValue: prefill, confirmText: 'Create & Move', isDanger: false,
        onConfirm: async (name: string) => {
          dialogState.isOpen = false;
          if (!name || name.trim() === '') return;
          try {
            const report = await invoke<ArchiveScanReport>('create_physical_folder', { gamePath, folderName: name.trim() });
            if (report && Array.isArray(report.archives)) {
              archives = report.archives;
              localArchives = [...archives];
              const newFileName = `[DIR] ${name.trim()}.txt`;
              executeMoveToFolder(newFileName);
              if (onStateChanged) onStateChanged();
              setTimeout(() => { focusModInMainList(newFileName); }, 100);
            }
          } catch (err) { console.error(err); }
        }
      };
      return;
    }
    executeMoveToFolder(targetFolderFileName);
  }

  function executeMoveToFolder(targetFolderFileName: string) {
    if (selectedMods.size === 0) return;
    const updated = [...localArchives];
    const extracted: ArchiveItem[] = [];
    for (let i = updated.length - 1; i >= 0; i--) {
      if (selectedMods.has(updated[i].file_name)) { extracted.unshift(updated.splice(i, 1)[0]); }
    }
    let insertIndex = 0;
    if (targetFolderFileName !== '[TOP]') {
      const catIndex = updated.findIndex(a => a.file_name === targetFolderFileName);
      if (catIndex !== -1) {
        insertIndex = catIndex + 1;
        while (insertIndex < updated.length && !updated[insertIndex].is_delimiter) { insertIndex++; }
      } else {
        insertIndex = updated.length;
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
      const report = await invoke<ArchiveScanReport>('link_xl_to_archive', { gamePath, xlName: targetXlToLink, archiveName: archiveFileName });
      archives = report.archives;
      scanReport = report;
      if (onStateChanged) onStateChanged();
    } catch (err) { console.error(err); }
    targetXlToLink = null;
  }

  async function handleUnlinkXl() {
    if (!contextMenu.archive || !contextMenu.archive.associated_xls) return;
    try {
      let report: ArchiveScanReport | null = null;
      for (const xl of contextMenu.archive.associated_xls) {
        report = await invoke<ArchiveScanReport>('unlink_xl_from_archive', { gamePath, xlName: xl.file_name });
      }
      if (report) {
        archives = report.archives;
        scanReport = report;
        if (onStateChanged) onStateChanged();
      }
    } catch (err) { console.error(err); }
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
    let menuHeight = 160;
    if (targetType === 'unassociated_xl') menuHeight = 180;
    else if (targetType === 'xl') menuHeight = 180;
    else if (targetType === 'archive') {
      const archive = target as ArchiveItem;
      menuHeight = archive.is_delimiter ? 190 : (selectedMods.size > 0 ? 320 : 220);
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

  async function handleContextMenuToggle() {
    if (!contextMenu.archive) { closeContextMenu(); return; }
    const targetState = !contextMenu.archive.enabled;
    if (!contextMenu.archive.is_delimiter && selectedMods.size > 1 && selectedMods.has(contextMenu.archive.file_name)) {
      const itemsToToggle = localArchives.filter(a => selectedMods.has(a.file_name) && a.enabled !== targetState);
      for (const item of itemsToToggle) { item.enabled = targetState; }
      (async () => {
        for (const item of itemsToToggle) {
          try { await invoke('toggle_mod_state', { gamePath, modName: item.file_name, enable: targetState }); } catch (err) { console.error(err); }
        }
        if (toggleDebounceTimer) clearTimeout(toggleDebounceTimer);
        toggleDebounceTimer = window.setTimeout(() => { persistState(); if (onStateChanged) onStateChanged(); }, 600);
      })();
    } else {
      toggleMod(contextMenu.archive);
    }
    closeContextMenu();
  }

  async function handleContextMenuShowInExplorer() {
    if (!gamePath) return;
    let targetFilename = '';
    if (contextMenu.targetType === 'unassociated_xl' && contextMenu.unassociatedXl) { targetFilename = contextMenu.unassociatedXl?.file_name; }
    else if (contextMenu.archive) {
      targetFilename = contextMenu.archive?.file_name;
      if (contextMenu.targetType === 'xl' && contextMenu.archive?.associated_xls?.length) { targetFilename = contextMenu.archive.associated_xls[0].file_name; }
    }
    if (!targetFilename) return;
    const fullPath = `${gamePath}\\archive\\pc\\mod\\${targetFilename}`;
    try { await revealItemInDir(fullPath); } catch (err) { console.error(err); }
    closeContextMenu();
  }

  async function handleContextMenuCopyName() {
    let targetFilename = '';
    if (contextMenu.targetType === 'unassociated_xl' && contextMenu.unassociatedXl) { targetFilename = contextMenu.unassociatedXl?.file_name; }
    else if (contextMenu.archive) {
      targetFilename = contextMenu.archive?.file_name;
      if (contextMenu.targetType === 'xl' && contextMenu.archive?.associated_xls?.length) { targetFilename = contextMenu.archive.associated_xls.map(x => x.file_name).join(', '); }
    }
    if (!targetFilename) return;
    try {
      await navigator.clipboard.writeText(targetFilename);
      copiedFeedback = true;
      setTimeout(() => { closeContextMenu(); }, 400);
    } catch (err) { console.error(err); closeContextMenu(); }
  }

  let availableFolders = $derived.by(() => {
    return localArchives.filter(a => a.is_delimiter).map(a => ({
      name: a.category_name || a.file_name.replace('[DIR] ', '').replace('.txt', ''),
      file_name: a.file_name
    }));
  });

  let availableArchivesForLink = $derived.by(() => {
    return localArchives.filter(a => !a.is_delimiter).map(a => ({ name: a.name, file_name: a.file_name }));
  });

  let folderCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    let currentCat: string | null = null;
    let currentCount = 0;
    for (const item of localArchives) {
      if (item.is_delimiter) {
        if (currentCat) { counts.set(currentCat, currentCount); }
        currentCat = item.file_name;
        currentCount = 0;
      } else if (currentCat) {
        currentCount++;
      }
    }
    if (currentCat) { counts.set(currentCat, currentCount); }
    return counts;
  });

  let previewConflictState = $derived.by(() => {
    if (activeDragIndex === null || dropTargetIndex === null || dropPlacement === null) { return null; }
    let targetIndex = dropTargetIndex;
    if (dropPlacement === 'after') {
      const targetItem = localArchives[dropTargetIndex];
      if (targetItem.is_delimiter && collapsedFolders[targetItem.file_name]) {
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
      if (targetIndex > activeDragIndex + dragBlockSize) { targetIndex -= dragBlockSize; } else { targetIndex = activeDragIndex; }
    }
    if (activeDragIndex === targetIndex) { return null; }
    const updated = [...localArchives];
    const movedItems = updated.splice(activeDragIndex, dragBlockSize);
    updated.splice(targetIndex, 0, ...movedItems);
    const indexMap = new Map<string, number>();
    for (let i = 0; i < updated.length; i++) { indexMap.set(updated[i].file_name, i); }
    const stateMap = new Map<string, { wins: string[], loses: string[] }>();
    for (let i = 0; i < updated.length; i++) {
      const item = updated[i];
      if (item.has_conflicts) {
        const newWins: string[] = [];
        const newLoses: string[] = [];
        for (const rival of item.conflicts_with) {
          const rivalIndex = indexMap.get(rival);
          if (rivalIndex !== undefined) {
            if (i < rivalIndex) { newWins.push(rival); } else { newLoses.push(rival); }
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
        currentCategoryCollapsed = !!collapsedFolders[archive.file_name];
        inCategory = true;
        if (searchQuery === '' || archive.name.toLowerCase().includes(searchQuery.toLowerCase())) {
          result.push({ archive, originalIndex: i, archiveRank: 0, inCategory: false });
        }
      } else {
        rank += 1;
        if (!currentCategoryCollapsed || searchQuery !== '') {
          if (searchQuery === '' || archive.name.toLowerCase().includes(searchQuery.toLowerCase())) {
            result.push({ archive, originalIndex: i, archiveRank: rank, inCategory });
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

  let draggedEntry = $derived(activeDragIndex !== null ? localArchives[activeDragIndex] : null);
  let unassociatedXlFiles = $derived((scanReport?.unassociated_xl ?? []).filter(item => {
    if (!searchQuery.trim()) return true;
    return item.file_name.toLowerCase().includes(searchQuery.toLowerCase());
  }));
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={completeDrop}
  onclick={closeContextMenu}
/>

{#if activeDragIndex !== null && draggedEntry}
  <div
    class="fixed pointer-events-none z-50 px-3 py-1.5 rounded-lg border shadow-2xl shadow-black/60 flex items-center gap-2.5 backdrop-blur-md -translate-x-4 -translate-y-6"
    style="left: {cursorX}px; top: {cursorY}px; background-color: var(--theme-card); border-color: var(--theme-accent);"
  >
    <GripVertical class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
    {#if draggedEntry.is_delimiter}
      <Folder class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
      <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">
        {draggedEntry.category_name || draggedEntry.file_name.replace('[DIR] ', '').replace('.txt', '')}
        {#if dragBlockSize > 1}
          <span class="ml-1 font-mono" style="color: var(--theme-accent);">({dragBlockSize - 1})</span>
        {/if}
      </span>
    {:else}
      <span class="text-xs font-mono font-semibold" style="color: var(--theme-text-primary);">{draggedEntry.file_name}</span>
    {/if}
  </div>
{/if}

<!-- Custom Context Menu -->
{#if contextMenu.visible && (contextMenu.archive || contextMenu.unassociatedXl)}
  {@const activeTargetName = contextMenu.targetType === 'unassociated_xl' 
    ? contextMenu.unassociatedXl?.file_name 
    : (contextMenu.targetType === 'xl' && contextMenu.archive?.associated_xls?.length)
      ? (contextMenu.archive.associated_xls.length === 1 ? contextMenu.archive.associated_xls[0].file_name : contextMenu.archive.associated_xls.length + ' Companion Files')
      : contextMenu.archive?.file_name}
  {@const isActive = contextMenu.targetType === 'unassociated_xl' ? contextMenu.unassociatedXl?.enabled : contextMenu.archive?.enabled}
  
  <div
    class="fixed z-50 w-56 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center justify-between gap-2" style="border-color: var(--theme-border); background-color: var(--theme-surface);">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.targetType === 'xl' || contextMenu.targetType === 'unassociated_xl'}
          <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--color-info);" />
        {:else if contextMenu.archive?.is_delimiter}
          <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-accent);" />
        {:else}
          <div class="h-2 w-2 rounded-full shrink-0" style="background-color: {isActive ? 'var(--theme-accent)' : 'var(--color-danger)'};"></div>
        {/if}
        <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);" title={activeTargetName}>
          {activeTargetName}
        </span>
      </div>
    </div>

    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleContextMenuShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <FolderSearch class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in Explorer</span>
      </button>

      <button
        type="button"
        onclick={() => {
          const name = contextMenu.targetType === 'unassociated_xl' ? contextMenu.unassociatedXl?.file_name : contextMenu.archive?.file_name;
          if (name && onRouteToHistory) onRouteToHistory(name);
          closeContextMenu();
        }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Clock class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in History</span>
      </button>

      {#if contextMenu.targetType === 'unassociated_xl'}
        <button
          type="button"
          onclick={() => { if(contextMenu.unassociatedXl) { targetXlToLink = contextMenu.unassociatedXl.file_name; isLinkModalOpen = true; closeContextMenu(); } }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Link2 class="h-3.5 w-3.5" style="color: var(--color-info);" />
          <span>Link to Archive...</span>
        </button>
      {:else}
        <button
          type="button"
          onclick={handleContextMenuToggle}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Power class="h-3.5 w-3.5" style="color: {isActive ? 'var(--color-warning)' : 'var(--theme-accent)'};" />
          {#if contextMenu.targetType === 'archive' && !contextMenu.archive?.is_delimiter && selectedMods.size > 1 && contextMenu.archive?.file_name && selectedMods.has(contextMenu.archive.file_name)}
            <span>{isActive ? 'Disable ' + selectedMods.size + ' Mods' : 'Enable ' + selectedMods.size + ' Mods'}</span>
          {:else}
            <span>{isActive ? (contextMenu.archive?.is_delimiter ? 'Disable Group' : 'Disable Mod') : (contextMenu.archive?.is_delimiter ? 'Enable Group' : 'Enable Mod')}</span>
          {/if}
        </button>
      {/if}

      {#if contextMenu.targetType === 'archive' && !contextMenu.archive?.is_delimiter}
        <button
          type="button"
          onclick={() => { targetNoteModName = contextMenu.archive?.file_name || ''; closeContextMenu(); isNoteModalOpen = true; }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
          <span>{notesConfig[contextMenu.archive?.file_name || ''] ? 'Edit Note...' : 'Add Note...'}</span>
        </button>
      {/if}

      {#if enableOrganizerView && contextMenu.targetType === 'archive' && !contextMenu.archive?.is_delimiter && selectedMods.size > 0}
        {@const hasExistingLabel = Array.from(selectedMods).some(m => labelsConfig.assignments[m])}
        <button
          type="button"
          onclick={() => { closeContextMenu(); isMoveCategoryModalOpen = true; }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Tag class="h-3.5 w-3.5" style="color: var(--color-info);" />
          <span>{hasExistingLabel ? 'Edit Category Label...' : 'Add Category Label...'}</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'archive' && contextMenu.archive?.is_delimiter}
        <button
          type="button"
          onclick={() => { const item = contextMenu.archive; closeContextMenu(); if (item) handleRenameFolder(item); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Pencil class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Rename Group</span>
        </button>

        <button
          type="button"
          onclick={() => { const item = contextMenu.archive; closeContextMenu(); if (item) handleDeleteFolder(item); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--color-danger);"
        >
          <Trash2 class="h-3.5 w-3.5" style="color: var(--color-danger);" />
          <span>Delete Group</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'archive' && !contextMenu.archive?.is_delimiter && selectedMods.size > 0}
        <button
          type="button"
          onclick={() => { closeContextMenu(); isMoveModalOpen = true; }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <FolderInput class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Move {selectedMods.size} Mods to Group...</span>
        </button>

        <button
          type="button"
          onclick={() => { closeContextMenu(); handleMoveToFolder('[TOP]'); }}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <ArrowUp class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Move to Top (Ungrouped)</span>
        </button>
      {/if}

      {#if contextMenu.targetType === 'xl'}
        <button
          type="button"
          onclick={handleUnlinkXl}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Unlink class="h-3.5 w-3.5" style="color: var(--color-warning);" />
          <span>Unlink .xl</span>
        </button>
      {/if}

      <button
        type="button"
        onclick={handleContextMenuCopyName}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <div class="flex items-center gap-2.5">
          {#if copiedFeedback}
            <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
            <span class="font-medium" style="color: var(--theme-accent);">Copied!</span>
          {:else}
            <Copy class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
            <span>Copy File Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="flex flex-col h-full select-none" oncontextmenu={(e) => e.preventDefault()}>
  
  <!-- Toolbar Header -->
  <div class="flex items-center justify-between gap-4 shrink-0 mb-3">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="h-4 w-4 absolute left-3 top-1/2 -translate-y-1/2" style="color: var(--theme-text-muted);" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter mods or groups..."
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

    <div class="flex items-center gap-2.5">
      {#if enableOrganizerView}
        <div class="flex items-center h-8 p-0.5 rounded border" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
          <button
            type="button"
            onclick={() => { activeViewMode = 'load_order'; searchQuery = ''; }}
            class="h-full px-2.5 rounded-sm flex items-center justify-center transition cursor-pointer"
            style="{activeViewMode === 'load_order' ? 'background-color: var(--theme-card); color: var(--theme-accent);' : 'color: var(--theme-text-muted);'}"
            title="Load Order View"
          >
            <ListTree class="h-4 w-4" />
          </button>
          <button
            type="button"
            onclick={() => { activeViewMode = 'organizer'; searchQuery = ''; }}
            class="h-full px-2.5 rounded-sm flex items-center justify-center transition cursor-pointer"
            style="{activeViewMode === 'organizer' ? 'background-color: var(--theme-card); color: var(--color-info);' : 'color: var(--theme-text-muted);'}"
            title="Organizer View"
          >
            <LayoutGrid class="h-4 w-4" />
          </button>
        </div>
      {/if}

      {#if activeViewMode === 'load_order'}
        <button
          onclick={toggleSummaryDrawer}
          class="h-8 px-2.5 rounded border transition cursor-pointer flex items-center justify-center hover:brightness-125"
          style="{showConflictSummary ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: var(--theme-accent); color: var(--theme-accent);' : 'background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);'}"
          title="Toggle Conflict Summary"
        >
          <AlertTriangle class="h-4 w-4" />
        </button>

        <button
          onclick={toggleAllFolders}
          disabled={totalFolders === 0}
          class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-125"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        >
          {#if isAllCollapsed}
            <FolderOpen class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
            <span>Expand All</span>
          {:else}
            <Folder class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
            <span>Collapse All</span>
          {/if}
        </button>

        <button
          onclick={addFolder}
          class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        >
          <FolderPlus class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Add Group</span>
        </button>

        <div class="w-px h-5 mx-1" style="background-color: var(--theme-border);"></div>

        <button
          onclick={() => confirmBulkAction(true)}
          class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
          title="Enable All Archive Mods"
        >
          <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span class="hidden xl:inline">Enable All</span>
        </button>

        <button
          onclick={() => confirmBulkAction(false)}
          class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--color-danger);"
          title="Disable All Archive Mods"
        >
          <Power class="h-3.5 w-3.5" style="color: var(--color-danger);" />
          <span class="hidden xl:inline">Disable All</span>
        </button>
      {:else}
        <button
          onclick={() => showUncategorizedDrawer = !showUncategorizedDrawer}
          class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
          style="{showUncategorizedDrawer ? 'background-color: color-mix(in srgb, var(--color-warning) 15%, transparent); border-color: var(--color-warning); color: var(--color-warning);' : 'background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);'}"
          title="Toggle The Stash Drawer"
        >
          {#if showUncategorizedDrawer}
            <Eye class="h-3.5 w-3.5" />
          {:else}
            <EyeOff class="h-3.5 w-3.5" />
          {/if}
          <span>The Stash</span>
        </button>
      {/if}
    </div>
  </div>

  <div class="flex gap-3 flex-1 overflow-hidden min-h-0">
    
    {#if activeViewMode === 'organizer'}
      <div class="flex-1 overflow-hidden pt-2">
        <OrganizerView 
          bind:labelsConfig 
          bind:notesConfig
          {archives} 
          {gamePath} 
          bind:showCategoryLabelsInLoadOrder
          onLabelsToggle={onLabelsToggle}
          {showUncategorizedDrawer}
          {searchQuery}
          onShowInLoadOrder={(modName) => {
            activeViewMode = 'load_order';
            searchQuery = '';
            setTimeout(() => focusModInMainList(modName), 100);
          }}
          {onRouteToHistory}
        />
      </div>
    {:else}
      <div
        bind:this={scrollContainer}
        class="space-y-0 overflow-y-auto pr-1 flex-1 max-h-[calc(100vh-120px)]"
      >
        {#if visibleItems.length === 0}
          <div class="p-6 text-center text-xs border rounded" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);">
            No matches found.
          </div>
        {:else}
          {#each visibleItems as { archive, originalIndex, archiveRank, inCategory } (archive.file_name + '_' + archive.enabled)}
            {@const isSource = activeDragIndex !== null && originalIndex >= activeDragIndex && originalIndex < activeDragIndex + dragBlockSize}
            {@const showLineBefore = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'before' && !isSource}
            {@const showLineAfter = activeDragIndex !== null && dropTargetIndex === originalIndex && dropPlacement === 'after' && !isSource}

            <div class="relative flex flex-col">
              {#if showLineBefore}
                <div class="absolute {originalIndex === 0 ? '-top-1.5' : '-top-1'} left-0 right-0 h-[2.5px] z-30 flex items-center shadow-[0_0_12px_var(--theme-accent)]" style="background-color: var(--theme-accent);">
                  <div class="h-2.5 w-2.5 rounded-full -ml-1.5 shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
                </div>
              {/if}

              {#if archive.is_delimiter}
                <FolderDelimiterRow
                  {archive}
                  {originalIndex}
                  {isSource}
                  collapsed={!!collapsedFolders[archive.file_name]}
                  modCount={folderCounts.get(archive.file_name) ?? 0}
                  onToggleCollapse={toggleFolderCollapse}
                  isHighlighted={highlightedModName === archive.file_name}
                  onDragStart={startDrag}
                  onPointerMove={onRowPointerMove}
                  onToggle={toggleMod}
                  onContextMenu={openContextMenu}
                  onRenameFolder={handleRenameFolder}
                  onDeleteFolder={handleDeleteFolder}
                  registerNode={registerModNode}
                />
              {:else}
                {@const isExpanded = !!expandedRows[archive.file_name]}
                {@const isHighlighted = highlightedModName === archive.file_name}
                {@const isSelected = selectedMods.has(archive.file_name)}
                {@const activeConflicts = previewConflictState?.get(archive.file_name) || { wins: archive.wins, loses: archive.loses }}
                {@const catId = labelsConfig.assignments[archive.file_name]}
                {@const assignedCat = catId ? labelsConfig.categories.find(c => c.id === catId) : null}

                <div
                  role="group"
                  aria-label="Archive mod item: {archive.file_name}"
                  use:registerModNode={archive.file_name}
                  onclick={(e) => handleModClick(e, archive)}
                  onpointermove={(e) => onRowPointerMove(e, originalIndex)}
                  onpointerenter={() => globalSelectedPath = archive.path}
                  onpointerleave={() => globalSelectedPath = ''}
                  oncontextmenu={(e) => openContextMenu(e, archive, 'archive')}
                  class="group transition-all duration-200 z-10 border-b cursor-context-menu
                    {isHighlighted ? 'ring-1 bg-[color-mix(in_srgb,var(--theme-accent)_15%,transparent)] border-[var(--theme-accent)] shadow-[0_0_15px_color-mix(in_srgb,var(--theme-accent)_35%,transparent)]' : 
                    (isSelected ? 'bg-[color-mix(in_srgb,var(--theme-accent)_10%,transparent)] border-[color-mix(in_srgb,var(--theme-accent)_60%,transparent)]' : 
                    'bg-transparent border-[var(--theme-border)] hover:bg-[color-mix(in_srgb,var(--theme-accent)_12%,transparent)]')}
                    {archive.enabled ? 'opacity-100' : 'opacity-40'} 
                    {isSource ? 'opacity-20 border-dashed' : ''}"
                >
                  <div class="flex items-center justify-between px-2 gap-2 density-row relative">
                    
                    <div class="flex items-center gap-2.5 min-w-0 flex-1 pl-1">
                      <div
                        role="button"
                        tabindex="0"
                        aria-label="Drag to adjust load order"
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                        onpointerdown={(e) => { e.stopPropagation(); startDrag(e, originalIndex); }}
                        class="cursor-grab active:cursor-grabbing p-1 rounded transition shrink-0 hover:bg-[var(--theme-surface)]"
                        style="color: var(--theme-text-muted);"
                        title="Drag to adjust load order"
                      >
                        <GripVertical class="h-3.5 w-3.5" />
                      </div>

                      <button
                        type="button"
                        onclick={(e) => { e.stopPropagation(); toggleMod(archive); }}
                        aria-label={archive.enabled ? "Disable mod " + archive.file_name : "Enable mod " + archive.file_name}
                        class="w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors shrink-0 cursor-pointer"
                        style="border-color: {archive.enabled ? 'var(--theme-accent)' : 'var(--theme-border)'}; background-color: {archive.enabled ? 'var(--theme-accent)' : 'transparent'};"
                      >
                        {#if archive.enabled}
                          <Check class="h-2.5 w-2.5" style="color: var(--theme-accent-contrast);" strokeWidth={3} />
                        {/if}
                      </button>

                      {#if inCategory}
                        <div class="flex items-center shrink-0 w-4 h-full relative">
                          <div class="absolute left-1 top-0 bottom-0 w-[1px] pointer-events-none" style="background-color: color-mix(in srgb, var(--theme-border) 70%, transparent);"></div>
                          <div class="absolute left-1 top-1/2 w-2.5 h-[1px] pointer-events-none" style="background-color: color-mix(in srgb, var(--theme-border) 70%, transparent);"></div>
                        </div>
                      {/if}

                      <span class="text-[10px] font-mono px-1 py-0.2 min-w-[34px] text-center rounded border shrink-0" style="background-color: var(--theme-card); color: var(--theme-text-muted); border-color: var(--theme-border);">
                        #{archiveRank}
                      </span>

                      <span class="font-mono font-medium truncate max-w-md" style="color: var(--theme-text-primary);">
                        {archive.file_name}
                      </span>

                      {#if archive.associated_xls && archive.associated_xls.length > 0}
                        <span
                          role="button"
                          tabindex="0"
                          oncontextmenu={(e) => { e.stopPropagation(); openContextMenu(e, archive, 'xl'); }}
                          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
                          class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded border transition-colors shrink-0 cursor-context-menu"
                          style="{archive.enabled ? 'background-color: color-mix(in srgb, var(--color-info) 15%, transparent); color: var(--color-info); border-color: color-mix(in srgb, var(--color-info) 40%, transparent);' : 'background-color: var(--theme-card); color: var(--theme-text-muted); border-color: var(--theme-border); opacity: 0.5;'}"
                          title="Companions: {archive.associated_xls.map(x => x.file_name).join(', ')} - Right-click for options"
                        >
                          XL {#if archive.associated_xls.length > 1}({archive.associated_xls.length}){/if}
                        </span>
                      {/if}

                      <div class="flex items-center gap-1.5 text-[10px] shrink-0 opacity-80 hidden lg:flex" style="color: var(--theme-text-muted);">
                        <span>({archive.file_count})</span>
                        <span>•</span>
                        <span>{formatBytes(archive.size_bytes)}</span>
                      </div>
                    </div>

                    <div class="flex items-center gap-2 shrink-0">
                      
                      <div class="w-28 flex justify-end pr-2">
                        {#if showCategoryLabelsInLoadOrder && assignedCat && !assignedCat.stashed}
                          <span 
                            class="text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.2 rounded border shrink-0 truncate max-w-full"
                            style="background-color: color-mix(in srgb, {assignedCat.color} 15%, transparent); color: {assignedCat.color}; border-color: color-mix(in srgb, {assignedCat.color} 40%, transparent);"
                            title="Assigned to {assignedCat.name}"
                          >
                            {assignedCat.name}
                          </span>
                        {/if}
                      </div>

                      <div class="w-4 flex items-center justify-center shrink-0">
                        {#if notesConfig[archive.file_name]}
                          <button 
                            type="button"
                            onclick={(e) => { e.stopPropagation(); targetNoteModName = archive.file_name; isNoteModalOpen = true; }}
                            class="hover:scale-110 transition-transform cursor-pointer"
                            title={notesConfig[archive.file_name]}
                          >
                            <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
                          </button>
                        {/if}
                      </div>

                      <div class="w-3 flex items-center justify-center shrink-0">
                        {#if !archive.enabled}
                          <div class="h-2 w-2 rounded-full opacity-50" style="background-color: var(--theme-text-muted);" title="Disabled"></div>
                        {:else if activeConflicts.loses.length > 0}
                          <div class="h-2 w-2 rounded-full shadow-[0_0_4px_var(--color-danger)]" style="background-color: var(--color-danger);" title="Overwritten by higher mod"></div>
                        {:else if activeConflicts.wins.length > 0}
                          <div class="h-2 w-2 rounded-full shadow-[0_0_4px_var(--color-success)]" style="background-color: var(--color-success);" title="Winning overwrites"></div>
                        {:else}
                          <div class="h-2 w-2 rounded-full opacity-50" style="background-color: var(--color-success);" title="Clean (No conflicts)"></div>
                        {/if}
                      </div>

                      <div class="flex items-center min-w-0 w-6">
                        {#if archive.has_conflicts}
                          <button
                            type="button"
                            onclick={(e) => { e.stopPropagation(); expandedRows[archive.file_name] = !expandedRows[archive.file_name]; }}
                            class="p-0.5 rounded transition cursor-pointer hover:bg-[var(--theme-card)]"
                            style="color: var(--theme-text-muted);"
                            title="Toggle conflict details"
                          >
                            {#if isExpanded}
                              <ChevronDown class="h-3.5 w-3.5" />
                            {:else}
                              <ChevronRight class="h-3.5 w-3.5" />
                            {/if}
                          </button>
                        {/if}
                      </div>

                      <div class="flex items-center justify-end shrink-0 w-6">
                        <button
                          type="button"
                          onclick={(e) => { e.stopPropagation(); openContextMenu(e, archive, 'archive'); }}
                          class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-surface)]"
                          style="color: var(--theme-text-muted);"
                          title="More options"
                        >
                          <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
                        </button>
                      </div>
                    </div>
                  </div>

                  {#if isExpanded && archive.has_conflicts}
                    <div class="px-4 py-2 border-t space-y-2 text-xs" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-card) 30%, transparent);">
                      {#if activeConflicts.wins.length > 0}
                        <div>
                          <span class="text-[10px] font-semibold uppercase tracking-wider" style="color: var(--color-success);">Overwrites Lower Mods:</span>
                          <div class="mt-1 flex flex-wrap gap-1">
                            {#each activeConflicts.wins as target}
                              <span class="px-1.5 py-0.2 rounded border text-[10px] font-mono" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);">
                                {target}
                              </span>
                            {/each}
                          </div>
                        </div>
                      {/if}

                      {#if activeConflicts.loses.length > 0}
                        <div>
                          <span class="text-[10px] font-semibold uppercase tracking-wider" style="color: var(--color-danger);">Loses To Higher Mods:</span>
                          <div class="mt-1 flex flex-wrap gap-1">
                            {#each activeConflicts.loses as target}
                              <span class="px-1.5 py-0.2 rounded border text-[10px] font-mono font-semibold" style="background-color: color-mix(in srgb, var(--color-danger) 15%, transparent); border-color: color-mix(in srgb, var(--color-danger) 40%, transparent); color: var(--color-danger);">
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
                <div class="absolute -bottom-1 left-0 right-0 h-[2.5px] z-30 flex items-center shadow-[0_0_12px_var(--theme-accent)]" style="background-color: var(--theme-accent);">
                  <div class="h-2.5 w-2.5 rounded-full -ml-1.5 shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
                </div>
              {/if}
            </div>
          {/each}
        {/if}

        <!-- Unassociated .xl Files Section -->
        {#if unassociatedXlFiles.length > 0}
          <div class="mt-6 pt-4 border-t space-y-2" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent);">
            <div 
              role="button"
              tabindex="0"
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showUnassociatedXl = !showUnassociatedXl; }}
              onclick={() => showUnassociatedXl = !showUnassociatedXl}
              class="rounded-lg border px-3 py-2 flex items-center justify-between cursor-pointer transition-colors hover:brightness-125"
              style="background-color: color-mix(in srgb, var(--color-info) 10%, transparent); border-color: color-mix(in srgb, var(--color-info) 30%, transparent);"
            >
              <div class="flex items-center gap-2 min-w-0">
                {#if showUnassociatedXl}
                  <ChevronDown class="h-4 w-4 shrink-0" style="color: var(--color-info);" />
                {:else}
                  <ChevronRight class="h-4 w-4 shrink-0" style="color: var(--color-info);" />
                {/if}
                <FileCode class="h-4 w-4 shrink-0" style="color: var(--color-info);" />
                <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--color-info);">
                  Unassociated .xl Files
                </span>
                <span class="text-[10px] hidden sm:inline" style="color: var(--theme-text-muted);">
                  (Active in game, without a systemically generated mod association)
                </span>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); showXlHelp = !showXlHelp; }}
                  class="flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium border transition-colors hover:brightness-125"
                  style="{showXlHelp ? 'background-color: color-mix(in srgb, var(--color-info) 20%, transparent); color: var(--color-info); border-color: var(--color-info);' : 'background-color: var(--theme-surface); color: var(--color-info); border-color: color-mix(in srgb, var(--color-info) 40%, transparent);'}"
                  title="Click for association instructions"
                >
                  <Info class="h-3.5 w-3.5" />
                  <span>Info</span>
                </button>

                <span class="text-[10px] font-mono px-2 py-0.5 rounded-full border" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--color-info);">
                  {unassociatedXlFiles.length} loose
                </span>
              </div>
            </div>

            {#if showXlHelp}
              <div class="p-3 rounded-lg border text-xs space-y-1.5" style="background-color: color-mix(in srgb, var(--color-info) 5%, transparent); border-color: color-mix(in srgb, var(--color-info) 30%, transparent); color: var(--theme-text-primary);">
                <p class="text-[11px] pl-6 leading-relaxed" style="color: var(--theme-text-muted);">
                  <span class="font-medium" style="color: var(--theme-text-primary);">Not required for the game to function. An organizational quality-of-life feature only. ArchiveXL will load untethered .xl files regardless.</span>
                  <br /><br />
                  <span class="font-semibold" style="color: var(--color-info);">What is Linking?</span><br />
                  Linking bonds a loose <span class="font-mono" style="color: var(--theme-text-primary);">.xl</span> file to a parent <span class="font-mono" style="color: var(--theme-text-primary);">.archive</span> mod. Once linked, the parent mod will display a blue <span class="font-mono font-bold" style="color: var(--color-info);">[XL]</span> badge, and toggling the parent mod will automatically toggle the companion file.
                  <br /><br />
                  <span class="font-semibold" style="color: var(--color-info);">How to Link / Unlink:</span><br />
                  • <span class="font-semibold" style="color: var(--theme-text-primary);">Link:</span> Click the "Active / Click to Link" button on any loose file below, or right-click it and select "Link to Archive...".<br />
                  • <span class="font-semibold" style="color: var(--theme-text-primary);">Unlink:</span> Right-click the blue <span class="font-mono font-bold" style="color: var(--color-info);">[XL]</span> badge on a parent mod and select "Unlink .xl".
                </p>
              </div>
            {/if}

            {#if showUnassociatedXl}
              <div class="space-y-0">
                {#each unassociatedXlFiles as xl (xl.file_name + '_' + xl.enabled)}
                  <div
                    oncontextmenu={(e) => openContextMenu(e, xl, 'unassociated_xl')}
                    class="border-b px-2 flex items-center justify-between density-row cursor-context-menu bg-transparent border-[var(--theme-border)] hover:bg-[color-mix(in_srgb,var(--theme-accent)_12%,transparent)]"
                  >
                    <div class="flex items-center gap-2.5 min-w-0">
                      <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--color-info);" />
                      <span class="font-mono font-medium truncate" style="color: var(--theme-text-primary);">{xl.file_name}</span>
                      <span class="text-[10px] font-mono shrink-0 hidden sm:block" style="color: var(--theme-text-muted);">
                        {formatBytes(xl.size_bytes)}
                      </span>
                    </div>

                    <div class="flex items-center gap-2 shrink-0">
                      <button
                        type="button"
                        onclick={(e) => { e.stopPropagation(); targetXlToLink = xl.file_name; isLinkModalOpen = true; }}
                        class="text-[9px] font-mono px-2 py-0.5 rounded border font-semibold transition cursor-pointer hover:brightness-125"
                        style="background-color: color-mix(in srgb, var(--color-info) 15%, transparent); border-color: color-mix(in srgb, var(--color-info) 30%, transparent); color: var(--color-info);"
                      >
                        Active / Click to Link
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Conflict Summary Drawer -->
      {#if showConflictSummary}
        <aside class="w-80 rounded-lg border flex flex-col overflow-hidden shrink-0 shadow-xl transition-all duration-200 relative" style="border-color: var(--theme-border);">
          <!-- Glass Background -->
          <div class="absolute inset-0 z-0 pointer-events-none" style="background-color: color-mix(in srgb, var(--theme-surface) var(--theme-surface-opacity, 100%), transparent); backdrop-filter: blur(var(--theme-backdrop-blur, 0px)); -webkit-backdrop-filter: blur(var(--theme-backdrop-blur, 0px));"></div>

          <div class="relative z-10 p-3 border-b flex items-center justify-between" style="background-color: color-mix(in srgb, var(--theme-card) 50%, transparent); border-color: var(--theme-border);">
            <div class="flex items-center gap-2">
              <AlertTriangle class="h-4 w-4" style="color: var(--color-danger);" />
              <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Conflict Summary</span>
            </div>
            <span class="text-[10px] font-mono px-2 py-0.5 rounded-full border" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);">
              {conflictingArchives.length} Contested
            </span>
          </div>

          <div class="relative z-10 p-2 space-y-1 overflow-y-auto flex-1 max-h-[calc(100vh-170px)]">
            {#if conflictingArchives.length === 0}
              <div class="p-6 text-center text-xs" style="color: var(--theme-text-muted);">
                No load order conflicts detected.
              </div>
            {:else}
              {#each conflictingArchives as archive (archive.file_name + '_' + archive.enabled)}
                {@const isLosing = archive.loses.length > 0}
                {@const isDragged = draggedEntry?.file_name === archive.file_name}
                <button
                  type="button"
                  onclick={() => focusModInMainList(archive.file_name)}
                  class="w-full text-left rounded px-2.5 flex items-center justify-between gap-2 border transition-all duration-200 cursor-pointer density-row group
                    {isDragged 
                      ? (isLosing 
                          ? 'shadow-[0_0_12px_rgba(239,68,68,0.3)] scale-[1.02] z-10 relative' 
                          : 'shadow-[0_0_12px_rgba(34,197,94,0.3)] scale-[1.02] z-10 relative')
                      : 'hover:brightness-125'}"
                  style="{isDragged ? (isLosing ? 'border-color: var(--color-danger); background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);' : 'border-color: var(--color-success); background-color: color-mix(in srgb, var(--color-success) 15%, transparent);') : 'border-color: transparent; background-color: color-mix(in srgb, var(--theme-surface) 80%, transparent);'}"
                  title="Click to locate in main load order"
                >
                  <div class="flex items-center gap-2 min-w-0 flex-1">
                    <div class="h-2 w-2 rounded-full shrink-0 shadow-[0_0_4px_currentColor]" style="background-color: {isLosing ? 'var(--color-danger)' : 'var(--color-success)'}; color: {isLosing ? 'var(--color-danger)' : 'var(--color-success)'};"></div>
                    <span class="font-mono truncate font-medium" style="color: var(--theme-text-primary);">
                      {archive.file_name}
                    </span>
                  </div>

                  <div class="shrink-0">
                    {#if isLosing}
                      <span class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded border" style="background-color: color-mix(in srgb, var(--color-danger) 15%, transparent); color: var(--color-danger); border-color: color-mix(in srgb, var(--color-danger) 30%, transparent);">
                        -{archive.loses.length}
                      </span>
                    {:else}
                      <span class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded border" style="background-color: color-mix(in srgb, var(--color-success) 15%, transparent); color: var(--color-success); border-color: color-mix(in srgb, var(--color-success) 30%, transparent);">
                        +{archive.wins.length}
                      </span>
                    {/if}
                  </div>
                </button>
              {/each}
            {/if}
          </div>

          <div class="relative z-10 p-2.5 border-t text-[10px] flex items-center justify-center" style="border-color: var(--theme-border); background-color: color-mix(in srgb, var(--theme-card) 30%, transparent); color: var(--theme-text-muted);">
            <span>Click any mod to locate in main load order</span>
          </div>
        </aside>
      {/if}
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
  onCancel={() => dialogState.isOpen = false}
/>

<MoveFolderModal
  bind:isOpen={isMoveModalOpen}
  folders={availableFolders}
  onConfirm={handleMoveToFolder}
  onCancel={() => isMoveModalOpen = false}
/>

<LinkXlModal
  bind:isOpen={isLinkModalOpen}
  xlName={targetXlToLink || ''}
  archives={availableArchivesForLink}
  onConfirm={handleLinkXl}
  onCancel={() => isLinkModalOpen = false}
/>

<CategoryModal
  bind:isOpen={isCategoryModalOpen}
  mode={categoryModalMode}
  initialData={categoryModalData}
  onConfirm={handleCategoryModalConfirm}
  onCancel={() => isCategoryModalOpen = false}
/>

<MoveCategoryModal
  bind:isOpen={isMoveCategoryModalOpen}
  categories={labelsConfig.categories}
  hasExistingLabel={selectedMods.size > 0 && Array.from(selectedMods).some(m => labelsConfig.assignments[m])}
  onConfirm={handleMoveCategoryConfirm}
  onCancel={() => isMoveCategoryModalOpen = false}
/>

<NoteModal
  bind:isOpen={isNoteModalOpen}
  modName={targetNoteModName}
  initialNote={notesConfig[targetNoteModName] || ''}
  onConfirm={handleNoteConfirm}
  onCancel={() => isNoteModalOpen = false}
/>
