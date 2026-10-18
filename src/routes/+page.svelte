<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { info, error, warn } from '@tauri-apps/plugin-log';
  import {
    loadThemeSettings,
    applyThemeSettings,
    loadConfigFromDisk,
    saveConfigToDisk
  } from '$lib/theme';
  import type {
    ScanResult,
    ArchiveItem,
    ArchiveScanReport,
    CetPluginItem,
    Red4extPluginItem,
    RedScriptItem,
    R6TweaksItem
  } from '$lib/types';

  import HeaderBar from '$lib/components/HeaderBar.svelte';
  import ArchiveView from '$lib/components/ArchiveView.svelte';
  import CetView from '$lib/components/CetView.svelte';
  import Red4extView from '$lib/components/Red4extView.svelte';
  import RedscriptView from '$lib/components/RedScriptView.svelte';
  import R6TweaksView from '$lib/components/R6TweaksView.svelte';
  import SettingsView from '$lib/components/SettingsView.svelte';
  import HelpView from '$lib/components/HelpView.svelte';
  import AboutView from '$lib/components/AboutView.svelte';

  import {
    Archive,
    Cpu,
    Puzzle,
    FileCode2,
    Sliders,
    HelpCircle,
    Settings,
    Info,
    Coffee,
    PanelLeftOpen,
    PanelLeftClose
  } from 'lucide-svelte';

  type TabType = 'archive' | 'cet' | 'red4ext' | 'redscript' | 'r6tweaks' | 'settings' | 'help' | 'about';

  let currentTab = $state<TabType>('archive');
  let isSidebarCollapsed = $state(false);

  async function setTab(tab: TabType) {
    currentTab = tab;
    info(`Navigated to tab: ${tab}`);
    try {
      const config = await loadConfigFromDisk();
      config.activeTab = tab;
      await saveConfigToDisk(config);
    } catch (err) {
      warn(`Failed to save active tab state: ${err}`);
    }
  }

  let gamePath = $state('');
  let isLoading = $state(false);

  let scanResult = $state<ScanResult | null>(null);
  let archiveReport = $state<ArchiveScanReport | null>(null);
  let archives = $state<ArchiveItem[]>([]);
  let cetPlugins = $state<CetPluginItem[]>([]);
  let red4extPlugins = $state<Red4extPluginItem[]>([]);
  let redscriptPackages = $state<RedScriptItem[]>([]);
  let r6tweaks = $state<R6TweaksItem[]>([]);

  // New Mods Badge System State
  let newModsHours = $state(6);
  let showNewMods = $state(false);
  let targetHighlightMod = $state<string | null>(null);

  let newModsSaveTimeout: ReturnType<typeof setTimeout>;
  function handleNewModsConfigChange() {
    clearTimeout(newModsSaveTimeout);
    newModsSaveTimeout = setTimeout(async () => {
      try {
        const config = await loadConfigFromDisk();
        config.newModsHours = newModsHours;
        config.showNewMods = showNewMods;
        await saveConfigToDisk(config);
      } catch (err) {
        error(`Failed to save new mods config: ${err}`);
      }
    }, 200);
  }

  async function openExternalLink(url: string) {
    if (!url) return;
    try {
      await openUrl(url);
    } catch (err) {
      error(`Failed to open URL ${url}: ${err}`);
    }
  }

  async function loadTabData(tab: string) {
    if (!gamePath) return;
    try {
      if (tab === 'archive') {
        const report = await invoke<ArchiveScanReport>('get_archive_details', { gamePath });
        archiveReport = report;
        archives = report?.archives ?? [];
      } else if (tab === 'cet') {
        cetPlugins = await invoke<CetPluginItem[]>('get_cet_details', { gamePath });
      } else if (tab === 'red4ext') {
        red4extPlugins = await invoke<Red4extPluginItem[]>('get_red4ext_details', { gamePath });
      } else if (tab === 'redscript') {
        redscriptPackages = await invoke<RedScriptItem[]>('get_redscript_details', { gamePath });
      } else if (tab === 'r6tweaks') {
        r6tweaks = await invoke<R6TweaksItem[]>('get_r6tweaks_details', { gamePath });
      }
    } catch (err) {
      error(`Failed to load data for tab ${tab}: ${err}`);
    }
  }

  async function refreshAll() {
    if (!gamePath) return;
    isLoading = true;
    info('Initiating full workspace refresh...');
    try {
      scanResult = await invoke<ScanResult>('scan_game_directory', { gamePath });
      await loadTabData('archive');
      await loadTabData('cet');
      await loadTabData('red4ext');
      await loadTabData('redscript');
      await loadTabData('r6tweaks');
      info('Workspace refresh complete.');
    } catch (err) {
      error(`Failed to scan game environment: ${err}`);
    } finally {
      isLoading = false;
    }
  }

  async function refreshCountsOnly() {
    if (!gamePath) return;
    try {
      scanResult = await invoke<ScanResult>('scan_game_directory', { gamePath });
    } catch (err) {
      error(`Failed to refresh counts: ${err}`);
    }
  }

  onMount(() => {
    const localSettings = loadThemeSettings();
    applyThemeSettings(localSettings);

    const splashStartTime = Date.now();
    const dismissSplash = () => {
      const elapsed = Date.now() - splashStartTime;
      const remaining = Math.max(0, 800 - elapsed);
      setTimeout(() => {
        const splash = document.getElementById('app-splash');
        if (splash) {
          splash.classList.add('fade-out');
          setTimeout(() => splash.remove(), 350);
        }
      }, remaining);
    };

    loadConfigFromDisk().then(config => {
      if (config.theme) {
        applyThemeSettings(config.theme);
      }
      if (config.targetGamePath) {
        gamePath = config.targetGamePath;
        let tab = config.activeTab as TabType;
        if (tab === 'home' || !tab) tab = 'archive';
        currentTab = tab;
      } else {
        currentTab = 'settings';
      }
      if (config.newModsHours !== undefined) {
        newModsHours = config.newModsHours;
      }
      if (config.showNewMods !== undefined) {
        showNewMods = config.showNewMods;
      }
    }).catch(err => {
      error(`Config hydration error: ${err}`);
    }).finally(() => {
      if (gamePath) {
        refreshAll().finally(() => dismissSplash());
      } else {
        dismissSplash();
      }
    });

    invoke('start_directory_watcher', { gamePath }).catch(() => {});
    const unlisten = listen('directory-changed', () => {
      info('Directory change detected, triggering refresh.');
      refreshAll();
    });

    return () => {
      unlisten.then(f => f());
    };
  });
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<div class="flex h-screen w-screen overflow-hidden bg-nvidia-bg text-nvidia-text-primary font-sans">
  <!-- Left Navigation Sidebar -->
  <aside class="border-r border-nvidia-border flex flex-col justify-between bg-nvidia-surface/40 select-none shrink-0 transition-all duration-300 {isSidebarCollapsed ? 'w-[72px]' : 'w-64'}">
    <div class="flex flex-col h-full overflow-hidden">
      
      <!-- Sidebar Header / Toggle -->
      <div class="h-16 border-b border-nvidia-border flex items-center {isSidebarCollapsed ? 'justify-center' : 'px-5 gap-3'} shrink-0 transition-all duration-300">
        {#if isSidebarCollapsed}
          <button onclick={() => isSidebarCollapsed = false} class="p-2 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-nvidia-accent transition cursor-pointer" title="Expand Sidebar">
            <PanelLeftOpen class="h-5 w-5" />
          </button>
        {:else}
          <div class="h-6 w-6 rounded bg-nvidia-accent flex items-center justify-center text-black font-black text-sm shadow-sm shrink-0">R4</div>
          <div class="flex flex-col flex-1 min-w-0">
            <h1 class="text-xs font-bold tracking-wider uppercase text-nvidia-text-primary truncate">R4 Mod Toolbox</h1>
          </div>
          <button onclick={() => isSidebarCollapsed = true} class="p-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer shrink-0" title="Collapse Sidebar">
            <PanelLeftClose class="h-4 w-4" />
          </button>
        {/if}
      </div>

      <div class="flex-1 overflow-y-auto overflow-x-hidden">
        <nav class="p-3 space-y-1.5">
          <button
            type="button"
            onclick={() => setTab('archive')}
            title={isSidebarCollapsed ? 'Archive Mods' : ''}
            class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3'} py-2.5 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'archive' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
          >
            {#if currentTab === 'archive'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Archive class="h-4 w-4 {currentTab === 'archive' ? 'text-nvidia-accent' : ''}" />
              {#if !isSidebarCollapsed}
                <span>Archive</span>
              {/if}
            </div>
            {#if !isSidebarCollapsed && scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border/50">
                {scanResult.archive_active}/{scanResult.archive_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('cet')}
            title={isSidebarCollapsed ? 'CET Plugins' : ''}
            class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3'} py-2.5 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'cet' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
          >
            {#if currentTab === 'cet'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Cpu class="h-4 w-4 {currentTab === 'cet' ? 'text-nvidia-accent' : ''}" />
              {#if !isSidebarCollapsed}
                <span>CET</span>
              {/if}
            </div>
            {#if !isSidebarCollapsed && scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border/50">
                {scanResult.cet_active}/{scanResult.cet_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('red4ext')}
            title={isSidebarCollapsed ? 'RED4ext Plugins' : ''}
            class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3'} py-2.5 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'red4ext' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
          >
            {#if currentTab === 'red4ext'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Puzzle class="h-4 w-4 {currentTab === 'red4ext' ? 'text-nvidia-accent' : ''}" />
              {#if !isSidebarCollapsed}
                <span>RED4ext</span>
              {/if}
            </div>
            {#if !isSidebarCollapsed && scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border/50">
                {scanResult.red4ext_active}/{scanResult.red4ext_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('redscript')}
            title={isSidebarCollapsed ? 'R6\\Scripts' : ''}
            class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3'} py-2.5 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'redscript' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
          >
            {#if currentTab === 'redscript'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
            {/if}
            <div class="flex items-center gap-3">
              <FileCode2 class="h-4 w-4 {currentTab === 'redscript' ? 'text-nvidia-accent' : ''}" />
              {#if !isSidebarCollapsed}
                <span>R6\Scripts</span>
              {/if}
            </div>
            {#if !isSidebarCollapsed && scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border/50">
                {scanResult.redscript_active}/{scanResult.redscript_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('r6tweaks')}
            title={isSidebarCollapsed ? 'R6\\Tweaks' : ''}
            class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3'} py-2.5 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'r6tweaks' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
          >
            {#if currentTab === 'r6tweaks'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Sliders class="h-4 w-4 {currentTab === 'r6tweaks' ? 'text-nvidia-accent' : ''}" />
              {#if !isSidebarCollapsed}
                <span>R6\Tweaks</span>
              {/if}
            </div>
            {#if !isSidebarCollapsed && scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted border border-nvidia-border/50">
                {scanResult.r6tweaks_active}/{scanResult.r6tweaks_total}
              </span>
            {/if}
          </button>
        </nav>
      </div>

      <!-- Secondary Navigation Cluster -->
      <div class="px-3 pb-3 space-y-1.5 bg-nvidia-surface/20 shrink-0 border-t border-nvidia-border/50 pt-3">
        <button
          type="button"
          onclick={() => setTab('help')}
          title={isSidebarCollapsed ? 'Help & Documentation' : ''}
          class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'gap-3 px-3'} py-2 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'help' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
        >
          {#if currentTab === 'help'}
            <div class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
          {/if}
          <HelpCircle class="h-4 w-4 {currentTab === 'help' ? 'text-nvidia-accent' : ''}" />
          {#if !isSidebarCollapsed}
            <span>Help</span>
          {/if}
        </button>
        <button
          type="button"
          onclick={() => setTab('settings')}
          title={isSidebarCollapsed ? 'Settings' : ''}
          class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'gap-3 px-3'} py-2 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'settings' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
        >
          {#if currentTab === 'settings'}
            <div class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
          {/if}
          <Settings class="h-4 w-4 {currentTab === 'settings' ? 'text-nvidia-accent' : ''}" />
          {#if !isSidebarCollapsed}
            <span>Settings</span>
          {/if}
        </button>
        <button
          type="button"
          onclick={() => setTab('about')}
          title={isSidebarCollapsed ? 'About R4' : ''}
          class="relative w-full flex items-center {isSidebarCollapsed ? 'justify-center px-0' : 'gap-3 px-3'} py-2 rounded-lg text-xs font-medium transition cursor-pointer {currentTab === 'about' ? 'bg-nvidia-card text-nvidia-accent shadow-sm border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary border border-transparent'}"
        >
          {#if currentTab === 'about'}
            <div class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-r-md bg-nvidia-accent shadow-[0_0_8px_var(--theme-accent)]"></div>
          {/if}
          <Info class="h-4 w-4 {currentTab === 'about' ? 'text-nvidia-accent' : ''}" />
          {#if !isSidebarCollapsed}
            <span>About</span>
          {/if}
        </button>
      </div>
    </div>

    <!-- Developer Tile -->
    <div class="p-3 border-t border-nvidia-border bg-nvidia-surface/30 shrink-0 flex justify-center">
      {#if isSidebarCollapsed}
        <button 
          type="button" 
          onclick={() => openExternalLink('https://www.nexusmods.com/profile/Mi55ionary')} 
          class="h-8 w-8 rounded-full bg-amber-500/10 border border-amber-500/30 flex items-center justify-center hover:border-amber-500 transition cursor-pointer" 
          title="Mi55ionary"
        >
          <svg viewBox="0 0 100 100" class="h-5 w-5 drop-shadow-sm" fill="none" stroke="#e08528" stroke-width="9" stroke-linecap="round" stroke-linejoin="round">
            <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
          </svg>
        </button>
      {:else}
        <div class="w-full rounded-lg border border-nvidia-border/70 bg-nvidia-surface/70 p-2 flex items-center justify-between gap-2 shadow-xs select-none">
          <div class="flex items-center gap-2 min-w-0">
            <div class="h-6 w-6 rounded bg-amber-500/10 border border-amber-500/30 flex items-center justify-center shrink-0">
              <svg viewBox="0 0 100 100" class="h-4 w-4 drop-shadow-sm" fill="none" stroke="#e08528" stroke-width="9" stroke-linecap="round" stroke-linejoin="round">
                <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
              </svg>
            </div>
            <div class="min-w-0 flex flex-col">
              <span class="text-[9px] text-nvidia-text-muted uppercase tracking-wider font-semibold leading-tight">Dev</span>
              <button 
                type="button" 
                onclick={() => openExternalLink('https://www.nexusmods.com/profile/Mi55ionary')} 
                class="text-[11px] font-mono font-bold text-nvidia-text-primary hover:text-nvidia-accent transition-colors truncate leading-tight text-left cursor-pointer" 
                title="View Mi55ionary on Nexus Mods"
              >
                Mi55ionary
              </button>
            </div>
          </div>

          <div class="flex items-center gap-1 shrink-0">
            <button
              type="button"
              onclick={() => openExternalLink('https://ko-fi.com/themissionary')}
              class="px-2 py-1 rounded bg-nvidia-card hover:bg-cyan-500/10 text-[10px] font-mono text-nvidia-text-muted hover:text-cyan-400 border border-nvidia-border hover:border-cyan-500/30 transition cursor-pointer flex items-center justify-center gap-1.5"
              title="Support on Ko-fi"
            >
              <Coffee class="h-3 w-3" />
              <span class="font-bold">Ko-fi</span>
            </button>
          </div>
        </div>
      {/if}
    </div>
  </aside>

  <!-- Workspace Canvas -->
  <div class="flex-1 flex flex-col h-full overflow-hidden relative">
    <HeaderBar {gamePath} {currentTab} onRefresh={refreshAll} {isLoading} {scanResult} />

    <main class="flex-1 overflow-y-auto {['settings', 'help', 'about'].includes(currentTab) ? 'p-0' : 'p-6'}">
      
      <!-- Persistent Tab Views -->
      <div class={currentTab === 'archive' ? 'h-full' : 'hidden'}>
        <ArchiveView
          bind:archives
          {gamePath}
          scanReport={archiveReport}
          onScanRequested={refreshAll}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          bind:newModsHours
          bind:showNewMods
          onNewModsConfigChanged={handleNewModsConfigChange}
        />
      </div>

      <div class={currentTab === 'cet' ? 'h-full' : 'hidden'}>
        <CetView
          bind:plugins={cetPlugins}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          bind:newModsHours
          bind:showNewMods
          onNewModsConfigChanged={handleNewModsConfigChange}
        />
      </div>

      <div class={currentTab === 'red4ext' ? 'h-full' : 'hidden'}>
        <Red4extView
          bind:plugins={red4extPlugins}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          bind:newModsHours
          bind:showNewMods
          onNewModsConfigChanged={handleNewModsConfigChange}
        />
      </div>

      <div class={currentTab === 'redscript' ? 'h-full' : 'hidden'}>
        <RedscriptView
          bind:packages={redscriptPackages}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          bind:newModsHours
          bind:showNewMods
          onNewModsConfigChanged={handleNewModsConfigChange}
        />
      </div>

      <div class={currentTab === 'r6tweaks' ? 'h-full' : 'hidden'}>
        <R6TweaksView
          bind:tweaks={r6tweaks}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          bind:newModsHours
          bind:showNewMods
          onNewModsConfigChanged={handleNewModsConfigChange}
        />
      </div>

      <!-- Full-Screen Views -->
      <div class={currentTab === 'settings' ? 'h-full' : 'hidden'}>
        <SettingsView bind:gamePath onRefresh={refreshAll} />
      </div>

      <div class={currentTab === 'help' ? 'h-full' : 'hidden'}>
        <HelpView />
      </div>

      <div class={currentTab === 'about' ? 'h-full' : 'hidden'}>
        <AboutView />
      </div>
    </main>
  </div>
</div>