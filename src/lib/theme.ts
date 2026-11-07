import { invoke } from '@tauri-apps/api/core';
import type { AppThemeColors, ThemeSettings, AppConfig } from '$lib/types';

export type { AppThemeColors, ThemeSettings, AppConfig };

export const ACCENT_PRESETS = [
  { label: 'NVIDIA Green', hex: '#76B900' },
  { label: 'Cyberpunk Yellow', hex: '#FCEE0A' },
  { label: 'Arasaka Crimson', hex: '#E11D48' },
  { label: 'Netrunner Cyan', hex: '#06B6D4' },
  { label: 'Tyger Violet', hex: '#A855F7' },
  { label: 'Afterlife Amber', hex: '#F59E0B' },
  { label: 'Militech Blue', hex: '#2563EB' },
  { label: 'Clean Slate', hex: '#64748B' },
] as const;

export const DARK_ATMOSPHERES: Record<string, Omit<AppThemeColors, 'accent' | 'accentHover' | 'accentContrast'>> = {
  obsidian: { bg: '#000000', surface: '#0a0a0a', card: '#141414', border: '#262626', textPrimary: '#ffffff', textMuted: '#a3a3a3' },
  slate: { bg: '#0f172a', surface: '#1e293b', card: '#334155', border: '#475569', textPrimary: '#f8fafc', textMuted: '#94a3b8' },
  graphite: { bg: '#121517', surface: '#181c20', card: '#1e2328', border: '#2a323d', textPrimary: '#ffffff', textMuted: '#94a3b8' },
  mocha: { bg: '#1c1917', surface: '#292524', card: '#44403c', border: '#57534e', textPrimary: '#fafaf9', textMuted: '#a8a29e' }
};

export const LIGHT_ATMOSPHERES: Record<string, Omit<AppThemeColors, 'accent' | 'accentHover' | 'accentContrast'>> = {
  pearl: { bg: '#ffffff', surface: '#f8fafc', card: '#f1f5f9', border: '#e2e8f0', textPrimary: '#0f172a', textMuted: '#64748b' },
  cloud: { bg: '#f3f4f6', surface: '#e5e7eb', card: '#d1d5db', border: '#9ca3af', textPrimary: '#111827', textMuted: '#6b7280' },
  frost: { bg: '#f1f5f9', surface: '#e2e8f0', card: '#cbd5e1', border: '#94a3b8', textPrimary: '#0f172a', textMuted: '#475569' },
  parchment: { bg: '#fdfbf7', surface: '#f4f1ea', card: '#e8e3d9', border: '#d6cebf', textPrimary: '#292524', textMuted: '#78716c' }
};

export function adjustBrightness(hex: string, percent: number): string {
  let cleanHex = hex.replace('#', '').trim();
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split('').map(c => c + c).join('');
  }
  const num = parseInt(cleanHex, 16);
  if (isNaN(num)) return hex;

  let r = (num >> 16) + percent;
  let g = ((num >> 8) & 0x00ff) + percent;
  let b = (num & 0x0000ff) + percent;

  r = Math.min(255, Math.max(0, r));
  g = Math.min(255, Math.max(0, g));
  b = Math.min(255, Math.max(0, b));

  return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1).toUpperCase()}`;
}

export function getContrastColor(hex: string): string {
  let cleanHex = hex.replace('#', '').trim();
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split('').map(c => c + c).join('');
  }
  const num = parseInt(cleanHex, 16);
  if (isNaN(num)) return '#000000';
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  const brightness = (r * 299 + g * 587 + b * 114) / 1000;
  return brightness > 135 ? '#000000' : '#ffffff';
}

export function isModNew(createdAtMs?: number | null, hours: number = 6): boolean {
  if (!createdAtMs) return false;
  const cutoff = Date.now() - (hours * 60 * 60 * 1000);
  return createdAtMs >= cutoff;
}

export function generateThemeColors(mode: 'dark' | 'light', atmosphere: string, accentHex: string): AppThemeColors {
  const normHex = accentHex.startsWith('#') ? accentHex.toUpperCase() : `#${accentHex.toUpperCase()}`;
  const hoverHex = mode === 'light' ? adjustBrightness(normHex, -22) : adjustBrightness(normHex, 20);
  const contrast = getContrastColor(normHex);
  
  const base = mode === 'light' 
    ? (LIGHT_ATMOSPHERES[atmosphere] || LIGHT_ATMOSPHERES.cloud)
    : (DARK_ATMOSPHERES[atmosphere] || DARK_ATMOSPHERES.graphite);

  return {
    accent: normHex,
    accentHover: hoverHex,
    accentContrast: contrast,
    ...base
  };
}

export const MOD_FONT_PRESETS = [
  { label: 'Geist Mono (Default)', value: "'GeistMono', \"Cascadia Code\", \"Consolas\", monospace" },
  { label: 'Cascadia Code (Windows Terminal)', value: '"Cascadia Code", "Consolas", monospace' },
  { label: 'Consolas (Crisp Monospace)', value: '"Consolas", "Courier New", monospace' },
  { label: 'Lucida Console (Terminal Classic)', value: '"Lucida Console", monospace' },
  { label: 'Courier New (Fixed Pitch)', value: '"Courier New", monospace' },
  { label: 'Bahnschrift (Technical Geometric)', value: '"Bahnschrift", sans-serif' },
  { label: 'Segoe UI Mono (Clean Monospace)', value: '"Segoe UI Mono", "Segoe UI", monospace' },
] as const;

export const DEFAULT_THEME_SETTINGS: ThemeSettings = {
  mode: 'dark',
  atmosphere: 'graphite',
  accentHex: '#76B900',
  fontFamilyMods: MOD_FONT_PRESETS[0].value,
  isCompact: true,
};

export const DEFAULT_APP_CONFIG: AppConfig = {
  targetGamePath: '',
  activeTab: 'archive',
  showConflictSummary: true,
  windowWidth: 1600,
  windowHeight: 1000,
  recentDaysThreshold: 30,
  newModsHours: 6,
  showNewMods: false,
  customTextEditorPath: '',
  customPhotoPath: '',
  enableOrganizerView: false,
  showCategoryLabelsInLoadOrder: true,
  organizerColumns: 'auto',
  theme: { ...DEFAULT_THEME_SETTINGS },
};

const STORAGE_KEY = 'cp2077_theme_settings_v3';

export function loadThemeSettings(): ThemeSettings {
  if (typeof localStorage === 'undefined') return { ...DEFAULT_THEME_SETTINGS };
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw) {
    try {
      return { ...DEFAULT_THEME_SETTINGS, ...JSON.parse(raw) };
    } catch {}
  }
  return { ...DEFAULT_THEME_SETTINGS };
}

export function applyThemeSettings(settings: ThemeSettings) {
  if (typeof document === 'undefined') return;

  const activeColors = generateThemeColors(settings.mode, settings.atmosphere, settings.accentHex);
  const root = document.documentElement;

  root.style.setProperty('--theme-accent', activeColors.accent);
  root.style.setProperty('--theme-accent-hover', activeColors.accentHover);
  root.style.setProperty('--theme-accent-contrast', activeColors.accentContrast);
  root.style.setProperty('--theme-bg', activeColors.bg);
  root.style.setProperty('--theme-surface', activeColors.surface);
  root.style.setProperty('--theme-card', activeColors.card);
  root.style.setProperty('--theme-border', activeColors.border);
  root.style.setProperty('--theme-text-primary', activeColors.textPrimary);
  root.style.setProperty('--theme-text-muted', activeColors.textMuted);

  root.style.setProperty('--theme-font-base', "'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif");
  root.style.setProperty('--theme-font-mods', settings.fontFamilyMods || MOD_FONT_PRESETS[0].value);
  root.style.colorScheme = settings.mode;

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }
}

export async function loadConfigFromDisk(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>('load_app_config');
    if (config && config.theme) {
      config.theme.atmosphere = config.theme.atmosphere || 'graphite';
      config.theme.accentHex = config.theme.accentHex || '#76B900';
      config.theme.fontFamilyMods = config.theme.fontFamilyMods || MOD_FONT_PRESETS[0].value;
      
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(config.theme));
      }
      return config;
    }
  } catch (err) {
    console.error('Failed to load config from disk, falling back to local defaults:', err);
  }
  return {
    ...DEFAULT_APP_CONFIG,
    theme: loadThemeSettings(),
  };
}

export async function saveConfigToDisk(config: AppConfig): Promise<void> {
  try {
    await invoke('save_app_config', { config });
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config.theme));
    }
  } catch (err) {
    console.error('Failed to save config to disk:', err);
  }
}