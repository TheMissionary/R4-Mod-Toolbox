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
    Heart,
    Download,
    History,
    ChevronDown,
    ChevronRight,
    Star
  } from 'lucide-svelte';

  let isKofiGlowing = $state(false);
  let showChangelog = $state(false);
  let showCoreTech = $state(false);
  let glowTimeout: ReturnType<typeof setTimeout>;

  const changelog = [
    {
      version: "0.42.0 (RC1)",
      date: "October 2026",
      changes: [
        "Organizer View & The Stash: Introduced a virtual categorization layer decoupled from physical load order, featuring 'The Stash' drawer for temporarily shelving categories and loose mods without deletion.",
        "Draggable Categories & Live Mod Counters: Custom pointer-based drag-and-drop allows moving categories directly into The Stash and back into the active list, with live mod count badges (X mods).",
        "Load Order Category Tagging: Added inline right-click 'Add Category Label' and 'Edit Category Label' actions directly from the main Load Order view with multi-select (Shift/Ctrl) batch tagging.",
        "Full-Spectrum Color Picker: Integrated a native color wheel directly into category creation and editing dialogs, eliminating the need for external hex tools.",
        "Real-Time Organizer Search: Wired the search bar directly into the Organizer View, auto-expanding categories with matches, highlighting matching mods, and dimming non-matches.",
        "Universal History Routing: Added 'Show in History' across all framework tabs, automatically routing users to the mod ledger with all-time context.",
        "Antivirus Compliance & Verifiable Cloud Builds: Excised all background process commands, implemented clean OS-native handlers, and integrated GitHub Actions CI/CD for public build provenance (0/62 clean MSI scan).",
        "Special Acknowledgement: Dedicated card in About R4 honoring community collaborator rapsodyas for their essential feedback and design insights throughout the Organizer View's development."
      ]
    },
    {
      version: "0.41.0 (Beta 6)",
      date: "October 2026",
      changes: [
        "Organizer View Foundation: Initial vertical stack prototype for decoupled mod categorization.",
        "Custom Pointer DnD Engine: High-performance 60fps drag-and-drop bypassing WebView2 HTML5 drag limitations.",
        "Universal History Routing: 'Show in History' context menu across framework tabs."
      ]
    },
    {
      version: "0.40.0 (Beta 5)",
      date: "October 2026",
      changes: [
        "Critical Fix: Duplicate Mod Crash: Resolved a fatal UI rendering halt when both active and .disabled versions of a mod existed.",
        "Antivirus Compliance (Clean Slate): Excised tasklist/taskkill monitoring to eliminate heuristic flags.",
        "Display Scaling Fix: Adjusted minimum window resolution to 1000x650 for high-scaling displays."
      ]
    }
  ];

  function watchVisibility(node: HTMLElement) {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        isKofiGlowing = true;
        clearTimeout(glowTimeout);
        glowTimeout = setTimeout(() => {
          isKofiGlowing = false;
        }, 3500);
      } else {
        isKofiGlowing = false;
      }
    }, { threshold: 0.1 });
    
    observer.observe(node);
    return {
      destroy() { observer.disconnect(); }
    };
  }

  async function handleOpenLink(url: string) {
    try {
      await openUrl(url);
    } catch (err) {
      console.error('Failed to open URL:', err);
    }
  }
</script>

<div class="flex flex-col h-full overflow-hidden bg-nvidia-bg text-nvidia-text-primary select-none">
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

  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-5xl">
      
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <ShieldCheck class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Application Info</h3>
        </div>
        <div class="p-5 space-y-4 flex-1">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <div class="h-14 w-14 rounded-xl bg-nvidia-accent flex items-center justify-center text-black font-black text-2xl shadow-lg shrink-0">
                R4
              </div>
              <div>
                <div class="text-lg font-bold text-nvidia-text-primary">R4 Mod Toolbox</div>
                <div class="text-xs font-mono text-nvidia-text-muted mt-0.5">Version 0.42.0 (RC1)</div>
              </div>
            </div>
            
            <button
              type="button"
              onclick={() => handleOpenLink('https://www.nexusmods.com/cyberpunk2077/mods/34691')}
              class="flex items-center gap-2 px-3 py-2 rounded bg-nvidia-surface hover:bg-nvidia-card border border-nvidia-border transition cursor-pointer shrink-0 shadow-xs group"
            >
              <Download class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
              <span class="text-[11px] font-bold text-nvidia-text-primary">Check for Updates</span>
            </button>
          </div>
          <div class="text-xs text-nvidia-text-muted leading-relaxed pt-2 border-t border-nvidia-border/50">
            Engineered to provide deterministic load order management, real-time archive conflict resolution, and seamless integration across all major Cyberpunk 2077 modding frameworks.
          </div>
        </div>
      </div>

      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Heart class="h-4 w-4 text-amber-500" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Developer</h3>
        </div>
        <div class="p-5 space-y-4 flex-1 flex flex-col justify-between">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <div class="h-14 w-14 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center shrink-0 shadow-inner">
                <svg viewBox="0 0 100 100" class="h-8 w-8 drop-shadow-md" fill="none" stroke="#e08528" stroke-width="8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
                </svg>
              </div>
              <div>
                <div class="text-lg font-bold text-nvidia-text-primary">Mi55ionary</div>
                <div class="text-xs font-mono text-nvidia-text-muted mt-0.5">Developer</div>
              </div>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <button
                type="button"
                onclick={() => handleOpenLink('https://www.nexusmods.com/profile/Mi55ionary')}
                class="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer group"
                title="Nexus Mods Profile"
              >
                <ExternalLink class="h-3.5 w-3.5 text-amber-500 group-hover:brightness-110" />
                <span class="text-[10px] font-bold text-nvidia-text-primary">Nexus</span>
              </button>
              <button
                type="button"
                onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
                class="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer group"
                title="GitHub Repository"
              >
                <Code class="h-3.5 w-3.5 text-nvidia-text-muted group-hover:text-nvidia-text-primary" />
                <span class="text-[10px] font-bold text-nvidia-text-primary">GitHub</span>
              </button>
            </div>
          </div>
          
          <div class="pt-4 border-t border-nvidia-border/50">
            <button
              use:watchVisibility
              type="button"
              onclick={() => handleOpenLink('https://ko-fi.com/themissionary')}
              class="w-full flex items-center justify-center gap-2.5 px-4 py-2.5 rounded bg-nvidia-card hover:bg-cyan-500/10 border transition-all duration-700 cursor-pointer group {isKofiGlowing ? 'border-cyan-400 shadow-[0_0_15px_rgba(34,211,238,0.4)]' : 'border-nvidia-border hover:border-cyan-500/50'}"
            >
              <Coffee class="h-4 w-4 text-cyan-400 group-hover:brightness-110 {isKofiGlowing ? 'animate-pulse' : ''}" />
              <span class="text-xs font-bold text-nvidia-text-primary">Buy me a Coffee. Keep the Engine Running.</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Special Acknowledgements Section -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col lg:col-span-2">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Star class="h-4 w-4 text-amber-400" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Special Acknowledgements</h3>
        </div>
        <div class="p-5 space-y-5">
          
          <!-- rfuzzo tribute -->
          <div class="text-xs text-nvidia-text-muted leading-relaxed flex items-start gap-4">
            <div class="h-10 w-10 rounded-full bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 shadow-inner">
              <Code class="h-5 w-5 text-nvidia-text-muted" />
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <strong class="text-nvidia-text-primary text-sm">rfuzzo</strong>
                <span class="text-[10px] font-mono text-nvidia-accent border border-nvidia-accent/30 bg-nvidia-accent/10 px-1.5 py-0.2 rounded">Core Technology</span>
              </div>
              <p class="mb-2">
                A massive thank you to <strong class="text-nvidia-text-primary">rfuzzo</strong> for their foundational work on Wolvenkit and the creation of <span class="font-mono text-nvidia-text-primary">red4lib</span>. 
                Without their continuous innovation and dedication to the Cyberpunk 2077 modding ecosystem, the real-time archive conflict resolution engine powering R4 would not be possible.
              </p>
              <button
                type="button"
                onclick={() => handleOpenLink('https://github.com/rfuzzo')}
                class="flex items-center gap-1.5 text-nvidia-accent hover:brightness-110 transition cursor-pointer font-semibold"
              >
                <ExternalLink class="h-3 w-3" />
                <span>Visit rfuzzo on GitHub</span>
              </button>
            </div>
          </div>

          <!-- rapsodyas tribute -->
          <div class="pt-4 border-t border-nvidia-border/50 text-xs text-nvidia-text-muted leading-relaxed flex items-start gap-4">
            <div class="h-10 w-10 rounded-full bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 shadow-inner">
              <Heart class="h-5 w-5 text-amber-500" />
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <strong class="text-nvidia-text-primary text-sm">rapsodyas</strong>
                <span class="text-[10px] font-mono text-cyan-400 border border-cyan-500/30 bg-cyan-500/10 px-1.5 py-0.2 rounded">Community Testing & Design Collaborator</span>
              </div>
              <p class="mb-2">
                A sincere thank you to <strong class="text-nvidia-text-primary">rapsodyas</strong> on NexusMods for their dedicated testing and design insight throughout the beta cycle. Their perspective on managing sensitive load orders without breaking file priorities was the direct catalyst for R4’s <strong class="text-nvidia-text-primary">Organizer View</strong>, along with key refinements to inline label assignment, custom color selection, and search ergonomics.
              </p>
              <button
                type="button"
                onclick={() => handleOpenLink('https://www.nexusmods.com/profile/rapsodyas')}
                class="flex items-center gap-1.5 text-cyan-400 hover:brightness-110 transition cursor-pointer font-semibold"
              >
                <ExternalLink class="h-3 w-3" />
                <span>Visit rapsodyas on NexusMods</span>
              </button>
            </div>
          </div>

        </div>
      </div>

      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col lg:col-span-2">
        <div
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showCoreTech = !showCoreTech; }}
          onclick={() => showCoreTech = !showCoreTech}
          class="w-full px-4 py-3 bg-nvidia-surface/60 hover:bg-nvidia-surface/90 flex items-center justify-between cursor-pointer transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <Layers class="h-4 w-4 text-nvidia-accent" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Core Technologies</h3>
          </div>
          <div class="p-1 rounded hover:bg-nvidia-card text-nvidia-text-muted transition">
            {#if showCoreTech}
              <ChevronDown class="h-4 w-4" />
            {:else}
              <ChevronRight class="h-4 w-4" />
            {/if}
          </div>
        </div>

        {#if showCoreTech}
          <div class="p-5 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 border-t border-nvidia-border/60 bg-nvidia-bg/30">
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
        {/if}
      </div>

      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col lg:col-span-2">
        <div
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showChangelog = !showChangelog; }}
          onclick={() => showChangelog = !showChangelog}
          class="w-full px-4 py-3 bg-nvidia-surface/60 hover:bg-nvidia-surface/90 flex items-center justify-between cursor-pointer transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <History class="h-4 w-4 text-nvidia-accent" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Release Notes & Changelog</h3>
          </div>
          
          <div class="flex items-center gap-4">
            <button
              type="button"
              onclick={(e) => { e.stopPropagation(); handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/releases'); }}
              class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-nvidia-card hover:bg-nvidia-surface border border-nvidia-border transition cursor-pointer"
              title="View all releases on GitHub"
            >
              <span class="text-[10px] font-bold uppercase tracking-wider text-nvidia-text-primary">See All</span>
              <ExternalLink class="h-3 w-3 text-nvidia-text-muted" />
            </button>

            <div class="p-1 rounded hover:bg-nvidia-card text-nvidia-text-muted transition">
              {#if showChangelog}
                <ChevronDown class="h-4 w-4" />
              {:else}
                <ChevronRight class="h-4 w-4" />
              {/if}
            </div>
          </div>
        </div>

        {#if showChangelog}
          <div class="p-5 border-t border-nvidia-border/60 space-y-6 bg-nvidia-bg/30">
            {#each changelog as release}
              <div class="space-y-2">
                <div class="flex items-baseline gap-3">
                  <h4 class="text-sm font-bold text-nvidia-text-primary">{release.version}</h4>
                  <span class="text-[10px] font-mono text-nvidia-text-muted">— {release.date}</span>
                </div>
                <ul class="space-y-1.5 pl-1">
                  {#each release.changes as change}
                    <li class="text-xs text-nvidia-text-muted flex items-start gap-2">
                      <span class="text-nvidia-accent mt-0.5">•</span>
                      <span class="leading-relaxed">{change}</span>
                    </li>
                  {/each}
                </ul>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    </div>
  </div>
</div>