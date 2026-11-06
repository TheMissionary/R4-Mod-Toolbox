<script lang="ts">
  import { tick } from 'svelte';
  import { X, MessageSquare, Check } from 'lucide-svelte';

  let {
    isOpen = $bindable(false),
    modName = '',
    initialNote = '',
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    modName: string;
    initialNote: string;
    onConfirm: (note: string) => void;
    onCancel: () => void;
  } = $props();

  let noteText = $state('');
  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  $effect(() => {
    if (isOpen) {
      noteText = initialNote;
      tick().then(() => {
        if (textareaEl) {
          textareaEl.focus();
          textareaEl.setSelectionRange(textareaEl.value.length, textareaEl.value.length);
        }
      });
    }
  });

  function handleClose() {
    isOpen = false;
    onCancel();
  }

  function handleSubmit() {
    onConfirm(noteText.trim());
    isOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
    } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
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
      class="w-full max-w-lg rounded-xl border shadow-2xl overflow-hidden flex flex-col"
      style="background-color: var(--theme-card); border-color: var(--theme-border);"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b flex items-center justify-between shrink-0" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
        <div class="flex items-center gap-2.5 min-w-0">
          <MessageSquare class="h-4 w-4 shrink-0" style="color: var(--color-warning);" />
          <span class="text-xs font-bold uppercase tracking-wider truncate" style="color: var(--theme-text-primary);">
            Note: {modName}
          </span>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-1 rounded transition cursor-pointer shrink-0 hover:bg-[var(--theme-bg)]"
          style="color: var(--theme-text-muted);"
        >
          <X class="h-4 w-4 hover:text-[var(--theme-text-primary)]" />
        </button>
      </div>

      <div class="p-5">
        <textarea
          bind:this={textareaEl}
          bind:value={noteText}
          oncontextmenu={(e) => e.stopPropagation()}
          placeholder="Enter personal notes, load order instructions, or add.to.game codes here..."
          class="w-full h-40 px-3 py-2 border rounded text-xs focus:outline-none font-sans resize-none"
          style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
        ></textarea>
        <div class="mt-2 text-[10px] flex justify-end" style="color: var(--theme-text-muted);">
          Press Ctrl+Enter to save
        </div>
      </div>

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
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer hover:brightness-110"
          style="background-color: var(--color-warning); color: #000000;"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>Save Note</span>
        </button>
      </div>
    </div>
  </div>
{/if}