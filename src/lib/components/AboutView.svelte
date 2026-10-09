<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import {
    Info,
    Code,
    Coffee,
    ExternalLink,
    Cpu,
    Layers,
    ShieldCheck,
    Heart
  } from 'lucide-svelte';

  async function handleOpenLink(url: string) {
    try {
      await openUrl(url);
    } catch (err) {
      console.error('Failed to open URL:', err);
    }
  }
</script>

<div class="flex flex-col h-full overflow-hidden bg-nvidia-bg text-nvidia-text-primary select-none">
  <!-- Header -->
  <div class="p-6 pb-4 shrink-0 border-b border-nvidia-border/50 bg-nvidia-surface/20">
    <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
      <Info class="h-4 w-4" />
      <span>ABOUT R4</span>
    </div>
    <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">R4 Mod Toolbox</h2>
    <p class="text-xs text-nvidia-text-muted max-w-2xl">
      Advanced Mod Manager and Conflict Resolution Engine for Cyberpunk 2077.
    </p>
  </div>

  <!-- Scrollable Content -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-5xl">
      
      <!-- Application Info Card -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <ShieldCheck class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Application Info</h3>
        </div>
        <div class="p-5 space-y-4 flex-1">
          <div class="flex items-center gap-4">
            <div class="h-14 w-14 rounded-xl bg-nvidia-accent flex items-center justify-center text-black font-black text-2xl shadow-lg">
              R4
            </div>
            <div>
              <div class="text-lg font-bold text-nvidia-text-primary">R4 Mod Toolbox</div>
              <div class="text-xs font-mono text-nvidia-text-muted mt-0.5">Version 0.1.0 (Beta)</div>
            </div>
          </div>
          <div class="text-xs text-nvidia-text-muted leading-relaxed pt-2 border-t border-nvidia-border/50">
            Engineered to provide deterministic load order management, real-time archive conflict resolution, and seamless integration across all major Cyberpunk 2077 modding frameworks.
          </div>
        </div>
      </div>

      <!-- Developer Card -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Heart class="h-4 w-4 text-amber-500" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Developer</h3>
        </div>
        <div class="p-5 space-y-4 flex-1 flex flex-col justify-between">
          <div class="flex items-center gap-4">
            <!-- Stylized Hand-Drawn Heart Avatar -->
            <div class="h-14 w-14 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center shrink-0 shadow-inner">
              <svg viewBox="0 0 100 100" class="h-8 w-8 drop-shadow-md" fill="none" stroke="#e08528" stroke-width="8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
              </svg>
            </div>
            <div>
              <div class="text-lg font-bold text-nvidia-text-primary">Mi55ionary</div>
              <div class="text-xs font-mono text-nvidia-text-muted mt-0.5">Lead Architect & Developer</div>
            </div>
          </div>
          
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-4 border-t border-nvidia-border/50">
            <button
              type="button"
              onclick={() => handleOpenLink('https://www.nexusmods.com/profile/Mi55ionary')}
              class="flex items-center justify-center gap-2 px-3 py-2 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer group"
            >
              <ExternalLink class="h-3.5 w-3.5 text-amber-500 group-hover:brightness-110" />
              <span class="text-[11px] font-bold text-nvidia-text-primary">Nexus</span>
            </button>
            <button
              type="button"
              onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
              class="flex items-center justify-center gap-2 px-3 py-2 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer group"
            >
              <Code class="h-3.5 w-3.5 text-nvidia-text-muted group-hover:text-nvidia-text-primary" />
              <span class="text-[11px] font-bold text-nvidia-text-primary">GitHub</span>
            </button>
            <button
              type="button"
              onclick={() => handleOpenLink('https://ko-fi.com/mi55ionary')}
              class="flex items-center justify-center gap-2 px-3 py-2 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer group"
            >
              <Coffee class="h-3.5 w-3.5 text-cyan-400 group-hover:brightness-110" />
              <span class="text-[11px] font-bold text-nvidia-text-primary">Ko-fi</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Technology Stack -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col lg:col-span-2">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Layers class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Core Technologies</h3>
        </div>
        <div class="p-5 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
          
          <div class="p-3 rounded-lg bg-nvidia-card border border-nvidia-border flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <Cpu class="h-4 w-4 text-nvidia-accent" />
              <span class="text-xs font-bold text-nvidia-text-primary">Rust & Tauri</span>
            </div>
            <span class="text-[10px] text-nvidia-text-muted leading-tight">High-performance systems backend and native desktop runtime.</span>
          </div>

          <div class="p-3 rounded-lg bg-nvidia-card border border-nvidia-border flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <Code class="h-4 w-4 text-amber-500" />
              <span class="text-xs font-bold text-nvidia-text-primary">Svelte 5</span>
            </div>
            <span class="text-[10px] text-nvidia-text-muted leading-tight">Reactive frontend framework utilizing fine-grained runes.</span>
          </div>

          <div class="p-3 rounded-lg bg-nvidia-card border border-nvidia-border flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <Layers class="h-4 w-4 text-cyan-400" />
              <span class="text-xs font-bold text-nvidia-text-primary">Tailwind CSS v4</span>
            </div>
            <span class="text-[10px] text-nvidia-text-muted leading-tight">Utility-first styling engine powering the generative theme system.</span>
          </div>

          <div class="p-3 rounded-lg bg-nvidia-card border border-nvidia-border flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <ShieldCheck class="h-4 w-4 text-nvidia-text-muted" />
              <span class="text-xs font-bold text-nvidia-text-primary">red4lib</span>
            </div>
            <span class="text-[10px] text-nvidia-text-muted leading-tight">Advanced REDengine archive parsing and hash extraction library.</span>
          </div>

        </div>
      </div>

    </div>
  </div>
</div>
