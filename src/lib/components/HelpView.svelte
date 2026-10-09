<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import {
    HelpCircle,
    BookOpen,
    Archive,
    Link2,
    AlertTriangle,
    ExternalLink,
    FolderTree,
    Cpu
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
    
    <div class="grid grid-cols-1 xl:grid-cols-2 gap-6 max-w-6xl">
      
      <!-- Load Order & Categories -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <FolderTree class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Load Order & Categories</h3>
        </div>
        <div class="p-4 space-y-3 text-xs text-nvidia-text-muted leading-relaxed flex-1">
          <p>
            <strong class="text-nvidia-text-primary">Drag and Drop:</strong> The Archive tab represents your exact <span class="font-mono text-nvidia-text-primary">modlist.txt</span> load order. Mods at the <strong class="text-nvidia-text-primary">top</strong> load first, and mods at the <strong class="text-nvidia-text-primary">bottom</strong> load last (overwriting those above them). Click and drag the grip icon to reorder.
          </p>
          <p>
            <strong class="text-nvidia-text-primary">Physical Categories:</strong> R4 uses 0-byte <span class="font-mono text-nvidia-text-primary">[CAT]</span> files to create categories. These are real files on your disk, meaning your organization is preserved even if you launch the game without the toolbox.
          </p>
          <p>
            <strong class="text-nvidia-text-primary">Multi-Select:</strong> Hold <kbd class="px-1 py-0.5 rounded bg-nvidia-card border border-nvidia-border font-mono text-[10px]">Ctrl</kbd> to select multiple mods, or <kbd class="px-1 py-0.5 rounded bg-nvidia-card border border-nvidia-border font-mono text-[10px]">Shift</kbd> to select a range. Right-click the selection to move them all into a category at once.
          </p>
        </div>
      </div>

      <!-- Conflict Resolution -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <AlertTriangle class="h-4 w-4 text-amber-500" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Conflict Resolution</h3>
        </div>
        <div class="p-4 space-y-3 text-xs text-nvidia-text-muted leading-relaxed flex-1">
          <p>
            R4 scans the internal hashes of every <span class="font-mono text-nvidia-text-primary">.archive</span> file to detect exact file collisions.
          </p>
          <ul class="space-y-2 mt-2">
            <li class="flex items-start gap-2">
              <div class="h-2.5 w-2.5 rounded-full bg-[#22c55e] shadow-[0_0_6px_rgba(34,197,94,0.7)] mt-0.5 shrink-0"></div>
              <span><strong class="text-[#22c55e]">Green (Winning):</strong> This mod is loaded lower in the order and is successfully overwriting assets from a mod above it.</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="h-2.5 w-2.5 rounded-full bg-[#ef4444] shadow-[0_0_6px_rgba(239,68,68,0.7)] mt-0.5 shrink-0"></div>
              <span><strong class="text-[#ef4444]">Red (Losing):</strong> This mod is loaded higher in the order, and some or all of its assets are being overwritten by a mod below it.</span>
            </li>
          </ul>
          <p class="pt-1">
            Click the chevron next to the conflict count to see exactly which mods are winning or losing against the selected file.
          </p>
        </div>
      </div>

      <!-- XL Linking -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Link2 class="h-4 w-4 text-cyan-400" />
          <h3 class="text-sm font-bold uppercase tracking-wider">XL Linking</h3>
        </div>
        <div class="p-4 space-y-3 text-xs text-nvidia-text-muted leading-relaxed flex-1">
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

      <!-- Frameworks & Scripts -->
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden flex flex-col">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <Cpu class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">Frameworks & Scripts</h3>
        </div>
        <div class="p-4 space-y-3 text-xs text-nvidia-text-muted leading-relaxed flex-1">
          <p>
            R4 manages all major Cyberpunk 2077 modding frameworks: <strong class="text-nvidia-text-primary">CET, RED4ext, Redscript, and R6 Tweaks.</strong>
          </p>
          <p>
            <strong class="text-nvidia-text-primary">Disabling Mods:</strong> When you disable a script or framework mod, R4 physically moves it to a <span class="font-mono text-nvidia-text-primary">Disabled_Mods</span> folder in your game directory. This ensures the game engine completely ignores it, preventing compilation errors.
          </p>
          <p>
            <strong class="text-nvidia-text-primary">Editing Files:</strong> You can right-click any configuration file (like <span class="font-mono text-nvidia-text-primary">.yaml</span>, <span class="font-mono text-nvidia-text-primary">.ini</span>, or <span class="font-mono text-nvidia-text-primary">.lua</span>) and select "Open in Editor". Configure your preferred text editor in the Settings tab.
          </p>
        </div>
      </div>

    </div>

    <!-- External Resources -->
    <div class="max-w-6xl">
      <div class="rounded-xl border border-nvidia-border bg-nvidia-surface/40 overflow-hidden">
        <div class="px-4 py-3 border-b border-nvidia-border/60 bg-nvidia-surface/60 flex items-center gap-2.5">
          <BookOpen class="h-4 w-4 text-nvidia-accent" />
          <h3 class="text-sm font-bold uppercase tracking-wider">External Resources</h3>
        </div>
        <div class="p-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
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

          <button
            type="button"
            onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/issues')}
            class="flex items-center justify-between p-3 rounded hover:bg-nvidia-surface border border-transparent hover:border-nvidia-border transition cursor-pointer group"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded bg-nvidia-card border border-nvidia-border flex items-center justify-center shrink-0 group-hover:border-nvidia-accent transition-colors">
                <AlertTriangle class="h-4 w-4 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors" />
              </div>
              <div class="text-left">
                <div class="text-xs font-bold text-nvidia-text-primary">Report an Issue</div>
                <div class="text-[10px] text-nvidia-text-muted">GitHub Issue Tracker</div>
              </div>
            </div>
            <ExternalLink class="h-3.5 w-3.5 text-nvidia-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
          </button>
        </div>
      </div>
    </div>

  </div>
</div>
