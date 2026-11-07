<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import {
    Info, Code, Coffee, ExternalLink, Cpu, Layers, ShieldCheck, Heart,
    Download, History, ChevronDown, ChevronRight, Star
  } from 'lucide-svelte';

  let isKofiGlowing = $state(false);
  let showChangelog = $state(false);
  let showCoreTech = $state(false);
  let glowTimeout: ReturnType<typeof setTimeout>;

  const changelog = [
    {
      version: "0.43.0",
      date: "October 2026",
      changes: [
        "IDE & File-Explorer Paradigm: Transitioned to an uncluttered workspace layout featuring a 240px Quick Access left rail, breadcrumb navigation header, and unified telemetry status bar.",
        "Restructured Telemetry Status Bar: Replaced legacy center text with live cursor hover path streaming on the left and right-justified system indicators ([Rescan] | [Active: X of Y] | [● Connected]) separated by clean vertical dividers.",
        "Full-Canvas Atmospheric Hero Wash: Extended custom background images across the entire workspace stage, layered with a protective multi-stop gradient wash to maintain strict WCAG AA (4.5:1) text contrast over data rows.",
        "Laser-Straight Vertical Symmetry: Locked grip handles and checkboxes to dedicated, fixed-width columns across all mod and group rows.",
        "IDE Tree Guide Wires: Nested child mods display continuous vertical guide lines and horizontal branches (├─), providing clear structural indentation under virtual groups without horizontal checkbox drift.",
        "Elevated Group Banners: Group separators feature subtle surface banding with an accent line marker and dedicated mod count pills (X mods).",
        "Clean Action Ergonomics: Moved Group Rename and Delete actions to the far-right edge, revealing cleanly on row hover only.",
        "High-Contrast Checkbox Fill: Solid accent fills with luminosity-adaptive contrast checkmarks across all framework tabs.",
        "New Base Atmospheres: Added 'Frost' (an icy Nordic slate neutral) and established 'Graphite' and 'Cloud' as the hardcoded defaults for Dark and Light modes respectively.",
        "Hover Ergonomics: Implemented a distinct 12% accent-tinted highlight when mousing over any mod row, clearing seamlessly on mouse exit.",
        "Tab-Exit Selection Clearing: Active multi-select highlights are now strictly cleared the moment a user navigates away from the active framework tab."
      ]
    },
    {
      version: "0.42.3 (RC1)",
      date: "October 2026",
      changes: [
        "Optimistic UI Toggling: Instant visual switch flipping across all framework tabs with debounced backend execution for a highly responsive feel.",
        "Personal Mod Notes: Added the ability to attach personal notes to any mod via the right-click context menu, complete with a dedicated UI modal and visual indicators.",
        "Organizer View Enhancements: Introduced a responsive Grid Layout (Span-Full) for categories, a Column Control dropdown (Auto, 1, 2, or 3 columns), and dynamic hover illumination for drag-and-drop targeting.",
        "Strict Search Decoupling: Search queries are now strictly isolated and automatically cleared when navigating between Load Order, Organizer, and framework tabs.",
        "Mod List Export: Added a new utility in the Settings tab to export the entire load order and framework mod lists to a formatted .txt file."
      ]
    },
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
        "Antivirus Compliance & Verifiable Cloud Builds: Excised all background process commands, implemented clean OS-native handlers, and integrated GitHub Actions CI/CD for public build provenance (0/62 clean MSI scan)."
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

<div class="flex flex-col h-full overflow-hidden select-none" style="background-color: var(--theme-bg); color: var(--theme-text-primary);">
  <div class="p-6 pb-4 shrink-0 border-b" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 20%, transparent);">
    <div class="flex items-center gap-2 text-xs font-mono mb-2" style="color: var(--theme-accent);">
      <Info class="h-4 w-4" />
      <span>ABOUT R4</span>
    </div>
    <h2 class="text-xl font-bold mb-1" style="color: var(--theme-text-primary);">R4 Mod Toolbox</h2>
    <p class="text-xs max-w-2xl" style="color: var(--theme-text-muted);">
      Advanced Mod Manager and Conflict Resolution Engine for Cyberpunk 2077.
    </p>
  </div>

  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-5xl">
      
      <!-- Application Info Card -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <div class="px-4 py-3 border-b flex items-center gap-2.5" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);">
          <ShieldCheck class="h-4 w-4" style="color: var(--theme-accent);" />
          <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Application Info</h3>
        </div>
        <div class="p-5 flex-1 flex items-center">
          <div class="flex items-center gap-4">
            <div class="h-14 w-14 rounded-xl flex items-center justify-center font-black text-2xl shadow-lg shrink-0" style="background-color: var(--theme-accent); color: var(--theme-accent-contrast);">
              R4
            </div>
            <div>
              <div class="text-lg font-bold" style="color: var(--theme-text-primary);">R4 Mod Toolbox</div>
              <div class="text-xs font-mono mt-0.5" style="color: var(--theme-text-muted);">Version 0.43.0</div>
            </div>
          </div>
        </div>
        <!-- Dual Update Action Bar -->
        <div class="grid grid-cols-2 border-t divide-x" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 30%, transparent);">
          <button
            type="button"
            onclick={() => handleOpenLink('https://www.nexusmods.com/cyberpunk2077/mods/34691')}
            class="flex items-center justify-center gap-2 py-3 transition group cursor-pointer hover:bg-[var(--theme-surface)]"
          >
            <Download class="h-4 w-4 group-hover:brightness-110" style="color: var(--theme-accent);" />
            <span class="text-xs font-bold" style="color: var(--theme-text-primary);">Update via Nexus</span>
          </button>
          <button
            type="button"
            onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/releases')}
            class="flex items-center justify-center gap-2 py-3 transition group cursor-pointer hover:bg-[var(--theme-surface)]"
          >
            <Code class="h-4 w-4 group-hover:text-[var(--theme-text-primary)]" style="color: var(--theme-text-muted);" />
            <span class="text-xs font-bold" style="color: var(--theme-text-primary);">Update via GitHub</span>
          </button>
        </div>
      </div>

      <!-- Developer Card -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <div class="px-4 py-3 border-b flex items-center gap-2.5" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);">
          <Heart class="h-4 w-4" style="color: var(--color-warning);" />
          <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Developer</h3>
        </div>
        <div class="p-5 flex-1 flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="h-14 w-14 rounded-xl border flex items-center justify-center shrink-0 shadow-inner" style="background-color: color-mix(in srgb, var(--color-warning) 10%, transparent); border-color: color-mix(in srgb, var(--color-warning) 30%, transparent);">
              <svg viewBox="0 0 100 100" class="h-8 w-8 drop-shadow-md" fill="none" stroke="currentColor" style="color: var(--color-warning);" stroke-width="8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
              </svg>
            </div>
            <div>
              <div class="text-lg font-bold" style="color: var(--theme-text-primary);">Mi55ionary</div>
              <div class="text-xs font-mono mt-0.5" style="color: var(--theme-text-muted);">Developer</div>
            </div>
          </div>
          <div class="flex flex-col gap-2 shrink-0">
            <button
              type="button"
              onclick={() => handleOpenLink('https://www.nexusmods.com/profile/Mi55ionary')}
              class="flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-full border transition cursor-pointer group hover:bg-[var(--theme-surface)]"
              style="background-color: var(--theme-card); border-color: var(--theme-border);"
            >
              <ExternalLink class="h-3 w-3 group-hover:brightness-110" style="color: var(--color-warning);" />
              <span class="text-[10px] font-bold" style="color: var(--theme-text-primary);">Nexus Profile</span>
            </button>
            <button
              type="button"
              onclick={() => handleOpenLink('https://github.com/TheMissionary')}
              class="flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-full border transition cursor-pointer group hover:bg-[var(--theme-surface)]"
              style="background-color: var(--theme-card); border-color: var(--theme-border);"
            >
              <Code class="h-3 w-3 group-hover:text-[var(--theme-text-primary)]" style="color: var(--theme-text-muted);" />
              <span class="text-[10px] font-bold" style="color: var(--theme-text-primary);">GitHub Profile</span>
            </button>
          </div>
        </div>
        <!-- 3-Column Support Action Bar -->
        <div class="grid grid-cols-3 border-t divide-x" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 30%, transparent);">
          <button
            type="button"
            onclick={() => handleOpenLink('https://www.nexusmods.com/cyberpunk2077/mods/34691')}
            class="flex flex-col items-center justify-center gap-1.5 py-2.5 transition group cursor-pointer hover:bg-[var(--theme-surface)]"
          >
            <Heart class="h-4 w-4 group-hover:scale-110 transition-transform" style="color: var(--color-warning);" />
            <span class="text-[10px] font-bold" style="color: var(--theme-text-primary);">Endorse on Nexus</span>
          </button>
          <button
            type="button"
            onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
            class="flex flex-col items-center justify-center gap-1.5 py-2.5 transition group cursor-pointer hover:bg-[var(--theme-surface)]"
          >
            <Star class="h-4 w-4 group-hover:scale-110 transition-transform" style="color: var(--color-warning);" />
            <span class="text-[10px] font-bold" style="color: var(--theme-text-primary);">Star on GitHub</span>
          </button>
          <button
            use:watchVisibility
            type="button"
            onclick={() => handleOpenLink('https://ko-fi.com/themissionary')}
            class="flex flex-col items-center justify-center gap-1.5 py-2.5 transition-all duration-700 group cursor-pointer hover:bg-[color-mix(in_srgb,var(--color-info)_10%,transparent)]"
            style="{isKofiGlowing ? 'background-color: color-mix(in srgb, var(--color-info) 5%, transparent); box-shadow: inset 0 0 15px color-mix(in srgb, var(--color-info) 20%, transparent);' : ''}"
          >
            <Coffee class="h-4 w-4 group-hover:scale-110 transition-transform {isKofiGlowing ? 'animate-pulse' : ''}" style="color: var(--color-info);" />
            <span class="text-[10px] font-bold" style="color: var(--theme-text-primary);">Buy me a Coffee</span>
          </button>
        </div>
      </div>

      <!-- Special Acknowledgements Section -->
      <div class="rounded-xl border overflow-hidden flex flex-col lg:col-span-2" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <div class="px-4 py-3 border-b flex items-center gap-2.5" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);">
          <Star class="h-4 w-4" style="color: var(--color-warning);" />
          <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Special Acknowledgements</h3>
        </div>
        <div class="p-5 space-y-5">
          
          <!-- rfuzzo tribute -->
          <div class="text-xs leading-relaxed flex items-start gap-4" style="color: var(--theme-text-muted);">
            <div class="h-10 w-10 rounded-full border flex items-center justify-center shrink-0 shadow-inner" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <Code class="h-5 w-5" style="color: var(--theme-text-muted);" />
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <strong class="text-sm" style="color: var(--theme-text-primary);">rfuzzo</strong>
                <span class="text-[10px] font-mono border px-1.5 py-0.2 rounded" style="color: var(--theme-accent); border-color: color-mix(in srgb, var(--theme-accent) 30%, transparent); background-color: color-mix(in srgb, var(--theme-accent) 10%, transparent);">Core Technology</span>
              </div>
              <p class="mb-2">
                A massive thank you to <strong style="color: var(--theme-text-primary);">rfuzzo</strong> for their foundational work on Wolvenkit and the creation of <span class="font-mono" style="color: var(--theme-text-primary);">red4lib</span>. 
                Without their continuous innovation and dedication to the Cyberpunk 2077 modding ecosystem, the real-time archive conflict resolution engine powering R4 would not be possible.
              </p>
              <button
                type="button"
                onclick={() => handleOpenLink('https://github.com/rfuzzo')}
                class="flex items-center gap-1.5 hover:brightness-110 transition cursor-pointer font-semibold"
                style="color: var(--theme-accent);"
              >
                <ExternalLink class="h-3 w-3" />
                <span>Visit rfuzzo on GitHub</span>
              </button>
            </div>
          </div>

          <!-- rapsodyas tribute -->
          <div class="pt-4 border-t text-xs leading-relaxed flex items-start gap-4" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent); color: var(--theme-text-muted);">
            <div class="h-10 w-10 rounded-full border flex items-center justify-center shrink-0 shadow-inner" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <Heart class="h-5 w-5" style="color: var(--color-warning);" />
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <strong class="text-sm" style="color: var(--theme-text-primary);">rapsodyas</strong>
                <span class="text-[10px] font-mono border px-1.5 py-0.2 rounded" style="color: var(--color-info); border-color: color-mix(in srgb, var(--color-info) 30%, transparent); background-color: color-mix(in srgb, var(--color-info) 10%, transparent);">Community Testing & Design Collaborator</span>
              </div>
              <p class="mb-2">
                A sincere thank you to <strong style="color: var(--theme-text-primary);">rapsodyas</strong> on NexusMods for their dedicated testing and design insight throughout the beta cycle. Their perspective on managing sensitive load orders without breaking file priorities was the direct catalyst for R4’s <strong style="color: var(--theme-text-primary);">Organizer View</strong>, along with key refinements to inline label assignment, custom color selection, and search ergonomics.
              </p>
              <button
                type="button"
                onclick={() => handleOpenLink('https://www.nexusmods.com/profile/rapsodyas')}
                class="flex items-center gap-1.5 hover:brightness-110 transition cursor-pointer font-semibold"
                style="color: var(--color-info);"
              >
                <ExternalLink class="h-3 w-3" />
                <span>Visit rapsodyas on NexusMods</span>
              </button>
            </div>
          </div>

        </div>
      </div>

      <div class="rounded-xl border overflow-hidden flex flex-col lg:col-span-2" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <div
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showCoreTech = !showCoreTech; }}
          onclick={() => showCoreTech = !showCoreTech}
          class="w-full px-4 py-3 flex items-center justify-between cursor-pointer transition-colors hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <Layers class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Core Technologies</h3>
          </div>
          <div class="p-1 rounded transition hover:bg-[var(--theme-card)]" style="color: var(--theme-text-muted);">
            {#if showCoreTech}
              <ChevronDown class="h-4 w-4" />
            {:else}
              <ChevronRight class="h-4 w-4" />
            {/if}
          </div>
        </div>

        {#if showCoreTech}
          <div class="p-5 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-3 rounded-lg border flex flex-col gap-1.5" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <div class="flex items-center gap-2">
                <Cpu class="h-4 w-4" style="color: var(--theme-accent);" />
                <span class="text-xs font-bold" style="color: var(--theme-text-primary);">Rust & Tauri</span>
              </div>
              <span class="text-[10px] leading-tight" style="color: var(--theme-text-muted);">High-performance systems backend and native desktop runtime.</span>
            </div>
            <div class="p-3 rounded-lg border flex flex-col gap-1.5" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <div class="flex items-center gap-2">
                <Code class="h-4 w-4" style="color: var(--color-warning);" />
                <span class="text-xs font-bold" style="color: var(--theme-text-primary);">Svelte 5</span>
              </div>
              <span class="text-[10px] leading-tight" style="color: var(--theme-text-muted);">Reactive frontend framework utilizing fine-grained runes.</span>
            </div>
            <div class="p-3 rounded-lg border flex flex-col gap-1.5" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <div class="flex items-center gap-2">
                <Layers class="h-4 w-4" style="color: var(--color-info);" />
                <span class="text-xs font-bold" style="color: var(--theme-text-primary);">Tailwind CSS v4</span>
              </div>
              <span class="text-[10px] leading-tight" style="color: var(--theme-text-muted);">Utility-first styling engine powering the generative theme system.</span>
            </div>
            <div class="p-3 rounded-lg border flex flex-col gap-1.5" style="background-color: var(--theme-card); border-color: var(--theme-border);">
              <div class="flex items-center gap-2">
                <ShieldCheck class="h-4 w-4" style="color: var(--theme-text-muted);" />
                <span class="text-xs font-bold" style="color: var(--theme-text-primary);">red4lib</span>
              </div>
              <span class="text-[10px] leading-tight" style="color: var(--theme-text-muted);">Advanced REDengine archive parsing and hash extraction library.</span>
            </div>
          </div>
        {/if}
      </div>

      <div class="rounded-xl border overflow-hidden flex flex-col lg:col-span-2" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <div
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showChangelog = !showChangelog; }}
          onclick={() => showChangelog = !showChangelog}
          class="w-full px-4 py-3 flex items-center justify-between cursor-pointer transition-colors hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <History class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">Release Notes & Changelog</h3>
          </div>
          
          <div class="flex items-center gap-4">
            <button
              type="button"
              onclick={(e) => { e.stopPropagation(); handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/releases'); }}
              class="flex items-center gap-1.5 px-2.5 py-1 rounded border transition cursor-pointer hover:bg-[var(--theme-surface)]"
              style="background-color: var(--theme-card); border-color: var(--theme-border);"
              title="View all releases on GitHub"
            >
              <span class="text-[10px] font-bold uppercase tracking-wider" style="color: var(--theme-text-primary);">See All</span>
              <ExternalLink class="h-3 w-3" style="color: var(--theme-text-muted);" />
            </button>

            <div class="p-1 rounded transition hover:bg-[var(--theme-card)]" style="color: var(--theme-text-muted);">
              {#if showChangelog}
                <ChevronDown class="h-4 w-4" />
              {:else}
                <ChevronRight class="h-4 w-4" />
              {/if}
            </div>
          </div>
        </div>

        {#if showChangelog}
          <div class="p-5 border-t space-y-6" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            {#each changelog as release}
              <div class="space-y-2">
                <div class="flex items-baseline gap-3">
                  <h4 class="text-sm font-bold" style="color: var(--theme-text-primary);">{release.version}</h4>
                  <span class="text-[10px] font-mono" style="color: var(--theme-text-muted);">— {release.date}</span>
                </div>
                <ul class="space-y-1.5 pl-1">
                  {#each release.changes as change}
                    <li class="text-xs flex items-start gap-2" style="color: var(--theme-text-muted);">
                      <span class="mt-0.5" style="color: var(--theme-accent);">•</span>
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