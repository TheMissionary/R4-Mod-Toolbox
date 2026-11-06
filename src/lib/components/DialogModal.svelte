<script lang="ts">
  import { tick } from 'svelte';
  import { X, AlertTriangle, FolderPlus, Pencil, Check } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    title = 'RED4 Mod Toolbox',
    message = '',
    mode = 'prompt',
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
      class="w-full max-w-md rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b flex items-center justify-between shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5">
          {#if isDanger}
            <AlertTriangle class="h-4 w-4" style="color: var(--color-danger);" />
          {:else if mode === 'prompt' && title.includes('Rename')}
            <Pencil class="h-4 w-4" style="color: var(--theme-accent);" />
          {:else}
            <FolderPlus class="h-4 w-4" style="color: var(--theme-accent);" />
          {/if}
          <span class="text-xs font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">
            {title}
          </span>
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

      <!-- Modal Body -->
      <div class="p-5 space-y-3.5">
        {#if message}
          <p class="text-xs leading-relaxed" style="color: var(--theme-text-muted);">
            {message}
          </p>
        {/if}

        {#if mode === 'prompt'}
          <div>
            <input
              bind:this={inputEl}
              type="text"
              bind:value={inputValue}
              class="w-full px-3 py-2 border rounded text-xs focus:outline-none font-sans"
              style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
            />
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t flex items-center justify-end gap-2.5 shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <button
          type="button"
          onclick={handleClose}
          class="px-3.5 py-1.5 rounded text-xs transition cursor-pointer hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          {cancelText}
        </button>

        <button
          type="button"
          onclick={handleSubmit}
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer hover:brightness-110"
          style="{isDanger ? 'background-color: var(--color-danger); color: #ffffff;' : 'background-color: var(--theme-accent); color: var(--theme-accent-contrast);'}"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>{confirmText}</span>
        </button>
      </div>
    </div>
  </div>
{/if}