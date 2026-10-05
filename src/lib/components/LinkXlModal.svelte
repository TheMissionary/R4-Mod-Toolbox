<script lang="ts">
  import { tick } from 'svelte';
  import { X, Link2, Search, FileArchive } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    xlName = '',
    archives = [],
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    xlName: string;
    archives: { name: string, file_name: string }[];
    onConfirm: (fileName: string) => void;
    onCancel: () => void;
  } = $props();

  let searchQuery = $state('');
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);

  let filtered = $derived(
    archives.filter(a => a.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  $effect(() => {
    if (isOpen) {
      searchQuery = '';
      selectedIndex = 0;
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
      selectedIndex = (selectedIndex + 1) % filtered.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filtered.length) % filtered.length;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filtered.length > 0) {
        onConfirm(filtered[selectedIndex].file_name);
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
        <div class="flex items-center gap-2.5 min-w-0">
          <Link2 class="h-4 w-4 text-cyan-400 shrink-0" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider truncate">Link {xlName}</span>
        </div>
        <button
          type="button"
          onclick={() => { isOpen = false; onCancel(); }}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer shrink-0"
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
          oninput={() => selectedIndex = 0}
          placeholder="Search for parent archive..."
          class="w-full pl-9 pr-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-cyan-400"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        {#each filtered as arch, i}
          <button
            type="button"
            onclick={() => onConfirm(arch.file_name)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center gap-2 cursor-pointer {i === selectedIndex ? 'bg-cyan-500/15 text-cyan-400 border border-cyan-500/30' : 'text-nvidia-text-primary hover:bg-nvidia-surface border border-transparent'}"
          >
            <FileArchive class="h-3.5 w-3.5 {i === selectedIndex ? 'text-cyan-400' : 'text-nvidia-text-muted'}" />
            <span class="truncate">{arch.name}</span>
          </button>
        {/each}
        {#if filtered.length === 0}
          <div class="p-4 text-center text-xs text-nvidia-text-muted">No archives found.</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
