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

export const DARK_BASE_FOUNDATION = {
  bg: '#121517',
  surface: '#181c20',
  card: '#1e2328',
  border: '#2a323d',
  textMuted: '#94a3b8',
  textPrimary: '#ffffff',
} as const;

export const LIGHT_BASE_FOUNDATION = {
  bg: '#f1f5f9',
  surface: '#f8fafc',
  card: '#ffffff',
  border: '#cbd5e1',
  textMuted: '#64748b',
  textPrimary: '#0f172a',
} as const;

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

export function generateThemeColors(mode: 'dark' | 'light', accentHex: string): AppThemeColors {
  const normHex = accentHex.startsWith('#') ? accentHex.toUpperCase() : `#${accentHex.toUpperCase()}`;
  const base = mode === 'light' ? LIGHT_BASE_FOUNDATION : DARK_BASE_FOUNDATION;
  const hoverHex = mode === 'light' ? adjustBrightness(normHex, -22) : adjustBrightness(normHex, 20);

  return {
    accent: normHex,
    accentHover: hoverHex,
    bg: base.bg,
    surface: base.surface,
    card: base.card,
    border: base.border,
    textMuted: base.textMuted,
    textPrimary: base.textPrimary,
  };
}

export const DEFAULT_DARK_THEME: AppThemeColors = generateThemeColors('dark', ACCENT_PRESETS[0].hex);
export const DEFAULT_LIGHT_THEME: AppThemeColors = generateThemeColors('light', '#5A8F00');
export const DEFAULT_THEME = DEFAULT_DARK_THEME;

export const THEME_COLOR_META = [
  { key: 'accent', label: 'Accent', description: 'Active switches, winning badges, highlights', cssVar: '--theme-accent' },
  { key: 'accentHover', label: 'Accent Hover', description: 'Button hovers, focus rings, pulse states', cssVar: '--theme-accent-hover' },
  { key: 'bg', label: 'Background', description: 'Root canvas, main app backdrop', cssVar: '--theme-bg' },
  { key: 'surface', label: 'Surface', description: 'Sidebar, list row backgrounds, inputs', cssVar: '--theme-surface' },
  { key: 'card', label: 'Card', description: 'Mod item cards, category banners, menus', cssVar: '--theme-card' },
  { key: 'border', label: 'Border', description: 'Structural lines, separators, dividers', cssVar: '--theme-border' },
  { key: 'textPrimary', label: 'Primary Text', description: 'Headings, mod titles, primary copy', cssVar: '--theme-text-primary' },
  { key: 'textMuted', label: 'Muted Text', description: 'Asset counts, file paths, helper text', cssVar: '--theme-text-muted' },
] as const;

export const APP_FONT_PRESETS = [
  { label: 'System Sans (Default)', value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif' },
  { label: 'Segoe UI (Modern Windows)', value: '"Segoe UI", sans-serif' },
  { label: 'Bahnschrift (Industrial DIN)', value: '"Bahnschrift", sans-serif' },
  { label: 'Aptos (Modern Clean)', value: '"Aptos", sans-serif' },
  { label: 'Verdana (High Legibility)', value: 'Verdana, sans-serif' },
  { label: 'Tahoma (Compact UI)', value: 'Tahoma, sans-serif' },
  { label: 'Trebuchet MS (Humanist UI)', value: '"Trebuchet MS", sans-serif' },
  { label: 'Arial (Neutral)', value: 'Arial, sans-serif' },
] as const;

export const MOD_FONT_PRESETS = [
  { label: 'Cascadia Code (Default Windows Terminal)', value: '"Cascadia Code", "Consolas", monospace' },
  { label: 'Consolas (Crisp Monospace)', value: '"Consolas", "Courier New", monospace' },
  { label: 'Lucida Console (Terminal Classic)', value: '"Lucida Console", monospace' },
  { label: 'Courier New (Fixed Pitch)', value: '"Courier New", monospace' },
  { label: 'Bahnschrift (Technical Geometric)', value: '"Bahnschrift", sans-serif' },
  { label: 'Segoe UI Mono (Clean Monospace)', value: '"Segoe UI Mono", "Segoe UI", monospace' },
] as const;

export const FONT_PRESETS = APP_FONT_PRESETS;

export type ThemeMode = 'default' | 'custom' | 'dark' | 'light';

export const DEFAULT_THEME_SETTINGS: ThemeSettings = {
  mode: 'dark',
  darkColors: { ...DEFAULT_DARK_THEME },
  lightColors: { ...DEFAULT_LIGHT_THEME },
  fontFamilyBase: APP_FONT_PRESETS[0].value,
  fontFamilyMods: MOD_FONT_PRESETS[0].value,
  isCompact: true,
};

export const DEFAULT_APP_CONFIG: AppConfig = {
  targetGamePath: 'G:\\SteamLibrary\\steamapps\\common\\Cyberpunk 2077',
  activeTab: 'home',
  showConflictSummary: true,
  windowWidth: 1600,
  windowHeight: 1000,
  theme: { ...DEFAULT_THEME_SETTINGS },
};

const STORAGE_KEY = 'cp2077_theme_settings_v2';

export function loadThemeSettings(): ThemeSettings {
  if (typeof localStorage === 'undefined') {
    return {
      ...DEFAULT_THEME_SETTINGS,
      darkColors: { ...DEFAULT_DARK_THEME },
      lightColors: { ...DEFAULT_LIGHT_THEME },
    };
  }

  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      const legacyFont = parsed.fontFamily || APP_FONT_PRESETS[0].value;
      const loadedMode = parsed.mode === 'light' ? 'light' : 'dark';

      const darkAccent = parsed.darkColors?.accent || ACCENT_PRESETS[0].hex;
      const lightAccent = parsed.lightColors?.accent || '#5A8F00';

      return {
        mode: loadedMode,
        darkColors: generateThemeColors('dark', darkAccent),
        lightColors: generateThemeColors('light', lightAccent),
        fontFamilyBase: parsed.fontFamilyBase || legacyFont,
        fontFamilyMods: parsed.fontFamilyMods || MOD_FONT_PRESETS[0].value,
        fontFamily: parsed.fontFamilyBase || legacyFont,
        isCompact: true,
      };
    } catch {
      // fallback
    }
  }

  return {
    ...DEFAULT_THEME_SETTINGS,
    darkColors: { ...DEFAULT_DARK_THEME },
    lightColors: { ...DEFAULT_LIGHT_THEME },
  };
}

export function applyThemeSettings(settings: ThemeSettings) {
  if (typeof document === 'undefined') return;

  const activeColors = settings.mode === 'light' ? settings.lightColors : settings.darkColors;
  const root = document.documentElement;

  for (const meta of THEME_COLOR_META) {
    root.style.setProperty(meta.cssVar, activeColors[meta.key as keyof AppThemeColors]);
  }

  const baseFont = settings.fontFamilyBase || settings.fontFamily || APP_FONT_PRESETS[0].value;
  const modsFont = settings.fontFamilyMods || MOD_FONT_PRESETS[0].value;

  root.style.setProperty('--theme-font-base', baseFont);
  root.style.setProperty('--theme-font-mods', modsFont);
  root.style.colorScheme = settings.mode;

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }
}

export async function loadConfigFromDisk(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>('load_app_config');
    if (config && config.theme) {
      if (!config.theme.fontFamilyBase && (config.theme as any).fontFamily) {
        config.theme.fontFamilyBase = (config.theme as any).fontFamily;
      }
      if (!config.theme.fontFamilyMods) {
        config.theme.fontFamilyMods = MOD_FONT_PRESETS[0].value;
      }
      config.theme.fontFamily = config.theme.fontFamilyBase;

      config.theme.darkColors = generateThemeColors('dark', config.theme.darkColors?.accent || ACCENT_PRESETS[0].hex);
      config.theme.lightColors = generateThemeColors('light', config.theme.lightColors?.accent || '#5A8F00');

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

export async function applyAndPersistTheme(settings: ThemeSettings): Promise<void> {
  applyThemeSettings(settings);
  try {
    settings.fontFamily = settings.fontFamilyBase;
    let config = await invoke<AppConfig>('load_app_config').catch(() => null);
    if (!config) {
      config = { ...DEFAULT_APP_CONFIG };
    }
    config.theme = settings;
    await invoke('save_app_config', { config });
  } catch (err) {
    console.error('Failed to persist theme to disk:', err);
  }
}

export function loadSavedTheme(): { mode: any; customColors: AppThemeColors } {
  const settings = loadThemeSettings();
  return {
    mode: settings.mode === 'light' ? 'custom' : 'default',
    customColors: settings.mode === 'light' ? settings.lightColors : settings.darkColors,
  };
}

export function applyTheme(mode: any, customColors?: AppThemeColors) {
  const settings = loadThemeSettings();
  if (customColors) {
    if (mode === 'custom' || mode === 'light') {
      settings.mode = 'dark';
      settings.darkColors = { ...customColors };
    } else {
      settings.mode = 'dark';
      settings.darkColors = { ...DEFAULT_DARK_THEME };
    }
  }
  applyThemeSettings(settings);
}
