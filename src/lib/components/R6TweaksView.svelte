<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import type { R6TweaksItem, FileNode } from '$lib/types';
  import { Search, X, FolderSearch, Power, Copy, Check, Folder, FileCode, ChevronDown, ChevronRight, FileEdit, MoreVertical, FolderOpen, EyeOff, Clock, MessageSquare } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick } from 'svelte';
  import { loadConfigFromDisk, isModNew } from '$lib/theme';
  import NoteModal from '$lib/components/NoteModal.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';

  let {
    tweaks = $bindable([]),
    gamePath = '',
    onStateChanged,
    targetHighlightMod = null,
    newModsHours = $bindable(6),
    showNewMods = $bindable(false),
    onNewModsConfigChanged,
    onRouteToHistory,
    currentTab = 'r6tweaks',
    globalSelectedPath = $bindable('')
  }: {
    tweaks: R6TweaksItem[];
    gamePath: string;
    onStateChanged?: () => void;
    targetHighlightMod?: string | null;
    newModsHours?: number;
    showNewMods?: boolean;
    onNewModsConfigChanged?: () => void;
    onRouteToHistory?: (modName: string) => void;
    currentTab?: string;
    globalSelectedPath?: string;
  } = $props();

  let searchQuery = $state('');
  let copiedFeedback = $state(false);
  let customEditorPath = $state('');

  let highlightedModName = $state<string | null>(null);
  let selectedMod = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;
  const modNodeMap = new Map<string, HTMLElement>();

  let expandedFolders = $state<Record<string, boolean>>({});
  let folderContents = $state<Record<string, FileNode[]>>({});
  let loadingFolders = $state<Record<string, boolean>>({});

  let notesConfig = $state<Record<string, string>>({});
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

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    tweak: R6TweaksItem | null;
    fileNode: FileNode | null;
    targetType: 'tweak' | 'file';
  }>({
    visible: false,
    x: 0,
    y: 0,
    tweak: null,
    fileNode: null,
    targetType: 'tweak'
  });

  const EDITABLE_EXTS = ['ini', 'reds', 'yaml', 'tweak', 'xl', 'log', 'txt', 'md', 'lua', 'preset', 'json'];
  
  let isEditable = $derived.by(() => {
    if (contextMenu.targetType === 'file' && contextMenu.fileNode && !contextMenu.fileNode.is_dir) {
      return EDITABLE_EXTS.includes(contextMenu.fileNode.extension);
    }
    if (contextMenu.targetType === 'tweak' && contextMenu.tweak && !contextMenu.tweak.is_dir) {
      const ext = contextMenu.tweak.name.split('.').pop()?.toLowerCase() || '';
      return EDITABLE_EXTS.includes(ext);
    }
    return false;
  });

  onMount(() => {
    loadConfigFromDisk().then(config => {
      customEditorPath = config.customTextEditorPath || '';
    });
  });

  $effect(() => {
    if (gamePath) {
      invoke<Record<string, string>>('load_notes_config').then(res => {
        notesConfig = res;
      }).catch(err => console.error('Failed to load notes config:', err));
    }
  });

  // Tab Exit Effect: Clears Search, Selections, and Highlights
  $effect(() => {
    if (currentTab !== 'r6tweaks') {
      untrack(() => {
        if (searchQuery !== '') searchQuery = '';
        if (selectedMod !== null) selectedMod = null;
        if (highlightedModName !== null) highlightedModName = null;
      });
    }
  });

  function watchVisibility(node: HTMLElement) {
    const observer = new IntersectionObserver((entries) => {
      if (!entries[0].isIntersecting) {
        expandedFolders = {};
        selectedMod = null;
      }
    }, { threshold: 0 });
    observer.observe(node);
    return {
      destroy() { observer.disconnect(); }
    };
  }

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

  let toggleDebounceTimer: number | null = null;

  async function handleToggle(tweak: R6TweaksItem) {
    if (!gamePath) return;
    const nextState = !tweak.enabled;
    
    tweak.enabled = nextState;
    tweaks = [...tweaks];

    (async () => {
      try {
        await invoke('toggle_plugin_state', {
          gamePath,
          modType: 'r6tweaks',
          modName: tweak.name,
          enable: nextState
        });
        
        if (toggleDebounceTimer) clearTimeout(toggleDebounceTimer);
        toggleDebounceTimer = window.setTimeout(() => {
          if (onStateChanged) onStateChanged();
        }, 600);
      } catch (err) {
        console.error(`Failed to toggle R6 Tweak ${tweak.name}:`, err);
        tweak.enabled = !nextState;
        tweaks = [...tweaks];
      }
    })();
  }

  function confirmBulkAction(enable: boolean) {
    dialogState = {
      isOpen: true,
      title: enable ? 'Enable All R6\\Tweaks' : 'Disable All R6\\Tweaks',
      message: enable 
        ? 'Are you sure you want to enable all R6\\Tweaks? This will activate every mod in this list.' 
        : 'Are you sure you want to disable all R6\\Tweaks? This is a nuclear option that will deactivate every mod in this list.',
      mode: 'confirm',
      initialValue: '',
      confirmText: enable ? 'Enable All' : 'Disable All',
      isDanger: !enable,
      onConfirm: async () => {
        dialogState.isOpen = false;
        try {
          await invoke('bulk_toggle_mod_type', { gamePath, modType: 'r6tweaks', enable });
          tweaks = await invoke<R6TweaksItem[]>('get_r6tweaks_details', { gamePath });
          if (onStateChanged) onStateChanged();
        } catch (err) {
          console.error(err);
        }
      }
    };
  }

  async function toggleFolder(path: string) {
    expandedFolders[path] = !expandedFolders[path];
    if (expandedFolders[path] && !folderContents[path]) {
      loadingFolders[path] = true;
      try {
        folderContents[path] = await invoke<FileNode[]>('get_folder_contents', { folderPath: path });
      } catch (err) {
        console.error('Failed to load folder contents:', err);
      } finally {
        loadingFolders[path] = false;
      }
    }
  }

  let topLevelFolders = $derived(tweaks.filter(p => p.is_dir));
  let isAllExpanded = $derived(topLevelFolders.length > 0 && topLevelFolders.every(p => expandedFolders[p.path]));

  async function toggleAllFolders() {
    if (isAllExpanded) {
      expandedFolders = {};
    } else {
      for (const p of topLevelFolders) {
        expandedFolders[p.path] = true;
        if (!folderContents[p.path]) {
          loadingFolders[p.path] = true;
          try {
            folderContents[p.path] = await invoke<FileNode[]>('get_folder_contents', { folderPath: p.path });
          } catch (err) {}
          loadingFolders[p.path] = false;
        }
      }
    }
  }

  function openContextMenu(event: MouseEvent, target: R6TweaksItem | FileNode, type: 'tweak' | 'file') {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    const menuWidth = 230;
    const menuHeight = 200;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    if (type === 'tweak') {
      contextMenu = { visible: true, x: posX, y: posY, tweak: target as R6TweaksItem, fileNode: null, targetType: 'tweak' };
    } else {
      contextMenu = { visible: true, x: posX, y: posY, tweak: null, fileNode: target as FileNode, targetType: 'file' };
    }
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.tweak = null;
      contextMenu.fileNode = null;
      copiedFeedback = false;
    }
  }

  function handleContextMenuToggle() {
    if (contextMenu.tweak) {
      handleToggle(contextMenu.tweak);
    }
    closeContextMenu();
  }

  async function handleContextMenuShowInExplorer() {
    const path = contextMenu.targetType === 'tweak' ? contextMenu.tweak?.path : contextMenu.fileNode?.path;
    if (!path) return;
    try {
      await revealItemInDir(path);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeContextMenu();
  }

  async function handleOpenInEditor() {
    let filePath = '';
    if (contextMenu.targetType === 'file' && contextMenu.fileNode) {
      filePath = contextMenu.fileNode.path;
    } else if (contextMenu.targetType === 'tweak' && contextMenu.tweak && !contextMenu.tweak.is_dir) {
      filePath = contextMenu.tweak.path;
    }
    
    if (filePath) {
      try {
        await invoke('open_in_text_editor', { 
          filePath, 
          editorPath: customEditorPath 
        });
      } catch (err) {
        console.error('Failed to open editor:', err);
      }
    }
    closeContextMenu();
  }

  async function handleContextMenuCopyName() {
    const name = contextMenu.targetType === 'tweak' ? contextMenu.tweak?.name : contextMenu.fileNode?.name;
    if (!name) return;
    try {
      await navigator.clipboard.writeText(name);
      copiedFeedback = true;
      setTimeout(() => {
        closeContextMenu();
      }, 400);
    } catch (err) {
      console.error('Failed to copy name:', err);
      closeContextMenu();
    }
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

  let filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return tweaks;
    const q = searchQuery.toLowerCase().trim();
    return tweaks.filter(p => p.name.toLowerCase().includes(q));
  });
</script>

<svelte:window onclick={closeContextMenu} />

<!-- Custom Context Menu -->
{#if contextMenu.visible && (contextMenu.tweak || contextMenu.fileNode)}
  <div
    class="fixed z-50 w-56 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center justify-between gap-2" style="border-color: var(--theme-border); background-color: var(--theme-surface);">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.targetType === 'tweak'}
          {#if contextMenu.tweak?.is_dir}
            <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          {:else}
            <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          {/if}
          <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);" title={contextMenu.tweak?.name}>
            {contextMenu.tweak?.name}
          </span>
        {:else}
          {#if contextMenu.fileNode?.is_dir}
            <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          {:else}
            <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          {/if}
          <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);" title={contextMenu.fileNode?.name}>
            {contextMenu.fileNode?.name}
          </span>
        {/if}
      </div>
      {#if contextMenu.targetType === 'tweak' && contextMenu.tweak}
        <span class="text-[9px] font-mono uppercase px-1 py-0.2 rounded border shrink-0" style="{contextMenu.tweak.enabled ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: color-mix(in srgb, var(--theme-accent) 40%, transparent); color: var(--theme-accent);' : 'background-color: color-mix(in srgb, var(--color-danger) 15%, transparent); border-color: color-mix(in srgb, var(--color-danger) 40%, transparent); color: var(--color-danger);'}">
          {contextMenu.tweak.enabled ? 'Active' : 'Disabled'}
        </span>
      {/if}
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
          const name = contextMenu.targetType === 'tweak' ? contextMenu.tweak?.name : contextMenu.fileNode?.name;
          if (name && onRouteToHistory) onRouteToHistory(name);
          closeContextMenu();
        }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Clock class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Show in History</span>
      </button>

      {#if contextMenu.targetType === 'tweak' && contextMenu.tweak}
        <button
          type="button"
          onclick={handleContextMenuToggle}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <Power class="h-3.5 w-3.5" style="color: {contextMenu.tweak.enabled ? 'var(--color-warning)' : 'var(--theme-accent)'};" />
          <span>{contextMenu.tweak.enabled ? 'Disable Mod' : 'Enable Mod'}</span>
        </button>
      {/if}

      {#if isEditable}
        <button
          type="button"
          onclick={handleOpenInEditor}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
          style="color: var(--theme-text-primary);"
        >
          <FileEdit class="h-3.5 w-3.5" style="color: var(--color-info);" />
          <span>Open in Editor</span>
        </button>
      {/if}

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

      <button
        type="button"
        onclick={() => { 
          targetNoteModName = (contextMenu.targetType === 'tweak' ? contextMenu.tweak?.name : contextMenu.fileNode?.name) || ''; 
          closeContextMenu(); 
          isNoteModalOpen = true; 
        }}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
        <span>{notesConfig[(contextMenu.targetType === 'tweak' ? contextMenu.tweak?.name : contextMenu.fileNode?.name) || ''] ? 'Edit Note...' : 'Add Note...'}</span>
      </button>

      <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

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
            <span>Copy Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<!-- Recursive Folder Snippet -->
{#snippet folderTree(parentPath: string, depth: number)}
  {#if loadingFolders[parentPath]}
    <div class="flex items-center pr-3 py-1 text-[10px] font-mono" style="padding-left: {4.5 + (depth - 1) * 1.375}rem; color: var(--theme-text-muted);">
      <div class="h-3 w-3 rounded-full border-2 animate-spin mr-2" style="border-color: var(--theme-surface); border-top-color: var(--theme-accent);"></div>
      Loading contents...
    </div>
  {:else if folderContents[parentPath]}
    {#each folderContents[parentPath] as node}
      <div
        oncontextmenu={(e) => openContextMenu(e, node, 'file')}
        onpointerenter={() => globalSelectedPath = node.path}
        onpointerleave={() => globalSelectedPath = ''}
        class="group flex items-center justify-between pr-2 border-b transition density-row cursor-context-menu hover:bg-[var(--theme-surface)]"
        style="padding-left: {4.5 + (depth - 1) * 1.375}rem; border-color: var(--theme-border);"
      >
        <div class="flex items-center gap-2.5 min-w-0 flex-1">
          {#if node.is_dir}
            <button
              type="button"
              onclick={(e) => { e.stopPropagation(); toggleFolder(node.path); }}
              class="flex items-center gap-1.5 transition cursor-pointer min-w-0 hover:brightness-125"
              style="color: var(--theme-text-primary);"
            >
              {#if expandedFolders[node.path]}
                <ChevronDown class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                <FolderOpen class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
              {:else}
                <ChevronRight class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
              {/if}
              <span class="font-bold truncate text-[11px]">{node.name}</span>
            </button>
          {:else}
            <div class="flex items-center gap-1.5 min-w-0">
              <div class="w-4 shrink-0"></div>
              <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
              <span class="font-mono truncate text-[11px]" style="color: var(--theme-text-primary);">{node.name}</span>
            </div>
          {/if}
        </div>
        
        <div class="flex items-center gap-2 shrink-0">
          <div class="w-20 hidden lg:block"></div>
          <div class="w-16 justify-end text-[10px] font-mono hidden lg:flex" style="color: var(--theme-text-muted);">
            {#if !node.is_dir}{formatBytes(node.size_bytes)}{/if}
          </div>
          <div class="w-6 flex justify-end">
            <button
              type="button"
              onclick={(e) => { e.stopPropagation(); openContextMenu(e, node, 'file'); }}
              class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-card)]"
              style="color: var(--theme-text-muted);"
              title="More options"
            >
              <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
            </button>
          </div>
        </div>
      </div>
      {#if node.is_dir && expandedFolders[node.path]}
        {@render folderTree(node.path, depth + 1)}
      {/if}
    {/each}
  {/if}
{/snippet}

<div use:watchVisibility class="flex flex-col h-full select-none" oncontextmenu={(e) => e.preventDefault()}>
  
  <div class="flex items-center justify-between gap-3 mb-3 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter R6 Tweaks..."
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
    
    <div class="flex items-center gap-2.5 shrink-0">
      {#if showNewMods}
        <button
          type="button"
          onclick={() => { showNewMods = false; }}
          class="h-8 flex items-center gap-2 px-3 rounded border transition cursor-pointer shadow-sm hover:brightness-125"
          style="background-color: color-mix(in srgb, var(--theme-accent) 10%, transparent); border-color: color-mix(in srgb, var(--theme-accent) 60%, transparent);"
          title="Clear new mod highlights"
        >
          <span
            class="inline-flex items-center justify-center px-1.5 py-0.2 rounded text-[9px] font-mono font-bold tracking-wider uppercase shrink-0 shadow-xs"
            style="background-color: var(--theme-accent); color: var(--theme-accent-contrast);"
          >
            NEW
          </span>
          <EyeOff class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        </button>
      {/if}

      <button
        type="button"
        onclick={toggleAllFolders}
        disabled={topLevelFolders.length === 0}
        class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-125"
        style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
      >
        {#if isAllExpanded}
          <Folder class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Collapse All</span>
        {:else}
          <FolderOpen class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
          <span>Expand All</span>
        {/if}
      </button>

      <div class="w-px h-5 mx-1" style="background-color: var(--theme-border);"></div>

      <button
        onclick={() => confirmBulkAction(true)}
        class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
        style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        title="Enable All R6\Tweaks"
      >
        <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span class="hidden xl:inline">Enable All</span>
      </button>

      <button
        onclick={() => confirmBulkAction(false)}
        class="h-8 flex items-center gap-1.5 px-3 rounded border text-xs font-medium transition cursor-pointer hover:brightness-125"
        style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--color-danger);"
        title="Disable All R6\Tweaks"
      >
        <Power class="h-3.5 w-3.5" style="color: var(--color-danger);" />
        <span class="hidden xl:inline">Disable All</span>
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto space-y-0 pr-1">
    {#if filteredTweaks.length === 0}
      <div class="p-6 text-center text-xs border rounded" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);">
        No R6 Tweaks detected.
      </div>
    {:else}
      {#each filteredTweaks as tweak (tweak.name + '_' + tweak.enabled)}
        {@const isHighlighted = highlightedModName === tweak.name}
        {@const isSelected = selectedMod === tweak.name}
        <div class="flex flex-col">
          <div
            use:registerModNode={tweak.name}
            onclick={() => selectedMod = tweak.name}
            onpointerenter={() => globalSelectedPath = tweak.path}
            onpointerleave={() => globalSelectedPath = ''}
            data-mod-item="true"
            data-mod-name={tweak.name}
            data-mod-path={tweak.path}
            data-mod-type="r6tweaks"
            data-is-file={!tweak.is_dir}
            oncontextmenu={(e) => openContextMenu(e, tweak, 'tweak')}
            class="group flex items-center justify-between px-2 rounded-none border-b transition-all duration-200 density-row cursor-context-menu
              {isHighlighted ? 'ring-1 z-10 relative bg-[color-mix(in_srgb,var(--theme-accent)_15%,transparent)] border-[var(--theme-accent)] shadow-[0_0_15px_color-mix(in_srgb,var(--theme-accent)_35%,transparent)]' : 
              (isSelected ? 'bg-[color-mix(in_srgb,var(--theme-accent)_10%,transparent)] border-[color-mix(in_srgb,var(--theme-accent)_60%,transparent)]' : 
              'bg-transparent border-[var(--theme-border)] hover:bg-[color-mix(in_srgb,var(--theme-accent)_12%,transparent)]')}
              {tweak.enabled ? 'opacity-100' : 'opacity-50'}"
          >
            <div class="flex items-center gap-2.5 min-w-0 flex-1 pl-1">
              <!-- Solid Accent Checkbox with --theme-border when disabled -->
              <button
                type="button"
                onclick={(e) => { e.stopPropagation(); handleToggle(tweak); }}
                aria-label={tweak.enabled ? "Disable tweak " + tweak.name : "Enable tweak " + tweak.name}
                class="w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors shrink-0 cursor-pointer"
                style="border-color: {tweak.enabled ? 'var(--theme-accent)' : 'var(--theme-border)'}; background-color: {tweak.enabled ? 'var(--theme-accent)' : 'transparent'};"
              >
                {#if tweak.enabled}
                  <Check class="h-2.5 w-2.5" style="color: var(--theme-accent-contrast);" strokeWidth={3} />
                {/if}
              </button>

              {#if tweak.is_dir}
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); toggleFolder(tweak.path); }}
                  class="flex items-center gap-1.5 transition cursor-pointer min-w-0 hover:brightness-125"
                  style="color: var(--theme-text-primary);"
                >
                  {#if expandedFolders[tweak.path]}
                    <ChevronDown class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                    <FolderOpen class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                  {:else}
                    <ChevronRight class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                    <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                  {/if}
                  <span class="font-bold truncate text-[11px]">{tweak.name}</span>
                </button>
              {:else}
                <div class="flex items-center gap-1.5 min-w-0">
                  <div class="w-4 shrink-0"></div>
                  <FileCode class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
                  <span class="font-mono truncate text-[11px]" style="{tweak.enabled ? 'color: var(--theme-text-primary);' : 'color: var(--theme-text-muted); text-decoration: line-through;'}">
                    {tweak.name}
                  </span>
                </div>
              {/if}

              {#if showNewMods && isModNew(tweak.created_at_ms, newModsHours)}
                <span
                  class="inline-flex items-center justify-center px-1.5 py-0.2 rounded text-[9px] font-mono font-bold tracking-wider uppercase shrink-0 shadow-xs"
                  style="background-color: var(--theme-accent); color: var(--theme-accent-contrast);"
                  title="Installed/modified in the last {newModsHours} hours"
                >
                  NEW
                </span>
              {/if}
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <div class="w-4 flex items-center justify-center shrink-0">
                {#if notesConfig[tweak.name]}
                  <button 
                    type="button"
                    onclick={(e) => { e.stopPropagation(); targetNoteModName = tweak.name; isNoteModalOpen = true; }}
                    class="hover:scale-110 transition-transform cursor-pointer"
                  >
                    <MessageSquare class="h-3.5 w-3.5" style="color: var(--color-warning);" />
                  </button>
                {/if}
              </div>

              <div class="w-20 justify-end hidden lg:flex">
                <span class="px-1.5 py-0.2 rounded border text-[10px] font-semibold truncate" style="background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: color-mix(in srgb, var(--theme-accent) 30%, transparent); color: var(--theme-accent);">
                  {tweak.file_count} {tweak.file_count === 1 ? 'file' : 'files'}
                </span>
              </div>
              <div class="w-16 justify-end text-[10px] font-mono hidden lg:flex" style="color: var(--theme-text-muted);">
                {formatBytes(tweak.size_bytes)}
              </div>
              <div class="w-6 flex justify-end">
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); openContextMenu(e, tweak, 'tweak'); }}
                  class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-surface)]"
                  style="color: var(--theme-text-muted);"
                  title="More options"
                >
                  <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
                </button>
              </div>
            </div>
          </div>

          {#if tweak.is_dir && expandedFolders[tweak.path]}
            {@render folderTree(tweak.path, 1)}
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
