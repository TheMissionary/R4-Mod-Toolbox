<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
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
  import HistoryView from '$lib/components/HistoryView.svelte';
  import SettingsView from '$lib/components/SettingsView.svelte';
  import HelpView from '$lib/components/HelpView.svelte';
  import AboutView from '$lib/components/AboutView.svelte';

  import {
    Folder,
    Clock,
    Camera,
    HelpCircle,
    Settings,
    Info,
    CheckCircle2,
    AlertCircle,
    AlertTriangle,
    RefreshCw,
    Check,
    Power
  } from 'lucide-svelte';

  type TabType = 'archive' | 'cet' | 'red4ext' | 'redscript' | 'r6tweaks' | 'history' | 'settings' | 'help' | 'about';

  let currentTab = $state<TabType>('archive');
  let customPhotoPath = $state('');
  let enableOrganizerView = $state(false);
  let showCategoryLabelsInLoadOrder = $state(true);

  let heroBannerUrl = $state<string | null>(null);

  function updateHeroBanner() {
    const settings = loadThemeSettings();
    if (settings.heroBannerPath) {
      heroBannerUrl = convertFileSrc(settings.heroBannerPath);
    } else {
      heroBannerUrl = null;
    }
  }

  async function setTab(tab: TabType) {
    currentTab = tab;
    info(`Navigated to tab: ${tab}`);
    try {
      const config = await loadConfigFromDisk();
      config.activeTab = tab;
      customPhotoPath = config.customPhotoPath || '';
      enableOrganizerView = config.enableOrganizerView || false;
      showCategoryLabelsInLoadOrder = config.showCategoryLabelsInLoadOrder ?? true;
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

  let newModsHours = $state(6);
  let targetHighlightMod = $state<string | null>(null);

  let globalSelectedPath = $state<string>('');

  let sidebarContextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    modType: string;
    label: string;
  }>({ visible: false, x: 0, y: 0, modType: '', label: '' });

  function openSidebarContextMenu(event: MouseEvent, modType: string, label: string) {
    event.preventDefault();
    event.stopPropagation();
    const menuWidth = 160;
    const menuHeight = 80;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;
    sidebarContextMenu = { visible: true, x: posX, y: posY, modType, label };
  }

  function closeSidebarContextMenu() {
    if (sidebarContextMenu.visible) {
      sidebarContextMenu.visible = false;
    }
  }

  async function handleBulkToggle(enable: boolean) {
    if (!gamePath || !sidebarContextMenu.modType) return;
    const type = sidebarContextMenu.modType;
    closeSidebarContextMenu();
    isLoading = true;
    try {
      await invoke('bulk_toggle_mod_type', { gamePath, modType: type, enable });
      await refreshAll();
    } catch (err) {
      error(`Failed to bulk toggle ${type}: ${err}`);
      isLoading = false;
    }
  }

  function handleNavigateToMod(tab: string, modName: string) {
    setTab(tab as TabType);
    targetHighlightMod = modName;
    setTimeout(() => { targetHighlightMod = null; }, 100);
  }

  function handleNavigateToHistory(modName: string) {
    setTab('history');
    targetHighlightMod = modName;
    setTimeout(() => { targetHighlightMod = null; }, 100);
  }

  async function handleOpenPhotos() {
    if (!customPhotoPath) return;
    try {
      await invoke('open_custom_folder', { folderPath: customPhotoPath });
    } catch (err) {
      error(`Failed to open photo folder: ${err}`);
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

  $effect(() => {
    if (gamePath) {
      invoke('start_directory_watcher', { gamePath }).catch(err => warn(`Watcher error: ${err}`));
    }
  });

  onMount(() => {
    const localSettings = loadThemeSettings();
    applyThemeSettings(localSettings);
    updateHeroBanner();

    window.addEventListener('theme-updated', updateHeroBanner);

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
        currentTab = 'archive';
      } else {
        currentTab = 'settings';
      }
      if (config.newModsHours !== undefined) {
        newModsHours = config.newModsHours;
      }
      if (config.customPhotoPath) {
        customPhotoPath = config.customPhotoPath;
      }
      enableOrganizerView = config.enableOrganizerView || false;
      showCategoryLabelsInLoadOrder = config.showCategoryLabelsInLoadOrder ?? true;
    }).catch(err => {
      error(`Config hydration error: ${err}`);
    }).finally(() => {
      if (gamePath) {
        refreshAll().finally(() => dismissSplash());
      } else {
        dismissSplash();
      }
    });

    const unlisten = listen('directory-changed', () => {
      info('Directory change detected, triggering refresh.');
      refreshAll();
    });

    return () => {
      window.removeEventListener('theme-updated', updateHeroBanner);
      unlisten.then(f => f());
    };
  });

  let activeCount = $derived.by(() => {
    if (!scanResult) return 0;
    switch(currentTab) {
      case 'archive': return scanResult.archive_active;
      case 'cet': return scanResult.cet_active;
      case 'red4ext': return scanResult.red4ext_active;
      case 'redscript': return scanResult.redscript_active;
      case 'r6tweaks': return scanResult.r6tweaks_active;
      default: return 0;
    }
  });

  let totalCount = $derived.by(() => {
    if (!scanResult) return 0;
    switch(currentTab) {
      case 'archive': return scanResult.archive_total;
      case 'cet': return scanResult.cet_total;
      case 'red4ext': return scanResult.red4ext_total;
      case 'redscript': return scanResult.redscript_total;
      case 'r6tweaks': return scanResult.r6tweaks_total;
      default: return 0;
    }
  });
</script>

<svelte:window onclick={closeSidebarContextMenu} oncontextmenu={(e) => e.preventDefault()} />

{#if sidebarContextMenu.visible}
  <div
    class="fixed z-50 w-40 rounded-md border py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {sidebarContextMenu.x}px; top: {sidebarContextMenu.y}px; background-color: var(--theme-card); border-color: var(--theme-border);"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b flex items-center gap-2" style="border-color: var(--theme-border); background-color: var(--theme-surface);">
      <span class="font-mono text-[11px] font-bold truncate" style="color: var(--theme-text-primary);">
        {sidebarContextMenu.label}
      </span>
    </div>
    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={() => handleBulkToggle(true)}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Check class="h-3.5 w-3.5" style="color: var(--theme-accent);" />
        <span>Enable All</span>
      </button>
      <button
        type="button"
        onclick={() => handleBulkToggle(false)}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded transition text-left cursor-pointer group hover:brightness-110"
        style="color: var(--theme-text-primary);"
      >
        <Power class="h-3.5 w-3.5" style="color: var(--color-danger);" />
        <span>Disable All</span>
      </button>
    </div>
  </div>
{/if}

<div class="flex h-screen w-screen overflow-hidden font-sans" style="background-color: var(--theme-bg); color: var(--theme-text-primary);">
  
  <!-- Left Navigation Sidebar -->
  <aside class="w-64 border-r flex flex-col justify-between shrink-0 select-none z-20" style="background-color: var(--theme-surface); border-color: var(--theme-border);">
    <div class="flex flex-col h-full overflow-hidden">
      
      <!-- Branding Header -->
      <div class="h-14 border-b flex items-center px-5 gap-3 shrink-0" style="border-color: var(--theme-border);">
        <div class="h-7 w-7 rounded flex items-center justify-center font-black text-sm shadow-sm shrink-0" style="background-color: var(--theme-accent); color: var(--theme-accent-contrast);">
          R4
        </div>
        <h1 class="text-xs font-bold tracking-wider uppercase truncate" style="color: var(--theme-text-primary);">R4 Mod Toolbox</h1>
      </div>

      <!-- Quick Access Navigation -->
      <div class="flex-1 overflow-y-auto overflow-x-hidden">
        <nav class="p-3 space-y-1.5">
          <button
            type="button"
            onclick={() => setTab('archive')}
            oncontextmenu={(e) => openSidebarContextMenu(e, 'archive', 'Archive Mods')}
            title="Right-click for options"
            class="relative w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-medium transition cursor-pointer border"
            style="{currentTab === 'archive' ? 'background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
          >
            {#if currentTab === 'archive'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Folder class="h-4 w-4" style="{currentTab === 'archive' ? 'color: var(--theme-accent);' : ''}" />
              <span>Archive Mods</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => setTab('cet')}
            oncontextmenu={(e) => openSidebarContextMenu(e, 'cet', 'CET Plugins')}
            title="Right-click for options"
            class="relative w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-medium transition cursor-pointer border"
            style="{currentTab === 'cet' ? 'background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
          >
            {#if currentTab === 'cet'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Folder class="h-4 w-4" style="{currentTab === 'cet' ? 'color: var(--theme-accent);' : ''}" />
              <span>CET Plugins</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => setTab('red4ext')}
            oncontextmenu={(e) => openSidebarContextMenu(e, 'red4ext', 'RED4ext Plugins')}
            title="Right-click for options"
            class="relative w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-medium transition cursor-pointer border"
            style="{currentTab === 'red4ext' ? 'background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
          >
            {#if currentTab === 'red4ext'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Folder class="h-4 w-4" style="{currentTab === 'red4ext' ? 'color: var(--theme-accent);' : ''}" />
              <span>RED4ext Plugins</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => setTab('redscript')}
            oncontextmenu={(e) => openSidebarContextMenu(e, 'redscript', 'R6\\Scripts')}
            title="Right-click for options"
            class="relative w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-medium transition cursor-pointer border"
            style="{currentTab === 'redscript' ? 'background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
          >
            {#if currentTab === 'redscript'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Folder class="h-4 w-4" style="{currentTab === 'redscript' ? 'color: var(--theme-accent);' : ''}" />
              <span>R6\Scripts</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => setTab('r6tweaks')}
            oncontextmenu={(e) => openSidebarContextMenu(e, 'r6tweaks', 'R6\\Tweaks')}
            title="Right-click for options"
            class="relative w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-medium transition cursor-pointer border"
            style="{currentTab === 'r6tweaks' ? 'background-color: var(--theme-card); border-color: var(--theme-border); color: var(--theme-text-primary);' : 'border-color: transparent; color: var(--theme-text-muted);'}"
          >
            {#if currentTab === 'r6tweaks'}
              <div class="absolute left-0 top-2 bottom-2 w-1 rounded-r-md shadow-[0_0_8px_var(--theme-accent)]" style="background-color: var(--theme-accent);"></div>
            {/if}
            <div class="flex items-center gap-3">
              <Folder class="h-4 w-4" style="{currentTab === 'r6tweaks' ? 'color: var(--theme-accent);' : ''}" />
              <span>R6\Tweaks</span>
            </div>
          </button>
        </nav>
      </div>

      <!-- Bottom Utility Dock -->
      <div class="px-4 pb-4 pt-3 border-t flex items-center justify-between shrink-0" style="border-color: var(--theme-border); background-color: var(--theme-bg);">
        <button
          type="button"
          onclick={() => setTab('history')}
          title="Mod History & Ledger"
          class="p-2 rounded transition cursor-pointer hover:brightness-125"
          style="{currentTab === 'history' ? 'color: var(--theme-accent); background-color: var(--theme-card);' : 'color: var(--theme-text-muted);'}"
        >
          <Clock class="h-4 w-4" />
        </button>

        {#if customPhotoPath}
          <button
            type="button"
            onclick={handleOpenPhotos}
            title="Cyberpunk 2077 Photos"
            class="p-2 rounded transition cursor-pointer hover:brightness-125"
            style="color: var(--theme-text-muted);"
          >
            <Camera class="h-4 w-4" />
          </button>
        {/if}

        <button
          type="button"
          onclick={() => setTab('help')}
          title="Help & Documentation"
          class="p-2 rounded transition cursor-pointer hover:brightness-125"
          style="{currentTab === 'help' ? 'color: var(--theme-accent); background-color: var(--theme-card);' : 'color: var(--theme-text-muted);'}"
        >
          <HelpCircle class="h-4 w-4" />
        </button>

        <button
          type="button"
          onclick={() => setTab('settings')}
          title="Settings"
          class="p-2 rounded transition cursor-pointer hover:brightness-125"
          style="{currentTab === 'settings' ? 'color: var(--theme-accent); background-color: var(--theme-card);' : 'color: var(--theme-text-muted);'}"
        >
          <Settings class="h-4 w-4" />
        </button>

        <button
          type="button"
          onclick={() => setTab('about')}
          title="About R4"
          class="p-2 rounded transition cursor-pointer hover:brightness-125"
          style="{currentTab === 'about' ? 'color: var(--theme-accent); background-color: var(--theme-card);' : 'color: var(--theme-text-muted);'}"
        >
          <Info class="h-4 w-4" />
        </button>
      </div>
    </div>
  </aside>

  <!-- Workspace Canvas -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden relative">
    
    <!-- Full-Canvas Hero Background Layer (Enhanced Vibrancy Wash) -->
    {#if heroBannerUrl}
      <div class="absolute inset-0 z-0 pointer-events-none overflow-hidden">
        <img 
          src={heroBannerUrl} 
          alt="" 
          onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'}
          class="absolute inset-0 w-full h-full object-cover object-[center_top] opacity-55 saturate-[1.15]" 
        />
        <!-- Multi-stop Gradient Protective Wash: Enhanced clarity at top transitioning to protective dark surface at bottom -->
        <div 
          class="absolute inset-0" 
          style="background: linear-gradient(to bottom, color-mix(in srgb, var(--theme-bg) 20%, transparent) 0%, color-mix(in srgb, var(--theme-bg) 65%, transparent) 40%, var(--theme-bg) 95%);"
        ></div>
      </div>
    {/if}

    <HeaderBar {gamePath} {currentTab} />

    <main class="flex-1 overflow-y-auto relative z-10 {['settings', 'help', 'about', 'history'].includes(currentTab) ? 'p-0' : 'p-6'}">
      
      <!-- Persistent Tab Views -->
      <div class={currentTab === 'archive' ? 'h-full' : 'hidden'}>
        <ArchiveView
          bind:archives
          {gamePath}
          scanReport={archiveReport}
          onScanRequested={refreshAll}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          onRouteToHistory={handleNavigateToHistory}
          {enableOrganizerView}
          {showCategoryLabelsInLoadOrder}
          onLabelsToggle={async (val) => {
            showCategoryLabelsInLoadOrder = val;
            try {
              const config = await loadConfigFromDisk();
              config.showCategoryLabelsInLoadOrder = val;
              await saveConfigToDisk(config);
            } catch (err) {
              console.error('Failed to save label toggle state:', err);
            }
          }}
          {currentTab}
          bind:globalSelectedPath
        />
      </div>

      <div class={currentTab === 'cet' ? 'h-full' : 'hidden'}>
        <CetView
          bind:plugins={cetPlugins}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          onRouteToHistory={handleNavigateToHistory}
          {currentTab}
          bind:globalSelectedPath
        />
      </div>

      <div class={currentTab === 'red4ext' ? 'h-full' : 'hidden'}>
        <Red4extView
          bind:plugins={red4extPlugins}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          onRouteToHistory={handleNavigateToHistory}
          {currentTab}
          bind:globalSelectedPath
        />
      </div>

      <div class={currentTab === 'redscript' ? 'h-full' : 'hidden'}>
        <RedscriptView
          bind:packages={redscriptPackages}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          onRouteToHistory={handleNavigateToHistory}
          {currentTab}
          bind:globalSelectedPath
        />
      </div>

      <div class={currentTab === 'r6tweaks' ? 'h-full' : 'hidden'}>
        <R6TweaksView
          bind:tweaks={r6tweaks}
          {gamePath}
          onStateChanged={refreshCountsOnly}
          {targetHighlightMod}
          onRouteToHistory={handleNavigateToHistory}
          {currentTab}
          bind:globalSelectedPath
        />
      </div>

      <!-- Full-Screen Views -->
      <div class={currentTab === 'history' ? 'h-full' : 'hidden'}>
        <HistoryView
          {gamePath}
          bind:newModsHours
          {targetHighlightMod}
          onNavigateToMod={handleNavigateToMod}
          {currentTab}
          {scanResult}
        />
      </div>

      <div class={currentTab === 'settings' ? 'h-full' : 'hidden'}>
        <SettingsView 
          bind:gamePath 
          onRefresh={refreshAll} 
          onSettingsSaved={async () => {
            const config = await loadConfigFromDisk();
            customPhotoPath = config.customPhotoPath || '';
            enableOrganizerView = config.enableOrganizerView || false;
            showCategoryLabelsInLoadOrder = config.showCategoryLabelsInLoadOrder ?? true;
          }}
        />
      </div>

      <div class={currentTab === 'help' ? 'h-full' : 'hidden'}>
        <HelpView />
      </div>

      <div class={currentTab === 'about' ? 'h-full' : 'hidden'}>
        <AboutView />
      </div>
    </main>

    <!-- Bottom Status Bar (Restructured Telemetry Cluster - Point 6) -->
    <footer class="h-8 border-t flex items-center justify-between px-4 shrink-0 text-[11px] font-mono select-none relative z-10" style="background-color: var(--theme-surface); border-color: var(--theme-border); color: var(--theme-text-muted);">
      
      <!-- Left: Active Cursor Hover Path (Empty when idle) -->
      <div class="flex-1 min-w-0 pr-4 truncate">
        {#if globalSelectedPath && globalSelectedPath !== 'Ready'}
          <span class="truncate opacity-80" style="color: var(--theme-text-muted);" title={globalSelectedPath}>
            {globalSelectedPath}
          </span>
        {/if}
      </div>

      <!-- Right: Unified Telemetry Cluster (Right-Justified with Dividers) -->
      <div class="flex items-center gap-3 shrink-0">
        <!-- Sync / Rescan Button -->
        <div class="flex items-center gap-1.5">
          {#if isLoading}
            <RefreshCw class="h-3 w-3 animate-spin" style="color: var(--theme-accent);" />
            <span style="color: var(--theme-accent);">Syncing...</span>
          {:else}
            <button type="button" onclick={refreshAll} class="hover:brightness-125 transition cursor-pointer" title="Force Rescan">
              <RefreshCw class="h-3 w-3" />
            </button>
          {/if}
        </div>

        <!-- Vertical Divider -->
        <div class="h-3 w-px" style="background-color: var(--theme-border);"></div>

        <!-- Active Mod Metrics -->
        {#if ['archive', 'cet', 'red4ext', 'redscript', 'r6tweaks'].includes(currentTab)}
          <div class="flex items-center gap-1">
            <span>Active:</span>
            <span class="font-bold" style="color: var(--theme-text-primary);">{activeCount}</span>
            <span>of {totalCount}</span>
          </div>

          <!-- Vertical Divider -->
          <div class="h-3 w-px" style="background-color: var(--theme-border);"></div>
        {/if}

        <!-- Connection Integrity (Right-Justified) -->
        <div class="flex items-center gap-1.5">
          {#if !gamePath}
            <AlertTriangle class="h-3.5 w-3.5" style="color: var(--color-warning);" />
            <span class="font-bold uppercase tracking-wider" style="color: var(--color-warning);">Setup Required</span>
          {:else if !scanResult?.is_valid_game_path}
            <AlertCircle class="h-3.5 w-3.5" style="color: var(--color-danger);" />
            <span class="font-bold uppercase tracking-wider" style="color: var(--color-danger);">Disconnected</span>
          {:else}
            <CheckCircle2 class="h-3.5 w-3.5" style="color: var(--color-success);" />
            <span class="font-bold uppercase tracking-wider" style="color: var(--color-success);">Connected</span>
          {/if}
        </div>
      </div>
    </footer>
  </div>
</div>
