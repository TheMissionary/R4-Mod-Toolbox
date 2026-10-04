<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl } from '@tauri-apps/plugin-opener';
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
    RedScriptItem
  } from '$lib/types';

  import HeaderBar from '$lib/components/HeaderBar.svelte';
  import ArchiveView from '$lib/components/ArchiveView.svelte';
  import CetView from '$lib/components/CetView.svelte';
  import Red4extView from '$lib/components/Red4extView.svelte';
  import RedscriptView from '$lib/components/RedscriptView.svelte';

  import {
    Home,
    Archive,
    Cpu,
    Puzzle,
    FileCode2,
    Play,
    CheckCircle2,
    ArrowUpRight
  } from 'lucide-svelte';

  type TabType = 'home' | 'archive' | 'cet' | 'red4ext' | 'redscript';

  function getInitialTab(): TabType {
    if (typeof sessionStorage !== 'undefined') {
      const saved = sessionStorage.getItem('cp2077_active_tab');
      if (saved === 'home' || saved === 'archive' || saved === 'cet' || saved === 'red4ext' || saved === 'redscript') {
        return saved as TabType;
      }
    }
    return 'home';
  }

  let currentTab = $state<TabType>(getInitialTab());

  async function setTab(tab: TabType) {
    currentTab = tab;
    if (typeof sessionStorage !== 'undefined') {
      sessionStorage.setItem('cp2077_active_tab', tab);
    }
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

  const coreUrls: Record<string, string> = {
    CP77: 'https://support.cdprojektred.com/en/cyberpunk/pc/sp-technical/issue/2734/patch-2-31-download-now',
    CET: 'https://www.nexusmods.com/cyberpunk2077/mods/107',
    RED4ext: 'https://www.nexusmods.com/cyberpunk2077/mods/2380',
    ArchiveXL: 'https://www.nexusmods.com/cyberpunk2077/mods/4198',
    Codeware: 'https://www.nexusmods.com/cyberpunk2077/mods/7780',
    TweakXL: 'https://www.nexusmods.com/cyberpunk2077/mods/4197'
  };

  async function openDependencyLink(key: string) {
    const url = coreUrls[key];
    if (!url) return;
    try {
      await openUrl(url);
    } catch (err) {
      console.error(`Failed to open URL for ${key}:`, err);
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
      if (typeof sessionStorage !== 'undefined' && !sessionStorage.getItem('cp2077_active_tab')) {
        if (config.activeTab) {
          currentTab = config.activeTab as TabType;
        }
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

<div class="flex h-screen w-screen overflow-hidden bg-nvidia-bg text-nvidia-text-primary font-sans">
  <!-- Left Navigation Sidebar -->
  <aside class="w-64 border-r border-nvidia-border flex flex-col justify-between bg-nvidia-surface/40 select-none shrink-0">
    <div>
      <div class="h-14 border-b border-nvidia-border px-5 flex items-center gap-3">
        <div class="h-6 w-6 rounded bg-nvidia-accent flex items-center justify-center text-black font-black text-sm shadow-sm">
          R4
        </div>
        <div>
          <h1 class="text-xs font-bold tracking-wider uppercase text-nvidia-text-primary">RED4 Mod Toolbox</h1>
          <p class="text-[10px] text-nvidia-text-muted font-mono truncate max-w-[140px]" title="Mod Load Order and Conflict Resolver">
            CONFLICT RESOLVER
          </p>
        </div>
      </div>

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

        <!-- Renamed Category: Redscript\R6 (Request 4) -->
        <button
          type="button"
          onclick={() => setTab('redscript')}
          class="w-full flex items-center justify-between px-3 py-2 rounded text-xs font-medium transition cursor-pointer {currentTab === 'redscript' ? 'bg-nvidia-card text-nvidia-accent font-semibold shadow-xs border border-nvidia-border' : 'text-nvidia-text-muted hover:bg-nvidia-surface hover:text-nvidia-text-primary'}"
        >
          <div class="flex items-center gap-3">
            <FileCode2 class="h-4 w-4" />
            <span>Redscript\R6</span>
          </div>
          {#if scanResult}
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-nvidia-surface text-nvidia-text-muted">
              {scanResult.redscript_active}/{scanResult.redscript_total}
            </span>
          {/if}
        </button>
      </nav>
    </div>

    <!-- Lower Left Launch Game & Status -->
    <div class="p-3 border-t border-nvidia-border bg-nvidia-surface/30 space-y-3 shrink-0">
      <button
        type="button"
        onclick={handleLaunchGame}
        disabled={isLaunching || !scanResult?.is_valid_game_path}
        class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded bg-nvidia-accent hover:brightness-105 text-black font-semibold text-xs transition shadow-sm disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
      >
        <Play class="h-3.5 w-3.5 fill-black" />
        <span>{isLaunching ? 'Starting Game...' : 'Launch Game'}</span>
      </button>

      <div class="space-y-1 text-xs font-mono select-none">
        <button
          type="button"
          onclick={() => openDependencyLink('CP77')}
          class="w-full flex items-center justify-between py-0.5 px-1 rounded hover:bg-nvidia-surface/80 group transition text-left cursor-pointer"
        >
          <span class="text-nvidia-text-muted group-hover:text-nvidia-text-primary transition">CP77</span>
          <span class="text-nvidia-accent font-semibold">{scanResult?.game_version || '2.31'}</span>
        </button>
      </div>
    </div>
  </aside>

  <!-- Workspace Canvas -->
  <div class="flex-1 flex flex-col h-full overflow-hidden">
    <HeaderBar bind:gamePath onRefresh={refreshAll} {isLoading} />

    <main class="flex-1 p-6 overflow-y-auto">
      <!-- Persistent Tab: Home -->
      <div class={currentTab === 'home' ? 'space-y-6' : 'hidden'}>
        <div class="p-6 rounded-lg border border-nvidia-border bg-nvidia-card relative overflow-hidden">
          <div class="flex items-center gap-2 text-xs font-mono text-nvidia-accent mb-2">
            <CheckCircle2 class="h-4 w-4" />
            <span>ENGINE CONNECTED</span>
          </div>
          <h2 class="text-xl font-bold text-nvidia-text-primary mb-1">RED4 Conflict Engine</h2>
          <p class="text-xs text-nvidia-text-muted max-w-xl">
            Inspect loose load orders, monitor overwrite priority conflicts across .archive, CET, and RedScript mod trees.
          </p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <!-- Card 1: Archive Mods -->
          <button
            type="button"
            onclick={() => setTab('archive')}
            class="p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-[10px] font-mono uppercase text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">ARCHIVE MODS</span>
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
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5">archive/pc/mod</p>
            </div>
          </button>

          <!-- Card 2: CET Plugins -->
          <button
            type="button"
            onclick={() => setTab('cet')}
            class="p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-[10px] font-mono uppercase text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">CET PLUGINS</span>
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
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5">bin/x64/plugins/cyber_engine_tweaks</p>
            </div>
          </button>

          <!-- Card 3: RED4ext Plugins -->
          <button
            type="button"
            onclick={() => setTab('red4ext')}
            class="p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-[10px] font-mono uppercase text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">RED4EXT PLUGINS</span>
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
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5">red4ext/plugins</p>
            </div>
          </button>

          <!-- Card 4: Renamed to REDSCRIPT\R6 (Request 4) -->
          <button
            type="button"
            onclick={() => setTab('redscript')}
            class="p-4 rounded border border-nvidia-border bg-nvidia-surface/60 hover:bg-nvidia-surface/90 hover:border-nvidia-accent/70 transition-all duration-150 flex flex-col justify-between h-28 text-left cursor-pointer group shadow-xs"
          >
            <div class="flex justify-between items-start w-full">
              <span class="text-[10px] font-mono uppercase text-nvidia-text-muted group-hover:text-nvidia-accent transition-colors">REDSCRIPT\R6</span>
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
              <p class="text-[10px] text-nvidia-text-muted font-mono mt-0.5">r6/scripts</p>
            </div>
          </button>
        </div>
      </div>

      <!-- Persistent Tab Views -->
      <div class={currentTab === 'archive' ? 'h-full' : 'hidden'}>
        <ArchiveView bind:archives {gamePath} scanReport={archiveReport} onScanRequested={refreshAll} onStateChanged={refreshCountsOnly} />
      </div>

      <div class={currentTab === 'cet' ? 'h-full' : 'hidden'}>
        <CetView bind:plugins={cetPlugins} {gamePath} onStateChanged={refreshCountsOnly} />
      </div>

      <div class={currentTab === 'red4ext' ? 'h-full' : 'hidden'}>
        <Red4extView bind:plugins={red4extPlugins} {gamePath} onStateChanged={refreshCountsOnly} />
      </div>

      <div class={currentTab === 'redscript' ? 'h-full' : 'hidden'}>
        <RedscriptView bind:packages={redscriptPackages} {gamePath} onStateChanged={refreshCountsOnly} />
      </div>
    </main>
  </div>
</div>
