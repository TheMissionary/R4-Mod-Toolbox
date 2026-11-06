<script lang="ts">
  import { tick } from 'svelte';
  import { X, FolderInput, Search, FolderPlus } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    folders = [],
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    folders: { name: string, file_name: string }[];
    onConfirm: (fileName: string) => void;
    onCancel: () => void;
  } = $props();

  let searchQuery = $state('');
  let selectedIndex = $state(-2);
  let inputEl = $state<HTMLInputElement | null>(null);

  let filtered = $derived(
    folders.filter(c => c.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  $effect(() => {
    if (isOpen) {
      searchQuery = '';
      selectedIndex = -2;
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
      if (selectedIndex === -2) {
        selectedIndex = -1;
      } else if (selectedIndex < filtered.length - 1) {
        selectedIndex++;
      } else {
        selectedIndex = -1;
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (selectedIndex === -2) {
        selectedIndex = filtered.length - 1;
      } else if (selectedIndex > -1) {
        selectedIndex--;
      } else {
        selectedIndex = filtered.length - 1;
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (selectedIndex >= 0 && filtered.length > 0) {
        onConfirm(filtered[selectedIndex].file_name);
      } else {
        onConfirm(`[CREATE_NEW]:${searchQuery}`);
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
          <FolderInput class="h-4 w-4" style="color: var(--theme-accent);" />
          <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Move to Group</span>
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
          oninput={() => selectedIndex = -2}
          placeholder="Search groups..."
          class="w-full pl-9 pr-3 py-2 border rounded text-xs focus:outline-none"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        <button
          type="button"
          onclick={() => onConfirm(`[CREATE_NEW]:${searchQuery}`)}
          class="w-full text-left px-3 py-2 rounded text-xs font-bold transition flex items-center gap-2 cursor-pointer border"
          style="color: var(--theme-accent); border-color: {selectedIndex === -1 ? 'var(--theme-accent)' : 'transparent'}; background-color: {selectedIndex === -1 ? 'color-mix(in srgb, var(--theme-accent) 15%, transparent)' : 'transparent'};"
        >
          <FolderPlus class="h-4 w-4" />
          {searchQuery ? `+ Create "${searchQuery}" Group...` : '+ Create New Group...'}
        </button>

        {#if filtered.length > 0}
          <div class="h-px my-1.5 mx-1" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>
        {/if}

        {#each filtered as cat, i}
          <button
            type="button"
            onclick={() => onConfirm(cat.file_name)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center gap-2 cursor-pointer border"
            style="color: var(--theme-text-primary); border-color: {i === selectedIndex ? 'var(--theme-border)' : 'transparent'}; background-color: {i === selectedIndex ? 'var(--theme-surface)' : 'transparent'};"
          >
            <FolderInput class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
            {cat.name}
          </button>
        {/each}
        
        {#if filtered.length === 0}
          <div class="p-4 text-center text-xs" style="color: var(--theme-text-muted);">No existing groups match your search.</div>
        {/if}
      </div>
    </div>
  </div>
{/if}