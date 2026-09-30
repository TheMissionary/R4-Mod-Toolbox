<script lang="ts">
  import type { ActiveView } from '$lib/types';
  import { Layers, Box, Cpu, AlertTriangle, ShieldCheck } from 'lucide-svelte';

  let { activeView = $bindable() }: { activeView: ActiveView } = $props();

  const navItems: { id: ActiveView; label: string; icon: any }[] = [
    { id: 'home', label: 'Home', icon: Layers },
    { id: 'archive', label: 'Archive', icon: Box },
    { id: 'cet', label: 'CET Mods', icon: Cpu },
    { id: 'red', label: 'Red Mods', icon: AlertTriangle },
  ];
</script>

<aside class="w-56 bg-nvidia-surface border-r border-nvidia-border flex flex-col justify-between select-none">
  <div>
    <!-- App Branding -->
    <div class="h-16 flex items-center gap-3 px-5 border-b border-nvidia-border">
      <div class="h-7 w-7 rounded bg-nvidia-accent/20 border border-nvidia-accent flex items-center justify-center">
        <div class="h-2.5 w-2.5 rounded-sm bg-nvidia-accent"></div>
      </div>
      <div>
        <h1 class="text-sm font-bold tracking-wider text-white uppercase">RED4 UTILITY</h1>
        <p class="text-[10px] text-nvidia-text-muted uppercase tracking-widest">CP2077 Toolset</p>
      </div>
    </div>

    <!-- Navigation Items -->
    <nav class="p-3 space-y-1">
      {#each navItems as item}
        {@const isActive = activeView === item.id}
        <button
          onclick={() => activeView = item.id}
          class="w-full flex items-center gap-3.5 px-3 py-2.5 rounded text-xs font-medium transition-all duration-150 relative {isActive ? 'bg-nvidia-card text-white font-semibold shadow-sm' : 'text-nvidia-text-muted hover:text-white hover:bg-nvidia-card/50'}"
        >
          {#if isActive}
            <div class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-r bg-nvidia-accent"></div>
          {/if}
          <item.icon class="h-4 w-4 {isActive ? 'text-nvidia-accent' : 'text-nvidia-text-muted'}" />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>
  </div>

  <!-- Footer Info / Link State -->
  <div class="p-4 border-t border-nvidia-border bg-nvidia-surface/80">
    <div class="flex items-center gap-2 text-[11px] text-nvidia-text-muted">
      <ShieldCheck class="h-3.5 w-3.5 text-nvidia-accent" />
      <span>Engine Monitor Active</span>
    </div>
  </div>
</aside>
