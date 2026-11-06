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
      class="w-full max-w-md rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b flex items-center justify-between" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5 min-w-0">
          <Link2 class="h-4 w-4 shrink-0" style="color: var(--color-info);" />
          <span class="text-xs font-bold uppercase tracking-wider truncate" style="color: var(--theme-text-primary);">Link {xlName}</span>
        </div>
        <button
          type="button"
          onclick={() => { isOpen = false; onCancel(); }}
          class="p-1 rounded transition cursor-pointer shrink-0 hover:bg-[var(--theme-bg)]"
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
          oninput={() => selectedIndex = 0}
          placeholder="Search for parent archive..."
          class="w-full pl-9 pr-3 py-2 border rounded text-xs focus:outline-none"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        />
      </div>

      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        {#each filtered as arch, i}
          <button
            type="button"
            onclick={() => onConfirm(arch.file_name)}
            class="w-full text-left px-3 py-2 rounded text-xs font-medium transition flex items-center gap-2 cursor-pointer border"
            style="color: {i === selectedIndex ? 'var(--color-info)' : 'var(--theme-text-primary)'}; border-color: {i === selectedIndex ? 'color-mix(in srgb, var(--color-info) 30%, transparent)' : 'transparent'}; background-color: {i === selectedIndex ? 'color-mix(in srgb, var(--color-info) 15%, transparent)' : 'transparent'};"
          >
            <FileArchive class="h-3.5 w-3.5" style="color: {i === selectedIndex ? 'var(--color-info)' : 'var(--theme-text-muted)'};" />
            <span class="truncate">{arch.name}</span>
          </button>
        {/each}
        {#if filtered.length === 0}
          <div class="p-4 text-center text-xs" style="color: var(--theme-text-muted);">No archives found.</div>
        {/if}
      </div>
    </div>
  </div>
{/if}