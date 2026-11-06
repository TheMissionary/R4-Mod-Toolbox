<script lang="ts">
  import type { ArchiveItem } from '$lib/types';
  import { GripVertical, Folder, FolderOpen, Pencil, Trash2, MoreVertical, ChevronDown, ChevronRight, Check } from 'lucide-svelte';

  let {
    archive,
    originalIndex,
    isSource,
    isHighlighted,
    collapsed = false,
    modCount = 0,
    onToggleCollapse,
    onDragStart,
    onPointerMove,
    onToggle,
    onContextMenu,
    onRenameFolder,
    onDeleteFolder,
    registerNode
  }: {
    archive: ArchiveItem;
    originalIndex: number;
    isSource: boolean;
    isHighlighted: boolean;
    collapsed?: boolean;
    modCount?: number;
    onToggleCollapse?: (folderFileName: string) => void;
    onDragStart: (e: PointerEvent, index: number) => void;
    onPointerMove: (e: PointerEvent, index: number) => void;
    onToggle: (archive: ArchiveItem) => void;
    onContextMenu: (e: MouseEvent, archive: ArchiveItem, type: 'archive' | 'xl') => void;
    onRenameFolder?: (archive: ArchiveItem) => void;
    onDeleteFolder?: (archive: ArchiveItem) => void;
    registerNode: (node: HTMLElement, name: string) => any;
  } = $props();
</script>

<div
  role="group"
  aria-label="Group: {archive.category_name || archive.file_name}"
  use:registerNode={archive.file_name}
  onpointermove={(e) => onPointerMove(e, originalIndex)}
  oncontextmenu={(e) => onContextMenu(e, archive, 'archive')}
  class="group transition-all duration-200 border-b {isHighlighted ? 'ring-1 z-10 relative' : ''} {archive.enabled ? 'opacity-100' : 'opacity-50'} {isSource ? 'opacity-20 border-dashed' : ''}"
  style="border-color: var(--theme-border); {isHighlighted ? 'background-color: color-mix(in srgb, var(--theme-accent) 15%, transparent); border-color: var(--theme-accent);' : 'background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent);'}"
>
  <div class="flex items-center justify-between px-2 gap-2 density-row relative overflow-hidden">
    <div class="absolute left-0 top-0 bottom-0 w-0.5" style="background-color: var(--theme-accent);"></div>

    <div class="flex items-center gap-3 min-w-0 flex-1 pl-2">
      <div
        role="button"
        tabindex="0"
        aria-label="Drag to adjust load order"
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
        onpointerdown={(e) => { e.stopPropagation(); onDragStart(e, originalIndex); }}
        class="cursor-grab active:cursor-grabbing p-1 rounded transition shrink-0 hover:bg-[var(--theme-surface)]"
        style="color: var(--theme-text-muted);"
        title="Drag to adjust load order"
      >
        <GripVertical class="h-3.5 w-3.5" />
      </div>

      <!-- Minimal Checkbox -->
      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onToggle(archive); }}
        aria-label={archive.enabled ? "Disable group" : "Enable group"}
        class="w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors shrink-0 cursor-pointer"
        style="border-color: {archive.enabled ? 'var(--theme-accent)' : 'var(--theme-text-muted)'}; background-color: {archive.enabled ? 'var(--theme-accent)' : 'transparent'};"
      >
        {#if archive.enabled}
          <Check class="h-3 w-3" style="color: var(--theme-accent-contrast);" strokeWidth={3} />
        {/if}
      </button>

      <!-- Unified Chevron + Folder + Name Toggle -->
      <button
        type="button"
        onclick={() => onToggleCollapse && onToggleCollapse(archive.file_name)}
        class="flex items-center gap-1.5 transition cursor-pointer min-w-0 hover:brightness-125"
        style="color: var(--theme-text-primary);"
        title={collapsed ? "Expand group" : "Collapse group"}
      >
        {#if collapsed}
          <ChevronRight class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
        {:else}
          <ChevronDown class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
          <FolderOpen class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-text-muted);" />
        {/if}
        <span class="font-bold uppercase tracking-wider truncate text-[11px]">
          {archive.category_name || archive.file_name.replace('[DIR] ', '').replace('.txt', '')}
        </span>
      </button>

      <!-- Dynamic Nested Mod Count Badge -->
      <span class="text-[10px] font-mono shrink-0 opacity-80" style="color: var(--theme-text-muted);">
        ({modCount})
      </span>
    </div>

    <!-- Right Controls -->
    <div class="flex items-center gap-1.5 shrink-0">
      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onRenameFolder && onRenameFolder(archive); }}
        class="p-1 rounded transition cursor-pointer hover:bg-[var(--theme-surface)]"
        style="color: var(--theme-text-muted);"
        title="Rename group"
      >
        <Pencil class="h-3.5 w-3.5 hover:text-[var(--theme-accent)]" />
      </button>

      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onDeleteFolder && onDeleteFolder(archive); }}
        class="p-1 rounded transition cursor-pointer hover:bg-[color-mix(in_srgb,var(--color-danger)_15%,transparent)]"
        style="color: var(--theme-text-muted);"
        title="Delete group (mods remain preserved)"
      >
        <Trash2 class="h-3.5 w-3.5 hover:text-[var(--color-danger)]" />
      </button>

      <div class="w-[1px] h-3 mx-0.5" style="background-color: var(--theme-border);"></div>

      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onContextMenu(e, archive, 'archive'); }}
        class="p-1 rounded transition cursor-pointer opacity-0 group-hover:opacity-100 hover:bg-[var(--theme-surface)]"
        style="color: var(--theme-text-muted);"
        title="More options"
      >
        <MoreVertical class="h-3.5 w-3.5 hover:text-[var(--theme-text-primary)]" />
      </button>
    </div>
  </div>
</div>