<script lang="ts">
  import { tick } from 'svelte';
  import { X, Search, Plus, Tag, Trash2 } from 'lucide-svelte';
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
      class="w-full max-w-md rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b flex items-center justify-between" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5">
          <Tag class="h-4 w-4" style="color: var(--color-info);" />
          <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Assign Category Label</span>
        </div>
        <button
          type="button"
          onclick={() => { isOpen = false; onCancel(); }}
          class="p-1 rounded transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          <X class="h-4 w-4 hover:text-[var(--theme-text-primary)]" />
        </button>
      </div>

      <div class="p-3 border-b relative" style="background-color: color-mix(in srgb, var(--theme-surface) 50%, transparent); border-color: var(--theme-border);">
        <Search class="absolute left-5 top-1/2 -translate-y-1/2 h-4 w-4" style="color: var(--theme-text-muted);" />
        <input
          bind:this={inputEl}
          type="text"
          bind:value={searchQuery}
          oninput={() => selectedIndex = -1}
          placeholder="Search categories..."
          class="w-full pl-9 pr-3 py-2 border rounded text-xs focus:outline-none font-sans"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        <!-- Create New Action -->
        <button
          type="button"
          onclick={() => onConfirm('[CREATE_NEW]')}
          class="w-full text-left px-3 py-2 rounded text-xs font-bold transition flex items-center gap-2 cursor-pointer border"
          style="color: var(--color-info); border-color: {selectedIndex === -1 ? 'var(--color-info)' : 'transparent'}; background-color: {selectedIndex === -1 ? 'color-mix(in srgb, var(--color-info) 15%, transparent)' : 'transparent'};"
        >
          <Plus class="h-4 w-4" />
          <span>{searchQuery.trim() ? `+ Create "${searchQuery.trim()}" Category...` : '+ Create New Category...'}</span>
        </button>

        {#if hasExistingLabel}
          <button
            type="button"
            onclick={() => onConfirm('[REMOVE]')}
            class="w-full text-left px-3 py-2 rounded text-xs font-semibold transition flex items-center gap-2 cursor-pointer border border-transparent hover:bg-[color-mix(in_srgb,var(--color-danger)_10%,transparent)] hover:border-[color-mix(in_srgb,var(--color-danger)_20%,transparent)]"
            style="color: var(--color-danger);"
          >
            <Trash2 class="h-4 w-4" style="color: var(--color-danger);" />
            <span>Remove Category Label</span>
          </button>
        {/if}

        <div class="h-px my-1 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

        {#each filtered as cat, i}
          <button
            type="button"
            onclick={() => onConfirm(cat.id)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center justify-between gap-2.5 cursor-pointer border"
            style="color: var(--theme-text-primary); border-color: {i === selectedIndex ? 'var(--theme-border)' : 'transparent'}; background-color: {i === selectedIndex ? 'var(--theme-surface)' : 'transparent'};"
          >
            <div class="flex items-center gap-2.5 truncate">
              <div class="h-2.5 w-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {cat.color};"></div>
              <span class="truncate">{cat.name}</span>
            </div>
            {#if cat.stashed}
              <span class="text-[9px] font-mono uppercase px-1.5 py-0.2 rounded border shrink-0" style="background-color: color-mix(in srgb, var(--color-warning) 10%, transparent); border-color: color-mix(in srgb, var(--color-warning) 30%, transparent); color: var(--color-warning);">
                In Stash
              </span>
            {/if}
          </button>
        {/each}
        
        {#if filtered.length === 0 && searchQuery.trim() !== ''}
          <div class="p-3 text-center text-xs" style="color: var(--theme-text-muted);">
            Press Enter or click above to create this category.
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}