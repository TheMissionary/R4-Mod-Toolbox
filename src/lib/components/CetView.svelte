<script lang="ts">
  import { onMount } from 'svelte';
  import type { CetPluginItem, FileNode } from '$lib/types';
  import { Search, X, FolderSearch, Power, Copy, Check, Folder, FileCode, ChevronDown, ChevronRight, FileEdit } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { tick } from 'svelte';
  import { loadConfigFromDisk } from '$lib/theme';

  let {
    plugins = $bindable([]),
    gamePath = '',
    onStateChanged,
    targetHighlightMod = null
  }: {
    plugins: CetPluginItem[];
    gamePath: string;
    onStateChanged?: () => void;
    targetHighlightMod?: string | null;
  } = $props();

  let searchQuery = $state('');
  let copiedFeedback = $state(false);
  let customEditorPath = $state('');

  let highlightedModName = $state<string | null>(null);
  let highlightTimeoutId: number | null = null;
  const modNodeMap = new Map<string, HTMLElement>();

  // Lazy Loading State
  let expandedFolders = $state<Record<string, boolean>>({});
  let folderContents = $state<Record<string, FileNode[]>>({});
  let loadingFolders = $state<Record<string, boolean>>({});

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    plugin: CetPluginItem | null;
    fileNode: FileNode | null;
    targetType: 'plugin' | 'file';
  }>({
    visible: false,
    x: 0,
    y: 0,
    plugin: null,
    fileNode: null,
    targetType: 'plugin'
  });

  const EDITABLE_EXTS = ['ini', 'reds', 'yaml', 'tweak', 'xl', 'log', 'txt', 'md', 'lua', 'preset', 'json'];
  
  let isEditable = $derived.by(() => {
    if (contextMenu.targetType === 'file' && contextMenu.fileNode && !contextMenu.fileNode.is_dir) {
      return EDITABLE_EXTS.includes(contextMenu.fileNode.extension);
    }
    if (contextMenu.targetType === 'plugin' && contextMenu.plugin && !contextMenu.plugin.is_dir) {
      const ext = contextMenu.plugin.name.split('.').pop()?.toLowerCase() || '';
      return EDITABLE_EXTS.includes(ext);
    }
    return false;
  });

  onMount(() => {
    loadConfigFromDisk().then(config => {
      customEditorPath = config.customTextEditorPath || '';
    });
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

  async function handleToggle(plugin: CetPluginItem) {
    if (!gamePath) return;
    const nextState = !plugin.enabled;
    try {
      await invoke('toggle_plugin_state', {
        gamePath,
        modType: 'cet',
        modName: plugin.name,
        enable: nextState
      });
      plugin.enabled = nextState;
      plugins = [...plugins];
      if (onStateChanged) onStateChanged();
    } catch (err) {
      console.error(`Failed to toggle CET plugin ${plugin.name}:`, err);
    }
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

  function openContextMenu(event: MouseEvent, target: CetPluginItem | FileNode, type: 'plugin' | 'file') {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    const menuWidth = 230;
    const menuHeight = 160;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    if (type === 'plugin') {
      contextMenu = { visible: true, x: posX, y: posY, plugin: target as CetPluginItem, fileNode: null, targetType: 'plugin' };
    } else {
      contextMenu = { visible: true, x: posX, y: posY, plugin: null, fileNode: target as FileNode, targetType: 'file' };
    }
  }

  function closeContextMenu() {
    if (contextMenu.visible) {
      contextMenu.visible = false;
      contextMenu.plugin = null;
      contextMenu.fileNode = null;
      copiedFeedback = false;
    }
  }

  function handleContextMenuToggle() {
    if (contextMenu.plugin) {
      handleToggle(contextMenu.plugin);
    }
    closeContextMenu();
  }

  async function handleContextMenuShowInExplorer() {
    const path = contextMenu.targetType === 'plugin' ? contextMenu.plugin?.path : contextMenu.fileNode?.path;
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
    } else if (contextMenu.targetType === 'plugin' && contextMenu.plugin && !contextMenu.plugin.is_dir) {
      filePath = contextMenu.plugin.path;
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
    const name = contextMenu.targetType === 'plugin' ? contextMenu.plugin?.name : contextMenu.fileNode?.name;
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

  let filteredPlugins = $derived.by(() => {
    if (!searchQuery.trim()) return plugins;
    const q = searchQuery.toLowerCase().trim();
    return plugins.filter(p => p.name.toLowerCase().includes(q));
  });

  let activeCount = $derived(plugins.filter(p => p.enabled).length);
</script>

<svelte:window onclick={closeContextMenu} />

<!-- Custom Context Menu -->
{#if contextMenu.visible && (contextMenu.plugin || contextMenu.fileNode)}
  <div
    class="fixed z-50 w-56 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center justify-between gap-2">
      <div class="flex items-center gap-1.5 min-w-0 flex-1">
        {#if contextMenu.targetType === 'plugin'}
          {#if contextMenu.plugin!.is_dir}
            <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {:else}
            <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {/if}
          <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={contextMenu.plugin!.name}>
            {contextMenu.plugin!.name}
          </span>
        {:else}
          {#if contextMenu.fileNode!.is_dir}
            <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {:else}
            <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {/if}
          <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={contextMenu.fileNode!.name}>
            {contextMenu.fileNode!.name}
          </span>
        {/if}
      </div>
      {#if contextMenu.targetType === 'plugin'}
        <span class="text-[9px] font-mono uppercase px-1 py-0.2 rounded border {contextMenu.plugin!.enabled ? 'bg-nvidia-accent/15 border-nvidia-accent/40 text-nvidia-accent' : 'bg-red-500/15 border-red-500/40 text-red-400'} shrink-0">
          {contextMenu.plugin!.enabled ? 'Active' : 'Disabled'}
        </span>
      {/if}
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

      {#if contextMenu.targetType === 'plugin'}
        <button
          type="button"
          onclick={handleContextMenuToggle}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <Power class="h-3.5 w-3.5 {contextMenu.plugin!.enabled ? 'text-amber-400' : 'text-nvidia-accent'}" />
          <span>{contextMenu.plugin!.enabled ? 'Disable Mod' : 'Enable Mod'}</span>
        </button>
      {/if}

      {#if isEditable}
        <button
          type="button"
          onclick={handleOpenInEditor}
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        >
          <FileEdit class="h-3.5 w-3.5 text-cyan-400 group-hover:brightness-110" />
          <span>Open in Editor</span>
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
    <div class="flex items-center px-3 py-1 text-[10px] font-mono text-nvidia-text-muted" style="padding-left: {depth * 1.25 + 1}rem">
      <div class="h-3 w-3 rounded-full border-2 border-nvidia-surface border-t-nvidia-accent animate-spin mr-2"></div>
      Loading contents...
    </div>
  {:else if folderContents[parentPath]}
    {#each folderContents[parentPath] as node}
      <div
        oncontextmenu={(e) => openContextMenu(e, node, 'file')}
        class="flex items-center justify-between px-3 rounded border-b border-nvidia-border/30 hover:bg-nvidia-surface/70 transition density-row cursor-context-menu"
        style="padding-left: {depth * 1.25 + 1}rem"
      >
        <div class="flex items-center gap-2.5 min-w-0">
          {#if node.is_dir}
            <button
              type="button"
              onclick={(e) => { e.stopPropagation(); toggleFolder(node.path); }}
              class="p-0.5 hover:bg-nvidia-card rounded text-nvidia-text-muted transition cursor-pointer"
            >
              {#if expandedFolders[node.path]}
                <ChevronDown class="h-3.5 w-3.5" />
              {:else}
                <ChevronRight class="h-3.5 w-3.5" />
              {/if}
            </button>
            <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {:else}
            <div class="w-[18px] shrink-0"></div>
            <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
          {/if}
          <span class="font-mono text-nvidia-text-primary truncate text-[11px]">{node.name}</span>
        </div>
        <div class="text-[10px] font-mono text-nvidia-text-muted shrink-0">
          {#if !node.is_dir}{formatBytes(node.size_bytes)}{/if}
        </div>
      </div>
      {#if node.is_dir && expandedFolders[node.path]}
        {@render folderTree(node.path, depth + 1)}
      {/if}
    {/each}
  {/if}
{/snippet}

<div class="flex flex-col h-full select-none" oncontextmenu={(e) => e.preventDefault()}>
  <div class="flex items-center justify-between gap-3 mb-3 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-nvidia-text-muted" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter CET plugins..."
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
        Active: <span class="text-nvidia-accent font-semibold">{activeCount}</span> of {plugins.length}
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto space-y-1 pr-1">
    {#if filteredPlugins.length === 0}
      <div class="p-8 rounded border border-nvidia-border/60 bg-nvidia-surface/30 text-center text-xs font-mono text-nvidia-text-muted">
        No CET plugins detected.
      </div>
    {:else}
      {#each filteredPlugins as plugin (plugin.name)}
        {@const isHighlighted = highlightedModName === plugin.name}
        <div class="flex flex-col">
          <div
            use:registerModNode={plugin.name}
            data-mod-item="true"
            data-mod-name={plugin.name}
            data-mod-path={plugin.path}
            data-mod-type="cet"
            data-is-file={!plugin.is_dir}
            oncontextmenu={(e) => openContextMenu(e, plugin, 'plugin')}
            class="flex items-center justify-between px-3 rounded border transition-all duration-300 density-row cursor-context-menu
              {isHighlighted ? 'border-nvidia-accent ring-2 ring-nvidia-accent bg-nvidia-accent/20 scale-[1.008] z-10 relative' : 'border-nvidia-border/70 bg-nvidia-surface/40 hover:bg-nvidia-surface/70'}
              {plugin.enabled ? 'text-nvidia-text-primary' : 'opacity-50 text-nvidia-text-muted'}"
            style={isHighlighted ? 'box-shadow: 0 0 15px color-mix(in srgb, var(--theme-accent) 35%, transparent);' : ''}
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- Subtle Calmed Switch -->
              <button
                type="button"
                onclick={() => handleToggle(plugin)}
                aria-label={plugin.enabled ? "Disable plugin " + plugin.name : "Enable plugin " + plugin.name}
                class="w-7 h-4 rounded-full transition-colors relative p-0.5 shrink-0 cursor-pointer {plugin.enabled ? 'bg-zinc-700/80 border border-zinc-600' : 'bg-zinc-900/90 border border-zinc-800'}"
              >
                <div class="h-2.5 w-2.5 rounded-full transition-transform transform {plugin.enabled ? 'translate-x-3 bg-zinc-100 shadow-xs' : 'translate-x-0 bg-zinc-500'}"></div>
              </button>

              <!-- Folder vs Loose File Icon with Chevron -->
              {#if plugin.is_dir}
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); toggleFolder(plugin.path); }}
                  class="p-0.5 hover:bg-nvidia-card rounded text-nvidia-text-muted transition cursor-pointer"
                >
                  {#if expandedFolders[plugin.path]}
                    <ChevronDown class="h-3.5 w-3.5" />
                  {:else}
                    <ChevronRight class="h-3.5 w-3.5" />
                  {/if}
                </button>
                <Folder class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
              {:else}
                <div class="w-[18px] shrink-0"></div>
                <FileCode class="h-3.5 w-3.5 text-nvidia-text-muted shrink-0" />
              {/if}

              <span class="font-mono truncate {plugin.enabled ? 'text-nvidia-text-primary' : 'line-through text-nvidia-text-muted'}">
                {plugin.name}
              </span>
            </div>

            <div class="flex items-center gap-3 text-[11px] font-mono text-nvidia-text-muted shrink-0">
              {#if plugin.has_init}
                <span class="px-1.5 py-0.2 rounded bg-nvidia-surface border border-nvidia-border/60 text-[10px] text-nvidia-accent">
                  init.lua
                </span>
              {/if}
              <span>{formatBytes(plugin.size_bytes)}</span>
            </div>
          </div>

          <!-- Render Lazy Loaded Children -->
          {#if plugin.is_dir && expandedFolders[plugin.path]}
            {@render folderTree(plugin.path, 1)}
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
