<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl, revealItemInDir } from '@tauri-apps/plugin-opener';
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
    R6TweaksItem,
    LedgerEntry
  } from '$lib/types';

  import HeaderBar from '$lib/components/HeaderBar.svelte';
  import ArchiveView from '$lib/components/ArchiveView.svelte';
  import CetView from '$lib/components/CetView.svelte';
  import Red4extView from '$lib/components/Red4extView.svelte';
  import RedscriptView from '$lib/components/RedscriptView.svelte';
  import R6TweaksView from '$lib/components/R6TweaksView.svelte';

  import {
    Home,
    Archive,
    Cpu,
    Puzzle,
    FileCode2,
    Sliders,
    Play,
    CheckCircle2,
    ArrowUpRight,
    Clock,
    ChevronDown,
    ChevronRight,
    ArrowUpDown,
    FolderSearch,
    ExternalLink,
    Copy,
    Check
  } from 'lucide-svelte';

  type TabType = 'home' | 'archive' | 'cet' | 'red4ext' | 'redscript' | 'r6tweaks';

  // Always boot cleanly to the Home dashboard
  let currentTab = $state<TabType>('home');

  async function setTab(tab: TabType) {
    currentTab = tab;
    try {
      const config = await loadConfigFromDisk();
      config.activeTab = tab;
      await saveConfigToDisk(config);
    } catch (err) {
      // non-critical
    }
  }

  let gamePath = $state('G:\\SteamLibrary\\steamapps\\common\\Cyberpunk 2077');
  let isLoading = $state(false);
  let isLaunching = $state(false);

  let scanResult = $state<ScanResult | null>(null);
  let archiveReport = $state<ArchiveScanReport | null>(null);
  let archives = $state<ArchiveItem[]>([]);
  let cetPlugins = $state<CetPluginItem[]>([]);
  let red4extPlugins = $state<Red4extPluginItem[]>([]);
  let redscriptPackages = $state<RedScriptItem[]>([]);
  let r6tweaks = $state<R6TweaksItem[]>([]);

  // Recently Added Mods State
  let recentMods = $state<LedgerEntry[]>([]);
  let showRecentMods = $state(false);
  let recentDaysThreshold = $state(30);
  let sortColumn = $state<'date' | 'name' | 'type'>('date');
  let sortAscending = $state(false);
  
  // Global Highlight Signal
  let targetHighlightMod = $state<string | null>(null);

  let recentContextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    entry: LedgerEntry | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    entry: null
  });
  let copiedFeedback = $state(false);

  let sortedRecentMods = $derived.by(() => {
    let arr = [...recentMods];
    arr.sort((a, b) => {
      if (sortColumn === 'date') {
        return sortAscending ? a.first_seen - b.first_seen : b.first_seen - a.first_seen;
      } else if (sortColumn === 'name') {
        return sortAscending ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
      } else if (sortColumn === 'type') {
        return sortAscending ? a.mod_type.localeCompare(b.mod_type) : b.mod_type.localeCompare(a.mod_type);
      }
      return 0;
    });
    return arr;
  });

  function formatTimestamp(ts: number) {
    if (!ts) return 'Unknown';
    const d = new Date(ts);
    return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function handleSort(column: 'date' | 'name' | 'type') {
    if (sortColumn === column) {
      sortAscending = !sortAscending;
    } else {
      sortColumn = column;
      sortAscending = column === 'name' || column === 'type'; // Default ascending for text, descending for date
    }
  }

  async function loadRecentMods() {
    if (!gamePath) return;
    try {
      recentMods = await invoke<LedgerEntry[]>('get_recent_mods', { gamePath, days: recentDaysThreshold });
    } catch (err) {
      console.error('Failed to load recent mods:', err);
    }
  }

  async function handleThresholdChange() {
    if (recentDaysThreshold < 1) recentDaysThreshold = 1;
    if (recentDaysThreshold > 365) recentDaysThreshold = 365;
    await loadRecentMods();
    try {
      const config = await loadConfigFromDisk();
      config.recentDaysThreshold = recentDaysThreshold;
      await saveConfigToDisk(config);
    } catch (err) {
      console.error('Failed to save threshold:', err);
    }
  }

  function openRecentContextMenu(event: MouseEvent, entry: LedgerEntry) {
    event.preventDefault();
    event.stopPropagation();
    copiedFeedback = false;

    const menuWidth = 200;
    const menuHeight = 120;
    const posX = (event.clientX + menuWidth > window.innerWidth) ? (window.innerWidth - menuWidth - 10) : event.clientX;
    const posY = (event.clientY + menuHeight > window.innerHeight) ? (window.innerHeight - menuHeight - 10) : event.clientY;

    recentContextMenu = { visible: true, x: posX, y: posY, entry };
  }

  function closeRecentContextMenu() {
    if (recentContextMenu.visible) {
      recentContextMenu.visible = false;
      recentContextMenu.entry = null;
      copiedFeedback = false;
    }
  }

  async function handleRecentShowInExplorer() {
    if (!recentContextMenu.entry) return;
    try {
      await revealItemInDir(recentContextMenu.entry.path);
    } catch (err) {
      console.error('Failed to reveal file in explorer:', err);
    }
    closeRecentContextMenu();
  }

  async function handleRecentShowInApp() {
    if (!recentContextMenu.entry) return;
    const entry = recentContextMenu.entry;
    
    const tabMap: Record<string, TabType> = {
      'archive': 'archive',
      'cet': 'cet',
      'red4ext': 'red4ext',
      'redscript': 'redscript',
      'r6tweaks': 'r6tweaks'
    };
    
    const targetTab = tabMap[entry.mod_type];
    if (targetTab) {
      await setTab(targetTab);
      targetHighlightMod = entry.name;
      // Clear the signal shortly after so it can be re-triggered later if needed
      setTimeout(() => { targetHighlightMod = null; }, 100);
      try {
        await navigator.clipboard.writeText(entry.name);
      } catch (err) {}
    }
    closeRecentContextMenu();
  }

  async function handleRecentCopyName() {
    if (!recentContextMenu.entry) return;
    try {
      await navigator.clipboard.writeText(recentContextMenu.entry.name);
      copiedFeedback = true;
      setTimeout(() => {
        closeRecentContextMenu();
      }, 400);
    } catch (err) {
      console.error('Failed to copy name:', err);
      closeRecentContextMenu();
    }
  }

  async function openExternalLink(url: string) {
    if (!url) return;
    try {
      await openUrl(url);
    } catch (err) {
      console.error(`Failed to open URL ${url}:`, err);
    }
  }

  async function handleLaunchGame() {
    if (!gamePath || isLaunching) return;
    isLaunching = true;
    try {
      await invoke('launch_game', { gamePath });
    } catch (err) {
      console.error('Failed to launch game:', err);
    } finally {
      setTimeout(() => { isLaunching = false; }, 2500);
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
      console.error(`Failed to load data for tab ${tab}:`, err);
    }
  }

  async function refreshAll() {
    if (!gamePath) return;
    isLoading = true;
    try {
      scanResult = await invoke<ScanResult>('scan_game_directory', { gamePath });
      await loadTabData('archive');
      await loadTabData('cet');
      await loadTabData('red4ext');
      await loadTabData('redscript');
      await loadTabData('r6tweaks');
      await loadRecentMods();
    } catch (err) {
      console.error('Failed to scan game environment:', err);
    } finally {
      isLoading = false;
    }
  }

  async function refreshCountsOnly() {
    if (!gamePath) return;
    try {
      scanResult = await invoke<ScanResult>('scan_game_directory', { gamePath });
      await loadRecentMods();
    } catch (err) {
      console.error('Failed to refresh counts:', err);
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
      }
      if (config.recentDaysThreshold !== undefined) {
        recentDaysThreshold = config.recentDaysThreshold;
      }
    }).catch(err => {
      console.error('Config hydration error:', err);
    }).finally(() => {
      refreshAll().finally(() => {
        dismissSplash();
      });
    });

    invoke('start_directory_watcher', { gamePath }).catch(() => {});
    const unlisten = listen('directory-changed', () => {
      refreshAll();
    });

    return () => {
      unlisten.then(f => f());
    };
  });
</script>

<svelte:window onclick={closeRecentContextMenu} oncontextmenu={(e) => e.preventDefault()} />

<!-- Recent Mods Context Menu -->
{#if recentContextMenu.visible && recentContextMenu.entry}
  <div
    class="fixed z-50 w-48 rounded-md border border-nvidia-border bg-nvidia-card py-1 shadow-2xl shadow-black/90 text-xs select-none backdrop-blur-md"
    style="left: {recentContextMenu.x}px; top: {recentContextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.preventDefault()}
  >
    <div class="px-3 py-1.5 border-b border-nvidia-border/60 bg-nvidia-surface/40 flex items-center gap-2">
      <span class="font-mono text-[11px] font-bold text-nvidia-text-primary truncate" title={recentContextMenu.entry.name}>
        {recentContextMenu.entry.name}
      </span>
    </div>
    <div class="p-1 space-y-0.5">
      <button
        type="button"
        onclick={handleRecentShowInExplorer}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <FolderSearch class="h-3.5 w-3.5 text-nvidia-accent group-hover:brightness-110" />
        <span>Show in Explorer</span>
      </button>
      <button
        type="button"
        onclick={handleRecentShowInApp}
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
        title="Switches to the correct tab and copies the mod name to your clipboard"
      >
        <ExternalLink class="h-3.5 w-3.5 text-cyan-400" />
        <span>Show in App</span>
      </button>
      <button
        type="button"
        onclick={handleRecentCopyName}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-nvidia-surface text-nvidia-text-primary transition text-left cursor-pointer group"
      >
        <div class="flex items-center gap-2.5">
          {#if copiedFeedback}
            <Check class="h-3.5 w-3.5 text-nvidia-accent" />
            <span class="text-nvidia-accent font-medium">Copied!</span>
          {:else}
            <Copy class="h-3.5 w-3.5 text-nvidia-text-muted group-hover:text-nvidia-text-primary" />
            <span>Copy Name</span>
          {/if}
        </div>
      </button>
    </div>
  </div>
{/if}

<div class="flex h-screen w-screen overflow-hidden bg-nvidia-bg text-nvidia-text-primary font-sans">
  <!-- Left Navigation Sidebar -->
  <aside class="w-64 border-r border-nvidia-border flex flex-col justify-between bg-nvidia-surface/40 select-none shrink-0">
    <div class="flex flex-col h-full">
      <div class="h-14 border-b border-nvidia-border px-5 flex items-center gap-3 shrink-0">
        <div class="h-6 w-6 rounded bg-nvidia-accent flex items-center justify-center text-black font-black text-sm shadow-sm">
          R4
        </div>
        <div>
          <h1 class="text-xs font-bold tracking-wider uppercase text-nvidia-text-primary">R4 Mod Toolbox</h1>
          <p class="text-[10px] text-nvidia-text-muted font-mono truncate max-w-[140px]" title="Advanced Mod Manager">
            ADVANCED MOD MANAGER
          </p>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto">
        <nav class="p-3 space-y-1">
          <button
            type="button"
            onclick={() => setTab('home')}
            class="w-full flex items-center gap-3 px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'home' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <Home class="h-4 w-4" />
            <span>Home</span>
          </button>

          <button
            type="button"
            onclick={() => setTab('archive')}
            class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'archive' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <div class="flex items-center gap-3">
              <Archive class="h-4 w-4" />
              <span>Archive</span>
            </div>
            {#if scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
                {scanResult.archive_active}/{scanResult.archive_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('cet')}
            class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'cet' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <div class="flex items-center gap-3">
              <Cpu class="h-4 w-4" />
              <span>CET Mods</span>
            </div>
            {#if scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
                {scanResult.cet_active}/{scanResult.cet_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('red4ext')}
            class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'red4ext' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <div class="flex items-center gap-3">
              <Puzzle class="h-4 w-4" />
              <span>RED4ext</span>
            </div>
            {#if scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
                {scanResult.red4ext_active}/{scanResult.red4ext_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('redscript')}
            class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'redscript' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <div class="flex items-center gap-3">
              <FileCode2 class="h-4 w-4" />
              <span>R6\Scripts</span>
            </div>
            {#if scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
                {scanResult.redscript_active}/{scanResult.redscript_total}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => setTab('r6tweaks')}
            class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'r6tweaks' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
          >
            <div class="flex items-center gap-3">
              <Sliders class="h-4 w-4" />
              <span>R6\Tweaks</span>
            </div>
            {#if scanResult}
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
                {scanResult.r6tweaks_active}/{scanResult.r6tweaks_total}
              </span>
            {/if}
          </button>
        </nav>
      </div>
    </div>

    <!-- Lower Left Launch Game & Developer Tile -->
    <div class="p-3 border-t border-nvidia-border bg-nvidia-surface/30 space-y-2.5 shrink-0">
      <button
        type="button"
        onclick={handleLaunchGame}
        disabled={isLaunching || !scanResult?.is_valid_game_path}
        class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition shadow-sm disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
      >
        <Play class="h-3.5 w-3.5 fill-black" />
        <span>{isLaunching ? 'Starting Game...' : 'Launch Game'}</span>
      </button>

      <!-- Mi55ionary Developer Tile -->
      <div class="rounded-lg border border-nvidia-border/70 bg-nvidia-surface/70 p-2 flex items-center justify-between gap-2 shadow-xs select-none">
        <div class="flex items-center gap-2 min-w-0">
          <!-- Stylized Hand-Drawn Heart Avatar -->
          <div class="h-6 w-6 rounded bg-amber-500/10 border border-amber-500/30 flex items-center justify-center shrink-0">
            <svg viewBox="0 0 100 100" class="h-4 w-4 drop-shadow-sm" fill="none" stroke="#e08528" stroke-width="9" stroke-linecap="round" stroke-linejoin="round">
              <path d="M 48 34 C 44 20 22 10 13 26 C 3 44 18 68 49 92 C 80 68 97 44 87 22 C 77 0 54 8 48 34 Z" />
            </svg>
          </div>
          <div class="min-w-0 flex flex-col">
            <span class="text-[9px] text-nvidia-text-muted uppercase tracking-wider font-semibold leading-tight">Dev</span>
            <span class="text-[11px] font-mono font-bold text-nvidia-text-primary truncate leading-tight" title="Mi55ionary">
              Mi55ionary
            </span>
          </div>
        </div>

        <div class="flex items-center gap-1 shrink-0">
          <button
            type="button"
            onclick={() => openExternalLink('https://www.nexusmods.com/profile/Mi55ionary')}
            class="px-1.5 py-0.5 rounded bg-nvidia-card hover:bg-amber-500/20 text-[10px] font-mono font-bold text-amber-400 border border-nvidia-border hover:border-amber-500/50 transition cursor-pointer"
            title="View Mi55ionary on Nexus Mods"
          >
            Nexus
          </button>

          <button
            type="button"
            onclick={() => openExternalLink('https://github.com/TheMissionary/R4-Mod-Toolbox')}
            class="px-1.5 py-0.5 rounded bg-nvidia-card hover:bg-nvidia-surface text-[10px] font-mono text-nvidia-text-muted hover:text-nvidia-text-primary border border-nvidia-border transition cursor-pointer"
            title="View GitHub Repository"
          >
            GitHub
          </button>
        </div>
      </div>
    </div>
  </aside>

  <!-- Workspace Canvas -->
  <div class="flex-1 flex flex-col h-full overflow-hidden relative">
    <HeaderBar bind:gamePath onRefresh={refreshAll} {isLoading} />

    <main class="flex-1 p-6 overflow-y-auto">
      <!-- Persistent Tab: Home -->
      <div class={currentTab === 'home' ? 'space-y-6' : 'hidden'}>
        <div class="p-6 rounded-lg border border-nvidia-border bg-nvidia-card relative overflow-hidden">
          <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
            <CheckCircle2 class="h-4 w-4" />
            <span>ENGINE CONNECTED</span>
          </div>
          <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">R4 Management Engine</h2>
          <p class="text-xs text-nvidia-text-muted max-w-xl">
            Master your mod conflicts with real-time archive resolution. Organize your workspace using custom categories, drag-and-drop load ordering, and seamless management across all mod frameworks.
          </p>
        </div>

        <!-- 6-Column Grid for 5 Cards (3 Top, 2 Bottom) -->
        <div class="grid grid-cols-1 md:grid-cols-6 gap-4">
          <!-- Card 1: Archive Mods -->
          <button
            type="button"
            onclick={() => setTab('archive')}
            class="col-span-1 md:col-span-2 p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-xs font-bold uppercase tracking-wider text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">ARCHIVE MODS</span>
              <div class="flex items-center gap-1 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">
                <Archive class="h-4 w-4" />
                <ArrowUpRight class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
            <div>
              <div class="flex items-baseline gap-1.5">
                <span class="text-2xl font-bold font-mono text-nvidia-text-primary group-hover:text-nvidia-accent transition-colors">{scanResult?.archive_active ?? 0}</span>
                <span class="text-xs font-mono text-nvidia-text-muted">of {scanResult?.archive_total ?? 0}</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5 truncate" title="archive/pc/mod">archive/pc/mod</p>
            </div>
          </button>

          <!-- Card 2: CET Plugins -->
          <button
            type="button"
            onclick={() => setTab('cet')}
            class="col-span-1 md:col-span-2 p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-xs font-bold uppercase tracking-wider text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">CET PLUGINS</span>
              <div class="flex items-center gap-1 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">
                <Cpu class="h-4 w-4" />
                <ArrowUpRight class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
            <div>
              <div class="flex items-baseline gap-1.5">
                <span class="text-2xl font-bold font-mono text-nvidia-text-primary group-hover:text-nvidia-accent transition-colors">{scanResult?.cet_active ?? 0}</span>
                <span class="text-xs font-mono text-nvidia-text-muted">of {scanResult?.cet_total ?? 0}</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5 truncate" title="bin/x64/plugins/cyber_engine_tweaks">bin/x64/plugins/cyber_engine_tweaks</p>
            </div>
          </button>

          <!-- Card 3: RED4ext Plugins -->
          <button
            type="button"
            onclick={() => setTab('red4ext')}
            class="col-span-1 md:col-span-2 p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-xs font-bold uppercase tracking-wider text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">RED4EXT PLUGINS</span>
              <div class="flex items-center gap-1 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">
                <Puzzle class="h-4 w-4" />
                <ArrowUpRight class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
            <div>
              <div class="flex items-baseline gap-1.5">
                <span class="text-2xl font-bold font-mono text-nvidia-text-primary group-hover:text-nvidia-accent transition-colors">{scanResult?.red4ext_active ?? 0}</span>
                <span class="text-xs font-mono text-nvidia-text-muted">of {scanResult?.red4ext_total ?? 0}</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5 truncate" title="red4ext/plugins">red4ext/plugins</p>
            </div>
          </button>

          <!-- Card 4: R6\SCRIPTS -->
          <button
            type="button"
            onclick={() => setTab('redscript')}
            class="col-span-1 md:col-span-3 p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-xs font-bold uppercase tracking-wider text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">R6\SCRIPTS</span>
              <div class="flex items-center gap-1 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">
                <FileCode2 class="h-4 w-4" />
                <ArrowUpRight class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
            <div>
              <div class="flex items-baseline gap-1.5">
                <span class="text-2xl font-bold font-mono text-nvidia-text-primary group-hover:text-nvidia-accent transition-colors">{scanResult?.redscript_active ?? 0}</span>
                <span class="text-xs font-mono text-nvidia-text-muted">of {scanResult?.redscript_total ?? 0}</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5 truncate" title="r6/scripts">r6/scripts</p>
            </div>
          </button>

          <!-- Card 5: R6\TWEAKS -->
          <button
            type="button"
            onclick={() => setTab('r6tweaks')}
            class="col-span-1 md:col-span-3 p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-xs font-bold uppercase tracking-wider text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">R6\TWEAKS</span>
              <div class="flex items-center gap-1 text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">
                <Sliders class="h-4 w-4" />
                <ArrowUpRight class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
            <div>
              <div class="flex items-baseline gap-1.5">
                <span class="text-2xl font-bold font-mono text-nvidia-text-primary group-hover:text-nvidia-accent transition-colors">{scanResult?.r6tweaks_active ?? 0}</span>
                <span class="text-xs font-mono text-nvidia-text-muted">of {scanResult?.r6tweaks_total ?? 0}</span>
              </div>
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5 truncate" title="r6/tweaks">r6/tweaks</p>
            </div>
          </button>
        </div>

        <!-- Recently Added Mods Ledger -->
        <div class="rounded-lg border border-nvidia-border bg-nvidia-card overflow-hidden flex flex-col shadow-xs">
          <div
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showRecentMods = !showRecentMods; }}
            onclick={() => showRecentMods = !showRecentMods}
            class="px-4 py-3 bg-nvidia-surface/60 hover:bg-nvidia-surface/90 flex items-center justify-between cursor-pointer select-none transition-colors"
          >
            <div class="flex items-center gap-2.5">
              <Clock class="h-4 w-4 text-nvidia-accent" />
              <span class="text-xs font-bold text-nvidia-text-primary uppercase tracking-wider">Recently Added Mods</span>
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-bg border border-nvidia-border text-nvidia-text-muted">
                {recentMods.length}
              </span>
            </div>
            <div class="flex items-center gap-4" onclick={(e) => e.stopPropagation()} role="presentation">
              <div class="flex items-center gap-2">
                <span class="text-[10px] text-nvidia-text-muted uppercase tracking-wider font-semibold">Days:</span>
                <input
                  type="number"
                  min="1"
                  max="365"
                  bind:value={recentDaysThreshold}
                  onchange={handleThresholdChange}
                  class="w-16 px-2 py-1 bg-nvidia-bg border border-nvidia-border rounded text-xs text-nvidia-text-primary focus:outline-none focus:border-nvidia-accent text-center font-mono"
                  title="Number of days to track"
                />
              </div>
              <button
                type="button"
                onclick={() => showRecentMods = !showRecentMods}
                class="p-1 rounded hover:bg-nvidia-surface text-nvidia-text-muted hover:text-nvidia-text-primary transition cursor-pointer"
              >
                {#if showRecentMods}
                  <ChevronDown class="h-4 w-4" />
                {:else}
                  <ChevronRight class="h-4 w-4" />
                {/if}
              </button>
            </div>
          </div>

          {#if showRecentMods}
            <div class="border-t border-nvidia-border bg-nvidia-bg/50 flex flex-col max-h-96">
              <!-- Table Header -->
              <div class="grid grid-cols-[160px_120px_1fr] gap-4 px-4 py-2 border-b border-nvidia-border/60 bg-nvidia-surface/40 text-[10px] font-bold text-nvidia-text-muted uppercase tracking-wider select-none shrink-0">
                <div class="flex items-center gap-1 cursor-pointer hover:text-nvidia-text-primary transition" onclick={() => handleSort('date')} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') handleSort('date'); }}>
                  <span>Date Added</span>
                  <ArrowUpDown class="h-3 w-3 {sortColumn === 'date' ? 'text-nvidia-accent' : 'opacity-50'}" />
                </div>
                <div class="flex items-center gap-1 cursor-pointer hover:text-nvidia-text-primary transition" onclick={() => handleSort('type')} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') handleSort('type'); }}>
                  <span>Mod Type</span>
                  <ArrowUpDown class="h-3 w-3 {sortColumn === 'type' ? 'text-nvidia-accent' : 'opacity-50'}" />
                </div>
                <div class="flex items-center gap-1 cursor-pointer hover:text-nvidia-text-primary transition" onclick={() => handleSort('name')} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') handleSort('name'); }}>
                  <span>File Name</span>
                  <ArrowUpDown class="h-3 w-3 {sortColumn === 'name' ? 'text-nvidia-accent' : 'opacity-50'}" />
                </div>
              </div>

              <!-- Table Body -->
              <div class="overflow-y-auto flex-1 p-1 space-y-0.5">
                {#if sortedRecentMods.length === 0}
                  <div class="p-6 text-center text-xs text-nvidia-text-muted font-mono">
                    No new mods detected in the last {recentDaysThreshold} days.
                  </div>
                {:else}
                  {#each sortedRecentMods as entry (entry.path)}
                    <div
                      oncontextmenu={(e) => openRecentContextMenu(e, entry)}
                      class="grid grid-cols-[160px_120px_1fr] gap-4 px-3 rounded hover:bg-nvidia-surface/80 transition items-center cursor-context-menu density-row"
                    >
                      <div class="text-[11px] font-mono text-nvidia-text-muted">
                        {formatTimestamp(entry.first_seen)}
                      </div>
                      <div>
                        <span class="text-[9px] font-mono font-bold uppercase px-1.5 py-0.5 rounded border bg-nvidia-surface border-nvidia-border text-nvidia-text-primary">
                          {entry.mod_type}
                        </span>
                      </div>
                      <div class="text-xs font-mono text-nvidia-text-primary truncate" title={entry.name}>
                        {entry.name}
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Persistent Tab Views -->
      <div class={currentTab === 'archive' ? 'h-full' : 'hidden'}>
        <ArchiveView bind:archives {gamePath} scanReport={archiveReport} onScanRequested={refreshAll} onStateChanged={refreshCountsOnly} {targetHighlightMod} />
      </div>

      <div class={currentTab === 'cet' ? 'h-full' : 'hidden'}>
        <CetView bind:plugins={cetPlugins} {gamePath} onStateChanged={refreshCountsOnly} {targetHighlightMod} />
      </div>

      <div class={currentTab === 'red4ext' ? 'h-full' : 'hidden'}>
        <Red4extView bind:plugins={red4extPlugins} {gamePath} onStateChanged={refreshCountsOnly} {targetHighlightMod} />
      </div>

      <div class={currentTab === 'redscript' ? 'h-full' : 'hidden'}>
        <RedscriptView bind:packages={redscriptPackages} {gamePath} onStateChanged={refreshCountsOnly} {targetHighlightMod} />
      </div>

      <div class={currentTab === 'r6tweaks' ? 'h-full' : 'hidden'}>
        <R6TweaksView bind:tweaks={r6tweaks} {gamePath} onStateChanged={refreshCountsOnly} {targetHighlightMod} />
      </div>
    </main>
  </div>
</div>
