<script lang="ts">
  import { tick } from 'svelte';
  import { X, Palette, Check, FolderPlus, Pencil } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    mode = 'create',
    initialData = { id: '', name: '', color: '#06B6D4' },
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    mode: 'create' | 'edit';
    initialData: { id: string; name: string; color: string };
    onConfirm: (data: { id: string; name: string; color: string }) => void;
    onCancel: () => void;
  } = $props();

  let name = $state('');
  let color = $state('#06B6D4');
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      name = initialData.name;
      color = initialData.color || '#06B6D4';
      tick().then(() => {
        if (inputEl) {
          inputEl.focus();
          inputEl.select();
        }
      });
    }
  });

  function handleClose() {
    isOpen = false;
    onCancel();
  }

  function handleSubmit() {
    if (name.trim() === '') return;
    onConfirm({
      id: initialData.id || crypto.randomUUID(),
      name: name.trim(),
      color: color.toUpperCase()
    });
    isOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleManualHex(e: Event) {
    let val = (e.target as HTMLInputElement).value.trim();
    if (!val.startsWith('#')) val = '#' + val;
    color = val.toUpperCase();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={handleClose}
    role="presentation"
  >
    <div
      class="w-full max-w-md rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b flex items-center justify-between shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5">
          {#if mode === 'create'}
            <FolderPlus class="h-4 w-4" style="color: var(--color-info);" />
            <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Create Category</span>
          {:else}
            <Pencil class="h-4 w-4" style="color: var(--color-info);" />
            <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Edit Category</span>
          {/if}
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-1 rounded transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          <X class="h-4 w-4 hover:text-[var(--theme-text-primary)]" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 space-y-5">
        <div class="space-y-2">
          <label class="text-xs font-semibold" style="color: var(--theme-text-primary);">Category Name</label>
          <input
            bind:this={inputEl}
            type="text"
            bind:value={name}
            placeholder="e.g., Weapons, Vehicles, UI..."
            class="w-full px-3 py-2 border rounded text-xs focus:outline-none font-sans"
            style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
          />
        </div>

        <div class="space-y-2.5">
          <div class="flex items-center gap-2 text-xs font-semibold" style="color: var(--theme-text-primary);">
            <Palette class="h-3.5 w-3.5" style="color: var(--theme-text-muted);" />
            <span>Label Color</span>
          </div>

          <div class="flex items-center gap-3 pt-1">
            <div class="relative h-9 w-14 rounded-lg border overflow-hidden shrink-0 shadow-sm transition cursor-pointer" style="border-color: var(--theme-border);">
              <input
                type="color"
                bind:value={color}
                class="absolute -inset-4 h-16 w-20 cursor-pointer bg-transparent"
                title="Click to open color palette"
              />
            </div>

            <input
              type="text"
              bind:value={color}
              oninput={handleManualHex}
              placeholder="#06B6D4"
              maxlength="7"
              class="w-28 px-3 py-2 border rounded text-xs font-mono focus:outline-none uppercase text-center font-bold tracking-wider"
              style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
            />

            <div class="flex-1 flex items-center justify-end">
              <div 
                class="px-3 py-1 rounded text-[11px] font-bold uppercase tracking-wider border shadow-xs"
                style="background-color: color-mix(in srgb, {color} 15%, transparent); color: {color}; border-color: color-mix(in srgb, {color} 40%, transparent);"
              >
                Preview Tag
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t flex items-center justify-end gap-2.5 shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <button
          type="button"
          onclick={handleClose}
          class="px-3.5 py-1.5 rounded text-xs transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={handleSubmit}
          disabled={name.trim() === ''}
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:brightness-110"
          style="background-color: var(--color-info); color: #000000;"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>{mode === 'create' ? 'Create' : 'Save Changes'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}