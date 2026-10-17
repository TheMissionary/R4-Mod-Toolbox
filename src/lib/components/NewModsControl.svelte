<script lang="ts">
  import { Sparkles } from 'lucide-svelte';

  let {
    newModsHours = $bindable(6),
    showNewMods = $bindable(false),
    newModsCount = 0,
    onConfigChanged
  }: {
    newModsHours: number;
    showNewMods: boolean;
    newModsCount?: number;
    onConfigChanged?: () => void;
  } = $props();

  function toggleShow() {
    showNewMods = !showNewMods;
    if (onConfigChanged) onConfigChanged();
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const val = parseInt(target.value, 10);
    if (!isNaN(val)) {
      newModsHours = Math.max(1, Math.min(8760, val));
    } else {
      newModsHours = 6;
    }
    target.value = newModsHours.toString();
    if (onConfigChanged) onConfigChanged();
  }
</script>

<div class="flex items-center rounded border transition-colors overflow-hidden {showNewMods ? 'border-nvidia-accent/60 bg-nvidia-accent/10' : 'border-nvidia-border bg-nvidia-surface'}">
  <button
    type="button"
    onclick={toggleShow}
    class="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium transition cursor-pointer {showNewMods ? 'font-semibold' : 'text-nvidia-text-muted hover:text-nvidia-text-primary hover:bg-nvidia-card'}"
    style={showNewMods ? 'background-color: var(--theme-accent); color: var(--theme-accent-contrast);' : ''}
    title={showNewMods ? `Highlighting mods installed in the last ${newModsHours} hours. Click to turn off.` : `Click to highlight mods installed in the last ${newModsHours} hours`}
  >
    <Sparkles class="h-3.5 w-3.5 {showNewMods ? '' : 'text-nvidia-accent'}" style={showNewMods ? 'color: var(--theme-accent-contrast);' : ''} />
    <span>New</span>
    {#if newModsCount > 0}
      <span
        class="px-1.5 py-0.2 rounded-full text-[10px] font-bold"
        style={showNewMods ? 'background-color: rgba(0,0,0,0.25); color: var(--theme-accent-contrast);' : 'background-color: color-mix(in srgb, var(--theme-accent) 20%, transparent); color: var(--theme-accent);'}
      >
        {newModsCount}
      </span>
    {/if}
  </button>
  <div class="flex items-center border-l px-1.5 {showNewMods ? 'border-nvidia-accent/40 bg-nvidia-accent/15' : 'border-nvidia-border/70 bg-nvidia-card/40'}">
    <input
      type="number"
      min="1"
      max="8760"
      value={newModsHours}
      onchange={handleInput}
      class="w-7 py-0.5 text-center font-mono text-xs bg-transparent text-nvidia-text-primary focus:outline-none [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
      title="Threshold in hours"
    />
    <span class="text-[10px] font-mono text-nvidia-text-muted pr-0.5">h</span>
  </div>
</div>
