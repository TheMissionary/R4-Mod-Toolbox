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
      class="w-full max-w-lg rounded-xl border border-nvidia-border bg-nvidia-card shadow-2xl overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="px-5 py-3.5 border-b border-nvidia-border flex items-center justify-between bg-nvidia-surface/40 shrink-0">
        <div class="flex items-center gap-2.5 min-w-0">
          <MessageSquare class="h-4 w-4 text-amber-400 shrink-0" />
          <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider truncate">
            Note: {modName}
          </span>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-1 rounded text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-surface transition cursor-pointer shrink-0"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="p-5">
        <textarea
          bind:this={textareaEl}
          bind:value={noteText}
          oncontextmenu={(e) => e.stopPropagation()}
          placeholder="Enter personal notes, load order instructions, or add.to.game codes here..."
          class="w-full h-40 px-3 py-2 bg-nvidia-surface border border-nvidia-border rounded text-xs text-nvidia-text-primary placeholder:text-nvidia-text-muted/60 focus:outline-none focus:border-amber-400 font-sans resize-none"
        ></textarea>
        <div class="mt-2 text-[10px] text-nvidia-text-muted flex justify-end">
          Press Ctrl+Enter to save
        </div>
      </div>

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
          class="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-semibold transition shadow-sm cursor-pointer bg-amber-500 hover:bg-amber-400 text-black"
        >
          <Check class="h-3.5 w-3.5 stroke-[2.5]" />
          <span>Save Note</span>
        </button>
      </div>
    </div>
  </div>
{/if}