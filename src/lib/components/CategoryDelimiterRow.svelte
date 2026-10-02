<script lang="ts">
  import type { ArchiveItem } from '$lib/types';
  import { GripVertical, Folder, FolderOpen, Pencil, Trash2 } from 'lucide-svelte';

  let {
    archive,
    originalIndex,
    isSource,
    isHighlighted,
    collapsed = false,
    onToggleCollapse,
    onDragStart,
    onPointerMove,
    onToggle,
    onContextMenu,
    onRenameCategory,
    onDeleteCategory,
    registerNode
  }: {
    archive: ArchiveItem;
    originalIndex: number;
    isSource: boolean;
    isHighlighted: boolean;
    collapsed?: boolean;
    onToggleCollapse?: (categoryFileName: string) => void;
    onDragStart: (e: PointerEvent, index: number) => void;
    onPointerMove: (e: PointerEvent, index: number) => void;
    onToggle: (archive: ArchiveItem) => void;
    onContextMenu: (e: MouseEvent, archive: ArchiveItem, type: 'archive' | 'xl') => void;
    onRenameCategory?: (archive: ArchiveItem) => void;
    onDeleteCategory?: (archive: ArchiveItem) => void;
    registerNode: (node: HTMLElement, name: string) => any;
  } = $props();
</script>

<div
  role="group"
  aria-label="Category Delimiter: {archive.category_name || archive.file_name}"
  use:registerNode={archive.file_name}
  onpointermove={(e) => onPointerMove(e, originalIndex)}
  oncontextmenu={(e) => onContextMenu(e, archive, 'archive')}
  class="rounded-lg border transition-all duration-300 {isHighlighted ? 'border-[#76b900] ring-2 ring-[#76b900] bg-[#76b900]/20 shadow-[0_0_15px_rgba(118,185,0,0.35)] scale-[1.008]' : 'border-nvidia-accent/35 bg-nvidia-accent/8 hover:bg-nvidia-accent/12 hover:border-nvidia-accent/65'} {archive.enabled ? 'opacity-100' : 'opacity-50'} {isSource ? 'opacity-20 border-dashed border-nvidia-accent/50' : ''} mt-2 mb-1"
>
  <div class="flex items-center justify-between px-3 gap-2 density-row relative overflow-hidden">
    <!-- Left accent line -->
    <div class="absolute left-0 top-0 bottom-0 w-1 bg-nvidia-accent"></div>

    <div class="flex items-center gap-3 min-w-0 flex-1 pl-1">
      <div
        role="button"
        tabindex="0"
        aria-label="Drag to adjust load order"
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.preventDefault(); }}
        onpointerdown={(e) => { e.stopPropagation(); onDragStart(e, originalIndex); }}
        class="cursor-grab active:cursor-grabbing p-1 text-nvidia-text-muted hover:text-nvidia-accent shrink-0 rounded hover:bg-nvidia-surface/60 transition"
        title="Drag to adjust load order"
      >
        <GripVertical class="h-4 w-4" />
      </div>

      <button
        onclick={() => onToggle(archive)}
        aria-label={archive.enabled ? "Disable category marker" : "Enable category marker"}
        class="w-7 h-4 rounded-full transition relative p-0.5 shrink-0 {archive.enabled ? 'bg-nvidia-accent' : 'bg-nvidia-border'}"
      >
        <div class="h-3 w-3 rounded-full bg-black transition transform {archive.enabled ? 'translate-x-3': 'translate-x-0'}"></div>
      </button>

      <!-- Collapse / Expand Toggle Button -->
      <button
        type="button"
        onclick={() => onToggleCollapse && onToggleCollapse(archive.file_name)}
        class="text-nvidia-accent hover:text-nvidia-accent-hover transition shrink-0 p-1 rounded hover:bg-nvidia-surface/60 cursor-pointer"
        title={collapsed ? "Expand category" : "Collapse category"}
      >
        {#if collapsed}
          <Folder class="h-4 w-4" />
        {:else}
          <FolderOpen class="h-4 w-4" />
        {/if}
      </button>

      <span class="font-bold text-nvidia-text-primary uppercase tracking-wider truncate">
        {archive.category_name || archive.file_name.replace('[CAT] ', '').replace('.archive', '')}
      </span>
    </div>

    <!-- Right Controls: Badge + In-line Rename and Delete -->
    <div class="flex items-center gap-2 shrink-0">
      <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded bg-nvidia-accent/15 border border-nvidia-accent/35 text-nvidia-accent uppercase tracking-widest">
        Category Marker
      </span>

      <!-- In-line Rename Button -->
      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onRenameCategory && onRenameCategory(archive); }}
        class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-accent hover:bg-nvidia-surface/80 transition cursor-pointer"
        title="Rename category"
      >
        <Pencil class="h-3.5 w-3.5" />
      </button>

      <!-- In-line Trash Can Delete Button -->
      <button
        type="button"
        onclick={(e) => { e.stopPropagation(); onDeleteCategory && onDeleteCategory(archive); }}
        class="p-1 rounded text-nvidia-text-muted hover:text-red-400 hover:bg-red-500/10 transition cursor-pointer"
        title="Delete category (mods remain preserved)"
      >
        <Trash2 class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</div>
