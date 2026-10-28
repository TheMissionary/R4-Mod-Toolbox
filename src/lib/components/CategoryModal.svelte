<script lang="ts">
  import { tick } from 'svelte';
  import { X, Palette, Check, FolderPlus, Pencil } from 'lucide-svelte';
  import { ACCENT_PRESETS } from '$lib/theme';

  let {
    isOpen = $bindable(false),
    mode = 'create',
    initialData = { id: '', name: '', color: ACCENT_PRESETS[0].hex },
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
  let color = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      name = initialData.name;
      color = initialData.color || ACCENT_PRESETS[0].hex;
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
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    onclick={handleClose}
    role="presentation"
  >
    <div
      class="w-full max-w-md rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40 shrink-0">
        <div class="flex items-center gap-2.5">
          {#if mode === 'create'}
            <FolderPlus class="h-4 w-4 text-cyan-400" />
            <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Create Category</span>
          {:else}
            <Pencil class="h-4 w-4 text-cyan-400" />
            <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Edit Category</span>
          {/if}
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 space-y-5">
        <div class="space-y-2">
          <label class="text-xs font-semibold text-nvidia-text-primary">Category Name</label>
          <input
            bind:this={inputEl}
            type="text"
            bind:value={name}
            placeholder="e.g., Weapons, Vehicles, UI..."
            class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-cyan-400 font-sans"
          />
        </div>

        <div class="space-y-3">
          <div class="flex items-center gap-2 text-xs font-semibold text-nvidia-text-primary">
            <Palette class="h-3.5 w-3.5 text-nvidia-text-muted" />
            <span>Label Color</span>
          </div>
          
          <div class="flex flex-wrap gap-3">
            {#each ACCENT_PRESETS as preset}
              <button
                type="button"
                onclick={() => color = preset.hex}
                class="w-6 h-6 rounded-full border-2 transition-all cursor-pointer hover:scale-110 {color === preset.hex ? 'border-white shadow-[0_0_10px_rgba(255,255,255,0.3)] scale-110' : 'border-transparent shadow-sm'}"
                style="background-color: {preset.hex};"
                title={preset.label}
                aria-label="Select color {preset.label}"
              ></button>
            {/each}
          </div>

          <div class="flex items-center gap-3 pt-2">
            <span class="text-xs text-nvidia-text-muted">Custom Hex:</span>
            <input
              type="text"
              bind:value={color}
              placeholder="#FFFFFF"
              maxlength="7"
              class="w-24 px-2 py-1 bg-nvidia-surface border border-nvidia-border rounded text-xs font-mono text-nvidia-text-primary focus:outline-none focus:border-cyan-400 uppercase text-center"
            />
            <div class="h-6 w-6 rounded border border-nvidia-border shadow-inner shrink-0" style="background-color: {color};"></div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-nvidia-border bg-nvidia-surface/40 flex items-center justify-end gap-2.5 shrink-0">
        <button
          type="button"
          onclick={handleClose}
          class="px-3.5 py-1.5 rounded hover:bg-nvidia-surface text-xs text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={handleSubmit}
          disabled={name.trim() === ''}
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer bg-cyan-500 hover:bg-cyan-400 text-black disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>{mode === 'create' ? 'Create' : 'Save Changes'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}