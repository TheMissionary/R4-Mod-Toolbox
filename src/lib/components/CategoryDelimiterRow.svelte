<script lang="ts">
  import type { ArchiveItem } from '$lib/types';
  import { GripVertical, Folder } from 'lucide-svelte';

  let {
    archive,
    originalIndex,
    isSource,
    isHighlighted,
    onDragStart,
    onPointerMove,
    onToggle,
    onContextMenu,
    registerNode
  }: {
    archive: ArchiveItem;
    originalIndex: number;
    isSource: boolean;
    isHighlighted: boolean;
    onDragStart: (e: PointerEvent, index: number) => void;
    onPointerMove: (e: PointerEvent, index: number) => void;
    onToggle: (archive: ArchiveItem) => void;
    onContextMenu: (e: MouseEvent, archive: ArchiveItem, type: 'archive' | 'xl') => void;
    registerNode: (node: HTMLElement, name: string) => any;
  } = $props();
</script>

<div
  role="group"
  aria-label="Category Delimiter: {archive.category_name || archive.file_name}"
  use:registerNode={archive.file_name}
  onpointermove={(e) => onPointerMove(e, originalIndex)}
  oncontextmenu={(e) => onContextMenu(e, archive, 'archive')}
  class="rounded-lg border transition-all duration-300 {isHighlighted ? 'border-[#76b900] ring-2 ring-[#76b900] bg-[#76b900]/20 shadow-[0_0_15px_rgba(118,185,0,0.35)] scale-[1.008]' : 'border-nvidia-accent/40 bg-gradient-to-r from-nvidia-card via-[#161a1e] to-nvidia-surface hover:border-nvidia-accent/70'} {archive.enabled ? 'opacity-100' : 'opacity-50'} {isSource ? 'opacity-20 border-dashed border-nvidia-accent/50' : ''} mt-2 mb-1"
>
  <div class="flex items-center justify-between px-3 py-2 gap-2 h-10 relative overflow-hidden">
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

      <Folder class="h-4 w-4 text-nvidia-accent shrink-0" />

      <span class="text-sm font-bold text-white uppercase tracking-wider truncate">
        {archive.category_name || archive.file_name.replace('[CAT] ', '').replace('.archive', '')}
      </span>
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded bg-nvidia-accent/10 border border-nvidia-accent/30 text-nvidia-accent uppercase tracking-widest">
        Physical Category
      </span>
    </div>
  </div>
</div>
