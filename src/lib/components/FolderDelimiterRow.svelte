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
  class="group transition-all duration-200 border-b 
    {isHighlighted ? 'ring-1 z-10 relative bg-[color-mix(in_srgb,var(--theme-accent)_15%,transparent)] border-[var(--theme-accent)]' : 
    'hover:bg-[color-mix(in_srgb,var(--theme-accent)_12%,transparent)]'}"
  style="{!isHighlighted ? 'background: var(--theme-group-bg); border-color: var(--theme-group-border);' : ''}
  {archive.enabled ? 'opacity: 1;' : 'opacity: 0.5;'} 
  {isSource ? 'opacity: 0.2; border-style: dashed;' : ''}"
>
  <div class="flex items-center justify-between px-2 gap-1 density-row relative overflow-hidden">
    <div class="absolute left-0 top-0 bottom-0 w-0.5" style="background-color: var(--theme-accent);"></div>

    <!-- STRICT SLOT ARCHITECTURE: LEFT ZONE -->
    <div class="flex items-center gap-1 min-w-0 flex-1 pl-1">
      
      <!-- Slot 1: Drag Handle (w-6) -->
      <div class="w-6 flex justify-center shrink-0">
        <div
          role="button"
          tabindex="0"
          aria-label="Drag to adjust load order"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
          onpointerdown={(e) => { e.stopPropagation(); onDragStart(e, originalIndex); }}
          class="cursor-grab active:cursor-grabbing p-1 rounded transition hover:bg-[var(--theme-surface)]"
          style="color: var(--theme-text-muted);"
          title="Drag to adjust load order"
        >
          <GripVertical class="h-3.5 w-3.5" />
        </div>
      </div>

      <!-- Slot 2: Checkbox (w-6) -->
      <div class="w-6 flex justify-center shrink-0">
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); onToggle(archive); }}
          aria-label={archive.enabled ? "Disable group" : "Enable group"}
          class="w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors cursor-pointer"
          style="border-color: {archive.enabled ? 'var(--theme-accent)' : 'var(--theme-border)'}; background-color: {archive.enabled ? 'var(--theme-accent)' : 'transparent'};"
        >
          {#if archive.enabled}
            <Check class="h-2.5 w-2.5" style="color: var(--theme-accent-contrast);" strokeWidth={3} />
          {/if}
        </button>
      </div>

      <!-- Slot 3: Folder Expand/Collapse (w-10) -> Aligns with Conflict Dot/Chevron -->
      <div class="w-10 flex items-center justify-center shrink-0">
        <button
          type="button"
          onclick={() => onToggleCollapse && onToggleCollapse(archive.file_name)}
          class="p-0.5 rounded transition cursor-pointer hover:bg-[var(--theme-card)]"
          style="color: var(--theme-text-muted);"
          title={collapsed ? "Expand group" : "Collapse group"}
        >
          {#if collapsed}
            <ChevronRight class="h-3.5 w-3.5" />
          {:else}
            <ChevronDown class="h-3.5 w-3.5" />
          {/if}
        </button>
      </div>

      <!-- Slot 4: Folder Name & Mod Count -->
      <div class="flex items-center gap-1.5 min-w-0">
        {#if collapsed}
          <Folder class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-accent);" />
        {:else}
          <FolderOpen class="h-3.5 w-3.5 shrink-0" style="color: var(--theme-accent);" />
        {/if}
        <span class="font-bold uppercase tracking-wider truncate text-[11px]" style="color: var(--theme-text-primary);">
          {archive.category_name || archive.file_name.replace('[DIR] ', '').replace('.txt', '')}
        </span>
        <span class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded border shrink-0 opacity-90 ml-1" style="background-color: color-mix(in srgb, var(--theme-accent) 12%, transparent); color: var(--theme-accent); border-color: color-mix(in srgb, var(--theme-accent) 30%, transparent);">
          {modCount} {modCount === 1 ? 'mod' : 'mods'}
        </span>
      </div>
    </div>

    <!-- STRICT SLOT ARCHITECTURE: RIGHT ZONE -->
    <div class="flex items-center gap-1 shrink-0">
      
      <!-- Slot 1: Category Label (w-28) -> Empty for folders -->
      <div class="w-28"></div>

      <!-- Slot 2: XL Icon (w-6) -> Empty for folders -->
      <div class="w-6"></div>

      <!-- Slot 3: Note Icon (w-6) -> Empty for folders -->
      <div class="w-6"></div>

      <!-- Slot 4: Multi-Link (w-10) -> Empty for folders -->
      <div class="w-10"></div>

      <!-- Slot 5: File Size / Mod Count (w-16) -> Empty for folders -->
      <div class="w-16 hidden lg:block"></div>

      <!-- Slot 6: Context Menu (w-6) -->
      <div class="w-6 flex justify-end shrink-0">
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
</div>