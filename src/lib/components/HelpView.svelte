<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { slide } from 'svelte/transition';
  import {
    HelpCircle, Link2, AlertTriangle, ExternalLink, FolderTree, Cpu,
    ChevronDown, ChevronRight, Star, Code, FolderOpen, Trash2, Check, LayoutGrid
  } from 'lucide-svelte';

  let openSection = $state<string | null>('support');
  let clearedFeedback = $state(false);

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

  async function handleOpenLogs() {
    try {
      await invoke('open_log_folder');
    } catch (err) {
      console.error('Failed to open log folder:', err);
    }
  }

  async function handleClearLogs() {
    try {
      await invoke('clear_user_log');
      clearedFeedback = true;
      setTimeout(() => { clearedFeedback = false; }, 2000);
    } catch (err) {
      console.error('Failed to clear logs:', err);
    }
  }
</script>

<div class="flex flex-col h-full overflow-hidden select-none" style="background-color: var(--theme-bg); color: var(--theme-text-primary);">
  <div class="p-6 pb-4 shrink-0 border-b" style="border-color: color-mix(in srgb, var(--theme-border) 50%, transparent); background-color: color-mix(in srgb, var(--theme-surface) 20%, transparent);">
    <div class="flex items-center gap-2 text-xs font-mono mb-2" style="color: var(--theme-accent);">
      <HelpCircle class="h-4 w-4" />
      <span>HELP & DOCUMENTATION</span>
    </div>
    <h2 class="text-xl font-bold mb-1" style="color: var(--theme-text-primary);">R4 Knowledge Base</h2>
    <p class="text-xs max-w-2xl" style="color: var(--theme-text-muted);">
      Learn how to manage your load order, resolve archive conflicts, and utilize advanced features like XL linking and simulated groups.
    </p>
  </div>

  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <div class="max-w-4xl space-y-3">
      
      <!-- Accordion 1: Load Order & Groups -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('load_order')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <FolderTree class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Load Order & Groups</h3>
          </div>
          {#if openSection === 'load_order'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'load_order'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-4 text-xs leading-relaxed" style="color: var(--theme-text-muted);">
              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Understanding Load Order:</strong>
                Cyberpunk 2077 uses a first-come, first-served loading system. Archives at the <strong style="color: var(--theme-text-primary);">top</strong> load first and lay claim to their assets (<span style="color: var(--color-success);">●</span> 'win'). Mods placed lower that attempt to modify the same assets are blocked (<span style="color: var(--color-danger);">●</span> 'loss').
              </div>
              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Adjusting the Order:</strong>
                To change which mod wins a conflict, simply click and drag the grip handle on any mod to move it higher or lower in the list.
              </div>
              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Virtual Groups:</strong>
                R4 allows you to create virtual groups directly within your load order to organize related mods. These are purely organizational. Under the hood, R4 creates a safe, 0-byte <span class="font-mono" style="color: var(--theme-text-primary);">.txt</span> file (e.g., <span class="font-mono" style="color: var(--theme-text-primary);">[DIR] UI Mods.txt</span>) in your mod directory. The game engine completely ignores these files, ensuring your load order remains perfectly intact even if you launch the game without the toolbox.
              </div>
              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Multi-Select:</strong>
                Hold <kbd class="px-1.5 py-0.5 rounded border font-mono text-[10px]" style="background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);">Ctrl</kbd> to select multiple mods, or <kbd class="px-1.5 py-0.5 rounded border font-mono text-[10px]" style="background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);">Shift</kbd> to select a range. Right-click the selection to move, disable, or organize them all at once.
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 2: Organizer View -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('organizer')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <LayoutGrid class="h-4 w-4" style="color: var(--color-info);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Organizer View (Virtual Categories)</h3>
          </div>
          {#if openSection === 'organizer'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'organizer'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-4 text-xs leading-relaxed" style="color: var(--theme-text-muted);">
              <div class="p-3 rounded border mb-2" style="background-color: color-mix(in srgb, var(--color-info) 10%, transparent); border-color: color-mix(in srgb, var(--color-info) 30%, transparent); color: var(--theme-text-primary);">
                <strong style="color: var(--color-info);">Note:</strong> The Organizer View is a completely decoupled visual tool. It allows you to view and organize your mods in a way that works best for you, <strong>without impacting your physical load order or game performance in any way</strong>.
              </div>
              <p>
                The Organizer View is a purely visual layer that allows you to group your mods into custom-named, color-coded categories (like "Vehicles", "Weapons", or "Clothing") without altering their physical load order.
              </p>
              <p>
                <strong>How it works:</strong><br/>
                • <strong>Grid UI:</strong> When enabled, you can toggle the Archive tab into "Organizer View". This displays your categories as a grid of boxes.<br/>
                • <strong>Zero Load Order Impact:</strong> Dragging a mod into a category here only assigns it a visual label. It does not move the file on your hard drive or change its conflict priority.<br/>
                • <strong>Labels in Load Order:</strong> You can enable an optional feature to display these category labels directly within the standard Load Order view, giving you visual context while managing physical conflicts.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 3: Conflict Resolution -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('conflicts')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <AlertTriangle class="h-4 w-4" style="color: var(--color-warning);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Conflict Resolution</h3>
          </div>
          {#if openSection === 'conflicts'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'conflicts'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-4 text-xs leading-relaxed" style="color: var(--theme-text-muted);">
              <p>
                R4 scans the internal hashes of every <span class="font-mono" style="color: var(--theme-text-primary);">.archive</span> file to detect exact file collisions.
              </p>
              <ul class="space-y-2 p-3 rounded border" style="background-color: color-mix(in srgb, var(--theme-surface) 30%, transparent); border-color: color-mix(in srgb, var(--theme-border) 50%, transparent);">
                <li class="flex items-start gap-2.5">
                  <div class="h-2.5 w-2.5 rounded-full mt-0.5 shrink-0" style="background-color: var(--color-success); box-shadow: 0 0 6px var(--color-success);"></div>
                  <span><strong style="color: var(--color-success);">Green (Winning):</strong> This mod is placed higher in the load order and successfully overwrites conflicting assets from mods below it.</span>
                </li>
                <li class="flex items-start gap-2.5">
                  <div class="h-2.5 w-2.5 rounded-full mt-0.5 shrink-0" style="background-color: var(--color-danger); box-shadow: 0 0 6px var(--color-danger);"></div>
                  <span><strong style="color: var(--color-danger);">Red (Losing):</strong> This mod is placed lower in the load order, and its assets are being overwritten by a winning mod above it.</span>
                </li>
              </ul>
              <p>
                Click the chevron next to the conflict count to see exactly which mods are winning or losing against the selected file. Utilize the <strong style="color: var(--theme-text-primary);">Conflict Summary</strong> drawer to quickly narrow your view to only competing mods. The drawer updates conflicts in real-time as you adjust your load order to resolve contested assets.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 4: XL Linking -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('xl_linking')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <Link2 class="h-4 w-4" style="color: var(--color-info);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">XL Linking</h3>
          </div>
          {#if openSection === 'xl_linking'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'xl_linking'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-4 text-xs leading-relaxed" style="color: var(--theme-text-muted);">
              <p>
                ArchiveXL requires <span class="font-mono" style="color: var(--theme-text-primary);">.xl</span> files to load custom entities. R4 automatically pairs these files with their parent <span class="font-mono" style="color: var(--theme-text-primary);">.archive</span> if they share the same name.
              </p>
              <p>
                If a mod author named them differently, the <span class="font-mono" style="color: var(--theme-text-primary);">.xl</span> file will appear at the bottom of the Archive tab under "Unassociated .xl Files".
              </p>
              <p>
                <strong style="color: var(--color-info);">How to Link:</strong> Click "Active / Click to Link" on the loose file, search for the parent archive, and confirm. The parent will now display a blue <strong style="color: var(--color-info);">[XL]</strong> badge. Toggling the parent mod will now automatically toggle the linked <span class="font-mono" style="color: var(--theme-text-primary);">.xl</span> file.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 5: Frameworks & Scripts -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('frameworks')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <Cpu class="h-4 w-4" style="color: var(--theme-accent);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Frameworks & Scripts</h3>
          </div>
          {#if openSection === 'frameworks'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'frameworks'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-4 text-xs leading-relaxed" style="color: var(--theme-text-muted);">
              <p>
                R4 Mod Toolbox supports the following Cyberpunk 2077 modding frameworks: Archive, CET (Cyber Engine Tweaks), RED4ext, R6\Scripts (Redscript), and R6\Tweaks (TweakXL).
              </p>
              
              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Disabling & Enabling Mods:</strong>
                R4 uses two highly optimized methods depending on the mod type to ensure safety:
                <ul class="list-disc pl-5 mt-1.5 space-y-1.5">
                  <li><strong style="color: var(--theme-text-primary);">Archive Mods:</strong> Toggling an archive renames it in-place (appending <span class="font-mono" style="color: var(--theme-text-primary);">.disabled</span>). It never leaves your mod folder and retains its exact alphabetical position.</li>
                  <li><strong style="color: var(--theme-text-primary);">Framework & Script Mods:</strong> Toggling these physically moves the files to a <span class="font-mono" style="color: var(--theme-text-primary);">Disabled_Mods</span> folder in the game's root directory to prevent engine compilation errors.</li>
                </ul>
                <p class="mt-2">Both methods execute in milliseconds, ensuring the game engine completely ignores the disabled files without permanently deleting your data.</p>
              </div>

              <div>
                <strong class="block mb-1" style="color: var(--theme-text-primary);">Viewing & Editing Files:</strong>
                R4 displays associated mod files directly in the interface. You can right-click mod files (like <span class="font-mono" style="color: var(--theme-text-primary);">.log</span>, <span class="font-mono" style="color: var(--theme-text-primary);">.yaml</span>, <span class="font-mono" style="color: var(--theme-text-primary);">.ini</span>, <span class="font-mono" style="color: var(--theme-text-primary);">.lua</span>, etc.) and select 'Open in Editor'. You can select your preferred text editor in the Settings tab.
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Accordion 6: Support & Feedback -->
      <div class="rounded-xl border overflow-hidden flex flex-col" style="background-color: color-mix(in srgb, var(--theme-surface) 40%, transparent); border-color: var(--theme-border);">
        <button 
          type="button"
          onclick={() => toggleSection('support')}
          class="w-full px-4 py-3 flex items-center justify-between transition-colors cursor-pointer hover:bg-[color-mix(in_srgb,var(--theme-surface)_90%,transparent)]"
          style="background-color: color-mix(in srgb, var(--theme-surface) 60%, transparent);"
        >
          <div class="flex items-center gap-2.5">
            <Star class="h-4 w-4" style="color: var(--color-warning);" />
            <h3 class="text-sm font-bold uppercase tracking-wider">Support & Feedback</h3>
          </div>
          {#if openSection === 'support'}
            <ChevronDown class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {:else}
            <ChevronRight class="h-4 w-4" style="color: var(--theme-text-muted);" />
          {/if}
        </button>
        
        {#if openSection === 'support'}
          <div transition:slide={{ duration: 200 }} class="border-t" style="border-color: color-mix(in srgb, var(--theme-border) 60%, transparent); background-color: color-mix(in srgb, var(--theme-bg) 30%, transparent);">
            <div class="p-5 space-y-6">
              
              <!-- Community Links -->
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <button
                  type="button"
                  onclick={() => handleOpenLink('https://www.nexusmods.com/cyberpunk2077/mods/34691')}
                  class="flex items-center justify-between p-3 rounded border transition cursor-pointer group hover:bg-[var(--theme-card)] hover:border-[color-mix(in_srgb,var(--color-warning)_50%,transparent)]"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border);"
                >
                  <div class="flex items-center gap-3">
                    <div class="h-8 w-8 rounded border flex items-center justify-center shrink-0 transition-colors group-hover:border-[color-mix(in_srgb,var(--color-warning)_50%,transparent)]" style="background-color: var(--theme-card); border-color: var(--theme-border);">
                      <Star class="h-4 w-4 transition-colors group-hover:text-[var(--color-warning)]" style="color: var(--theme-text-muted);" />
                    </div>
                    <div class="text-left">
                      <div class="text-xs font-bold" style="color: var(--theme-text-primary);">Nexus Mods</div>
                      <div class="text-[10px]" style="color: var(--theme-text-muted);">Download updates</div>
                    </div>
                  </div>
                  <ExternalLink class="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--theme-text-muted);" />
                </button>

                <button
                  type="button"
                  onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
                  class="flex items-center justify-between p-3 rounded border transition cursor-pointer group hover:bg-[var(--theme-card)] hover:border-[color-mix(in_srgb,var(--theme-accent)_50%,transparent)]"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border);"
                >
                  <div class="flex items-center gap-3">
                    <div class="h-8 w-8 rounded border flex items-center justify-center shrink-0 transition-colors group-hover:border-[var(--theme-accent)]" style="background-color: var(--theme-card); border-color: var(--theme-border);">
                      <Code class="h-4 w-4 transition-colors group-hover:text-[var(--theme-accent)]" style="color: var(--theme-text-muted);" />
                    </div>
                    <div class="text-left">
                      <div class="text-xs font-bold" style="color: var(--theme-text-primary);">GitHub Repo</div>
                      <div class="text-[10px]" style="color: var(--theme-text-muted);">View source code</div>
                    </div>
                  </div>
                  <ExternalLink class="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--theme-text-muted);" />
                </button>

                <button
                  type="button"
                  onclick={() => handleOpenLink('https://github.com/TheMissionary/R4-Mod-Toolbox/issues')}
                  class="flex items-center justify-between p-3 rounded border transition cursor-pointer group hover:bg-[var(--theme-card)] hover:border-[color-mix(in_srgb,var(--color-danger)_50%,transparent)]"
                  style="background-color: var(--theme-surface); border-color: var(--theme-border);"
                >
                  <div class="flex items-center gap-3">
                    <div class="h-8 w-8 rounded border flex items-center justify-center shrink-0 transition-colors group-hover:border-[color-mix(in_srgb,var(--color-danger)_50%,transparent)]" style="background-color: var(--theme-card); border-color: var(--theme-border);">
                      <AlertTriangle class="h-4 w-4 transition-colors group-hover:text-[var(--color-danger)]" style="color: var(--theme-text-muted);" />
                    </div>
                    <div class="text-left">
                      <div class="text-xs font-bold" style="color: var(--theme-text-primary);">Report Issue</div>
                      <div class="text-[10px]" style="color: var(--theme-text-muted);">Submit bug reports</div>
                    </div>
                  </div>
                  <ExternalLink class="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--theme-text-muted);" />
                </button>
              </div>

              <div class="h-px" style="background-color: color-mix(in srgb, var(--theme-border) 50%, transparent);"></div>

              <!-- Diagnostic Logs -->
              <div>
                <h4 class="text-xs font-bold uppercase tracking-wider mb-1" style="color: var(--theme-text-primary);">Diagnostic Logs</h4>
                <p class="text-[11px] mb-3" style="color: var(--theme-text-muted);">Application logs are saved automatically. Use these files when reporting issues to the developer.</p>
                <div class="flex items-center gap-3">
                  <button
                    type="button"
                    onclick={handleOpenLogs}
                    class="flex items-center gap-2 px-4 py-2 rounded border text-xs transition cursor-pointer shadow-xs hover:bg-[var(--theme-card)]"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  >
                    <FolderOpen class="h-4 w-4" style="color: var(--theme-text-muted);" />
                    <span>Open Log Folder</span>
                  </button>
                  <button
                    type="button"
                    onclick={handleClearLogs}
                    class="flex items-center gap-2 px-4 py-2 rounded border text-xs transition cursor-pointer shadow-xs hover:bg-[color-mix(in_srgb,var(--color-danger)_10%,transparent)] hover:border-[color-mix(in_srgb,var(--color-danger)_30%,transparent)] hover:text-[var(--color-danger)]"
                    style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-primary);"
                  >
                    {#if clearedFeedback}
                      <Check class="h-4 w-4" style="color: var(--theme-accent);" />
                      <span style="color: var(--theme-accent);">Cleared!</span>
                    {:else}
                      <Trash2 class="h-4 w-4" />
                      <span>Clear User Logs</span>
                    {/if}
                  </button>
                </div>
              </div>

            </div>
          </div>
        {/if}
      </div>

    </div>
  </div>
</div>