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
      class="w-full max-w-md rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40">
        <div class="flex items-center gap-2.5">
          <FolderInput class="h-4 w-4 text-nvidia-accent" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Move to Folder</span>
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
          oninput={() => selectedIndex = -2}
          placeholder="Search folders..."
          class="w-full pl-9 pr-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        <button
          type="button"
          onclick={() => onConfirm(`[CREATE_NEW]:${searchQuery}`)}
          class="w-full text-left px-3 py-2 rounded text-xs font-bold transition flex items-center gap-2 cursor-pointer text-nvidia-accent border {selectedIndex === -1 ? 'bg-nvidia-accent/15 border-nvidia-accent/30' : 'border-transparent hover:bg-nvidia-accent/10'}"
        >
          <FolderPlus class="h-4 w-4" />
          {searchQuery ? `+ Create "${searchQuery}" Folder...` : '+ Create New Folder...'}
        </button>

        {#if filtered.length > 0}
          <div class="h-px bg-nvidia-border/50 my-1.5 mx-1"></div>
        {/if}

        {#each filtered as cat, i}
          <button
            type="button"
            onclick={() => onConfirm(cat.file_name)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center gap-2 cursor-pointer text-nvidia-text-primary border {i === selectedIndex ? 'bg-nvidia-surface border-nvidia-border shadow-sm' : 'border-transparent hover:bg-nvidia-surface/60'}"
          >
            <FolderInput class="h-3.5 w-3.5 text-nvidia-text-muted" />
            {cat.name}
          </button>
        {/each}
        
        {#if filtered.length === 0}
          <div class="p-4 text-center text-xs text-nvidia-text-muted">No existing folders match your search.</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
