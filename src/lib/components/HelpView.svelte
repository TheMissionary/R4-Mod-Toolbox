<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { slide } from 'svelte/transition';
  import {
    HelpCircle,
    BookOpen,
    Link2,
    AlertTriangle,
    ExternalLink,
    FolderTree,
    Cpu,
    ChevronDown,
    ChevronRight,
    Star,
    Code
  } from 'lucide-svelte';

  // State to track which accordion section is currently open. 
  // Defaulting to the first section so the page doesn't look completely empty.
  let openSection = $state<string | null>('load_order');

  function toggleSection(section: string) {
    openSection = openSection === section ? null : section;
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
  <!-- Header -->
  <div class="p-6 pb-4 shrink-0 border-b border-nvidia-border/50 bg-nvidia-surface/20">
    <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
      <HelpCircle class="h-4 w-4" />
      <span>HELP & DOCUMENTATION</span>
    </div>
    <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">R4 Knowledge Base</h2>
    <p class="text-xs text-nvidia-text-muted max-w-2xl">
      Learn how to manage your load order, resolve archive conflicts, and utilize advanced features like XL linking and physical categories.
    </p>
  </div>

  <!-- Scrollable Content -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    
    <div class="max-w-4xl space-y-3">
      
      <!-- Accordion 1: Load Order & Categories -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <button 
          type="button"
          onclick={() => toggleSection('load_order')}
          class="w-full px-4 py-3 flex items-center justify-between bg-nvidia-surface/60 hover:bg-nvidia-surface/90 transition-colors cursor-pointer"
        >
          <div class="flex items-center gap-2.5">
            <FolderTree class="h-4 w-4 text-nvidia-accent" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Load Order & Categories</h3>
          </div>
          {#if openSection === 'load_order'}
            <ChevronDown class="h-4 w-4 text-nvidia-text-muted" />
          {:else}
            <ChevronRight class="h-4 w-4 text-nvidia-text-muted" />
          {/if}
        </button>
        
        {#if openSection === 'load_order'}
          <div transition:slide={{ duration: 200 }} class="border-t border-nvidia-border/60 bg-nvidia-bg/30">
            <div class="p-5 space-y-4 text-xs text-nvidia-text-muted leading-relaxed">
              <div>
                <strong class="text-nvidia-text-primary block mb-1">Understanding Load Order:</strong>
                Cyberpunk 2077 uses a first-come, first-served loading system. Archives at the <strong class="text-nvidia-text-primary">top</strong> load first and lay claim to their assets (<span class="text-[#22c55e]">●</span> 'win'). Mods placed lower that attempt to modify the same assets are blocked (<span class="text-[#ef4444]">●</span> 'loss').
              </div>
              <div>
                <strong class="text-nvidia-text-primary block mb-1">Adjusting the Order:</strong>
                To change which mod wins a conflict, simply click and drag the grip handle on any mod to move it higher or lower in the list.
              </div>
              <div>
                <strong class="text-nvidia-text-primary block mb-1">Physical Categories:</strong>
                R4 leverages native engine mechanics to create categories without compromising your data through app-injected logic. These optional created categories are real 0-byte <span class="font-mono text-nvidia-text-primary">.archive</span> files, meaning your perfect build is preserved even if you launch the game without the toolbox.
              </div>
              <div>
                <strong class="text-nvidia-text-primary block mb-1">Multi-Select:</strong>
                Hold <kbd class="px-1.5 py-0.5 rounded bg-nvidia-card border border-nvidia-border font-mono text-[10px] text-nvidia-text-primary">Ctrl</kbd> to select multiple mods, or <kbd class="px-1.5 py-0.5 rounded bg-nvidia-card border border-nvidia-border font-mono text-[10px] text-nvidia-text-primary">Shift</kbd> to select a range. Right-click the selection to move, disable, or categorize them all at once.
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 2: Conflict Resolution -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <button 
          type="button"
          onclick={() => toggleSection('conflicts')}
          class="w-full px-4 py-3 flex items-center justify-between bg-nvidia-surface/60 hover:bg-nvidia-surface/90 transition-colors cursor-pointer"
        >
          <div class="flex items-center gap-2.5">
            <AlertTriangle class="h-4 w-4 text-amber-500" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Conflict Resolution</h3>
          </div>
          {#if openSection === 'conflicts'}
            <ChevronDown class="h-4 w-4 text-nvidia-text-muted" />
          {:else}
            <ChevronRight class="h-4 w-4 text-nvidia-text-muted" />
          {/if}
        </button>
        
        {#if openSection === 'conflicts'}
          <div transition:slide={{ duration: 200 }} class="border-t border-nvidia-border/60 bg-nvidia-bg/30">
            <div class="p-5 space-y-4 text-xs text-nvidia-text-muted leading-relaxed">
              <p>
                R4 scans the internal hashes of every <span class="font-mono text-nvidia-text-primary">.archive</span> file to detect exact file collisions.
              </p>
              <ul class="space-y-2 bg-nvidia-surface/30 p-3 rounded border border-nvidia-border/50">
                <li class="flex items-start gap-2.5">
                  <div class="h-2.5 w-2.5 rounded-full bg-[#22c55e] shadow-[0_0_6px_rgba(34,197,94,0.7)] mt-0.5 shrink-0"></div>
                  <span><strong class="text-[#22c55e]">Green (Winning):</strong> This mod is placed higher in the load order and successfully overwrites conflicting assets from mods below it.</span>
                </li>
                <li class="flex items-start gap-2.5">
                  <div class="h-2.5 w-2.5 rounded-full bg-[#ef4444] shadow-[0_0_6px_rgba(239,68,68,0.7)] mt-0.5 shrink-0"></div>
                  <span><strong class="text-[#ef4444]">Red (Losing):</strong> This mod is placed lower in the load order, and its assets are being overwritten by a winning mod above it.</span>
                </li>
              </ul>
              <p>
                Click the chevron next to the conflict count to see exactly which mods are winning or losing against the selected file. Utilize the <strong class="text-nvidia-text-primary">Conflict Summary</strong> drawer to quickly narrow your view to only competing mods. The drawer updates conflicts in real-time as you adjust your load order to resolve contested assets.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 3: XL Linking -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <button 
          type="button"
          onclick={() => toggleSection('xl_linking')}
          class="w-full px-4 py-3 flex items-center justify-between bg-nvidia-surface/60 hover:bg-nvidia-surface/90 transition-colors cursor-pointer"
        >
          <div class="flex items-center gap-2.5">
            <Link2 class="h-4 w-4 text-cyan-400" />
            <h3 class="text-sm font-bold uppercase tracking-wider">XL Linking</h3>
          </div>
          {#if openSection === 'xl_linking'}
            <ChevronDown class="h-4 w-4 text-nvidia-text-muted" />
          {:else}
            <ChevronRight class="h-4 w-4 text-nvidia-text-muted" />
          {/if}
        </button>
        
        {#if openSection === 'xl_linking'}
          <div transition:slide={{ duration: 200 }} class="border-t border-nvidia-border/60 bg-nvidia-bg/30">
            <div class="p-5 space-y-4 text-xs text-nvidia-text-muted leading-relaxed">
              <p>
                ArchiveXL requires <span class="font-mono text-nvidia-text-primary">.xl</span> files to load custom entities. R4 automatically pairs these files with their parent <span class="font-mono text-nvidia-text-primary">.archive</span> if they share the same name.
              </p>
              <p>
                If a mod author named them differently, the <span class="font-mono text-nvidia-text-primary">.xl</span> file will appear at the bottom of the Archive tab under "Unassociated .xl Files".
              </p>
              <p>
                <strong class="text-cyan-400">How to Link:</strong> Click "Active / Click to Link" on the loose file, search for the parent archive, and confirm. The parent will now display a blue <strong class="text-cyan-400">[XL]</strong> badge. Toggling the parent mod will now automatically toggle the linked <span class="font-mono text-nvidia-text-primary">.xl</span> file.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 4: Frameworks & Scripts -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <button 
          type="button"
          onclick={() => toggleSection('frameworks')}
          class="w-full px-4 py-3 flex items-center justify-between bg-nvidia-surface/60 hover:bg-nvidia-surface/90 transition-colors cursor-pointer"
        >
          <div class="flex items-center gap-2.5">
            <Cpu class="h-4 w-4 text-nvidia-accent" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Frameworks & Scripts</h3>
          </div>
          {#if openSection === 'frameworks'}
            <ChevronDown class="h-4 w-4 text-nvidia-text-muted" />
          {:else}
            <ChevronRight class="h-4 w-4 text-nvidia-text-muted" />
          {/if}
        </button>
        
        {#if openSection === 'frameworks'}
          <div transition:slide={{ duration: 200 }} class="border-t border-nvidia-border/60 bg-nvidia-bg/30">
            <div class="p-5 space-y-4 text-xs text-nvidia-text-muted leading-relaxed">
              <p>
                R4 Mod Toolbox supports the following Cyberpunk 2077 modding frameworks: Archive, CET (Cyber Engine Tweaks), RED4ext, R6\Scripts (Redscript), and R6\Tweaks (TweakXL).
              </p>
              
              <div>
                <strong class="text-nvidia-text-primary block mb-1">Disabling & Enabling Mods:</strong>
                R4 uses two highly optimized methods depending on the mod type to ensure safety:
                <ul class="list-disc pl-5 mt-1.5 space-y-1.5">
                  <li><strong class="text-nvidia-text-primary">Archive Mods:</strong> Toggling an archive renames it in-place (appending <span class="font-mono text-nvidia-text-primary">.disabled</span>). It never leaves your mod folder and retains its exact alphabetical position.</li>
                  <li><strong class="text-nvidia-text-primary">Framework & Script Mods:</strong> Toggling these physically moves the files to a <span class="font-mono text-nvidia-text-primary">Disabled_Mods</span> folder in the game's root directory to prevent engine compilation errors.</li>
                </ul>
                <p class="mt-2">Both methods execute in milliseconds, ensuring the game engine completely ignores the disabled files without permanently deleting your data.</p>
              </div>

              <div>
                <strong class="text-nvidia-text-primary block mb-1">Viewing & Editing Files:</strong>
                R4 displays associated mod files directly in the interface. You can right-click mod files (like <span class="font-mono text-nvidia-text-primary">.log</span>, <span class="font-mono text-nvidia-text-primary">.yaml</span>, <span class="font-mono text-nvidia-text-primary">.ini</span>, <span class="font-mono text-nvidia-text-primary">.lua</span>, etc.) and select 'Open in Editor'. You can select your preferred text editor in the Settings tab.
              </div>
            </div>
          </div>
        {/if}
      </div>

    </div>

    <!-- Official R4 Community & Support -->
    <div class="max-w-4xl mt-8">
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Star class="h-4 w-4 text-amber-500" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Official R4 Community & Support</h3>
        </div>
        <div class="p-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
          <button
            type="button"
            onclick={() => handleOpenLink('https://www.nexusmods.com/cyberpunk2077/mods/34691')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-amber-500/50 transition-colors">
                <Star class="h-4 w-4 text-nvidia-text-muted group-hover:text-amber-500 transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">Nexus Mods Page</div>
                <div class="text-[10px] text-nvidia-text-muted">Download updates & join discussions</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>

          <button
            type="button"
            onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-nvidia-accent transition-colors">
                <Code class="h-4 w-4 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">GitHub Repository</div>
                <div class="text-[10px] text-nvidia-text-muted">View source code & track development</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>

          <button
            type="button"
            onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/issues')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-red-400/50 transition-colors">
                <AlertTriangle class="h-4 w-4 text-nvidia-text-muted group-hover:text-red-400 transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">Report an Issue</div>
                <div class="text-[10px] text-nvidia-text-muted">Submit bug reports to the developer</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>
        </div>
      </div>
    </div>

    <!-- General Modding Resources -->
    <div class="max-w-4xl mt-6">
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <BookOpen class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">General Modding Resources</h3>
        </div>
        <div class="p-2 grid grid-cols-1 sm:grid-cols-2 gap-2">
          <button
            type="button"
            onclick={() => handleOpenLink('https://wiki.redmodding.org/')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-nvidia-accent transition-colors">
                <BookOpen class="h-4 w-4 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">REDmodding Wiki</div>
                <div class="text-[10px] text-nvidia-text-muted">Official modding documentation</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>

          <button
            type="button"
            onclick={() => handleOpenLink('https://wiki.redmodding.org/cyber-engine-tweaks/')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-nvidia-accent transition-colors">
                <Cpu class="h-4 w-4 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">CET Wiki</div>
                <div class="text-[10px] text-nvidia-text-muted">Cyber Engine Tweaks guide</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>
        </div>
      </div>
    </div>

  </div>
</div>
