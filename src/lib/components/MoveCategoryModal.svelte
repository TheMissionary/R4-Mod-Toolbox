<script lang="ts">
  import { tick } from 'svelte';
  import { X, FolderInput, Search, Plus, Tag, Trash2 } from 'lucide-svelte';
  import type { CategoryLabel } from '$lib/types';

  let {
    isOpen = $bindable(false),
    categories = [],
    hasExistingLabel = false,
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    categories: CategoryLabel[];
    hasExistingLabel?: boolean;
    onConfirm: (actionOrId: string) => void;
    onCancel: () => void;
  } = $props();

  let searchQuery = $state('');
  let selectedIndex = $state(-1);
  let inputEl = $state<HTMLInputElement | null>(null);

  let filtered = $derived(
    categories.filter(c => c.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  $effect(() => {
    if (isOpen) {
      searchQuery = '';
      selectedIndex = -1;
      tick().then(() => inputEl?.focus());
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      isOpen = false;
      onCancel();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (selectedIndex < filtered.length - 1) selectedIndex++;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (selectedIndex > -1) selectedIndex--;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (selectedIndex === -1) {
        onConfirm('[CREATE_NEW]');
      } else if (filtered[selectedIndex]) {
        onConfirm(filtered[selectedIndex].id);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={() => { isOpen = false; onCancel(); }}
    role="presentation"
  >
    <div
      class="w-full max-w-md rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40">
        <div class="flex items-center gap-2.5">
          <Tag class="h-4 w-4 text-cyan-400" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Assign Category Label</span>
        </div>
        <button
          type="button"
          onclick={() => { isOpen = false; onCancel(); }}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="p-3 border-b border-nvidia-border bg-nvidia-surface/20 relative">
        <Search class="absolute left-5 top-1/2 -translate-y-1/2 h-4 w-4 text-nvidia-text-muted" />
        <input
          bind:this={inputEl}
          type="text"
          bind:value={searchQuery}
          oninput={() => selectedIndex = -1}
          placeholder="Search categories..."
          class="w-full pl-9 pr-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-cyan-400 font-sans"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        <!-- Create New Action -->
        <button
          type="button"
          onclick={() => onConfirm('[CREATE_NEW]')}
          class="w-full text-left px-3 py-2 rounded text-xs font-bold transition flex items-center gap-2 cursor-pointer text-cyan-400 border {selectedIndex === -1 ? 'bg-cyan-500/15 border-cyan-500/30' : 'border-transparent hover:bg-cyan-500/10'}"
        >
          <Plus class="h-4 w-4" />
          <span>{searchQuery.trim() ? `+ Create "${searchQuery.trim()}" Category...` : '+ Create New Category...'}</span>
        </button>

        {#if hasExistingLabel}
          <button
            type="button"
            onclick={() => onConfirm('[REMOVE]')}
            class="w-full text-left px-3 py-2 rounded text-xs font-semibold transition flex items-center gap-2 cursor-pointer text-red-400 border border-transparent hover:bg-red-500/10 hover:border-red-500/20"
          >
            <Trash2 class="h-4 w-4 text-red-400" />
            <span>Remove Category Label</span>
          </button>
        {/if}

        <div class="h-px bg-nvidia-border/50 my-1 mx-1"></div>

        {#each filtered as cat, i}
          <button
            type="button"
            onclick={() => onConfirm(cat.id)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center justify-between gap-2.5 cursor-pointer text-nvidia-text-primary border {i === selectedIndex ? 'bg-nvidia-surface border-nvidia-border shadow-sm' : 'border-transparent hover:bg-nvidia-surface/60'}"
          >
            <div class="flex items-center gap-2.5 truncate">
              <div class="h-2.5 w-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {cat.color};"></div>
              <span class="truncate">{cat.name}</span>
            </div>
            {#if cat.stashed}
              <span class="text-[9px] font-mono uppercase px-1.5 py-0.2 rounded bg-amber-500/10 border border-amber-500/30 text-amber-400 shrink-0">
                In Stash
              </span>
            {/if}
          </button>
        {/each}
        
        {#if filtered.length === 0 && searchQuery.trim() !== ''}
          <div class="p-3 text-center text-xs text-nvidia-text-muted">
            Press Enter or click above to create this category.
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}