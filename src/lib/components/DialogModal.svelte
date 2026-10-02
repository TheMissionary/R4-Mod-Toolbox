<script lang="ts">
  import { tick } from 'svelte';
  import { X, AlertTriangle, FolderPlus, Pencil, Check } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    title = 'RED4 Mod Toolbox',
    message = '',
    mode = 'prompt', // 'prompt' | 'confirm'
    initialValue = '',
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    isDanger = false,
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    title?: string;
    message?: string;
    mode?: 'prompt' | 'confirm';
    initialValue?: string;
    confirmText?: string;
    cancelText?: string;
    isDanger?: boolean;
    onConfirm: (val: string) => void;
    onCancel?: () => void;
  } = $props();

  let inputValue = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      inputValue = initialValue;
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
    if (onCancel) onCancel();
  }

  function handleSubmit() {
    if (mode === 'prompt' && inputValue.trim() === '') return;
    onConfirm(inputValue);
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
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40 shrink-0">
        <div class="flex items-center gap-2.5">
          {#if isDanger}
            <AlertTriangle class="h-4 w-4 text-red-400" />
          {:else if mode === 'prompt' && title.includes('Rename')}
            <Pencil class="h-4 w-4 text-nvidia-accent" />
          {:else}
            <FolderPlus class="h-4 w-4 text-nvidia-accent" />
          {/if}
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">
            {title}
          </span>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-3.5">
        {#if message}
          <p class="text-xs text-nvidia-text-muted leading-relaxed">
            {message}
          </p>
        {/if}

        {#if mode === 'prompt'}
          <div>
            <input
              bind:this={inputEl}
              type="text"
              bind:value={inputValue}
              class="w-full px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-nvidia-accent font-sans"
            />
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-nvidia-border bg-nvidia-surface/40 flex items-center justify-end gap-2.5 shrink-0">
        <button
          type="button"
          onclick={handleClose}
          class="px-3.5 py-1.5 rounded hover:bg-nvidia-surface text-xs text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
        >
          {cancelText}
        </button>

        <button
          type="button"
          onclick={handleSubmit}
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer {isDanger ? 'bg-red-600 hover:bg-red-500 text-white' : 'bg-nvidia-accent hover:brightness-105 text-black'}"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>{confirmText}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
