export interface AppThemeColors {
  accent: string;
  accentHover: string;
  bg: string;
  surface: string;
  card: string;
  border: string;
  textMuted: string;
  textPrimary: string;
}

export const DEFAULT_DARK_THEME: AppThemeColors = {
  accent: '#76b900',
  accentHover: '#88d600',
  bg: '#121517',
  surface: '#181c20',
  card: '#1e2328',
  border: '#2a323d',
  textMuted: '#94a3b8',
  textPrimary: '#ffffff',
};

export const DEFAULT_LIGHT_THEME: AppThemeColors = {
  accent: '#5a8f00',
  accentHover: '#4c7a00',
  bg: '#f1f5f9',
  surface: '#f8fafc',
  card: '#ffffff',
  border: '#cbd5e1',
  textMuted: '#64748b',
  textPrimary: '#0f172a',
};

// Backward-compatibility alias for existing components
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

export const FONT_PRESETS = [
  { label: 'System Sans (Default)', value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif' },
  { label: 'Cyberpunk Mono (Consolas)', value: '"Consolas", "Courier New", monospace' },
  { label: 'Cascadia Code', value: '"Cascadia Code", "Consolas", monospace' },
  { label: 'Segoe UI', value: '"Segoe UI", sans-serif' },
  { label: 'Bahnschrift', value: '"Bahnschrift", sans-serif' },
  { label: 'Arial', value: 'Arial, sans-serif' },
] as const;

export type ThemeMode = 'default' | 'custom' | 'dark' | 'light';

export interface ThemeSettings {
  mode: 'dark' | 'light';
  darkColors: AppThemeColors;
  lightColors: AppThemeColors;
  fontFamily: string;
  isCompact: boolean;
}

export const DEFAULT_THEME_SETTINGS: ThemeSettings = {
  mode: 'dark',
  darkColors: { ...DEFAULT_DARK_THEME },
  lightColors: { ...DEFAULT_LIGHT_THEME },
  fontFamily: FONT_PRESETS[0].value,
  isCompact: false,
};

const STORAGE_KEY = 'cp2077_theme_settings_v2';

// Modern Settings Loader
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
      return {
        mode: parsed.mode === 'light' ? 'light' : 'dark',
        darkColors: { ...DEFAULT_DARK_THEME, ...parsed.darkColors },
        lightColors: { ...DEFAULT_LIGHT_THEME, ...parsed.lightColors },
        fontFamily: parsed.fontFamily || FONT_PRESETS[0].value,
        isCompact: !!parsed.isCompact,
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

// Modern Settings Applier
export function applyThemeSettings(settings: ThemeSettings) {
  if (typeof document === 'undefined') return;

  const activeColors = settings.mode === 'light' ? settings.lightColors : settings.darkColors;
  const root = document.documentElement;

  for (const meta of THEME_COLOR_META) {
    root.style.setProperty(meta.cssVar, activeColors[meta.key as keyof AppThemeColors]);
  }

  root.style.setProperty('--theme-font-family', settings.fontFamily);
  root.style.colorScheme = settings.mode;
  root.setAttribute('data-density', settings.isCompact ? 'compact' : 'normal');

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }
}

// Backward-compatible Loader for existing components
export function loadSavedTheme(): { mode: any; customColors: AppThemeColors } {
  const settings = loadThemeSettings();
  return {
    mode: settings.mode === 'light' ? 'custom' : 'default',
    customColors: settings.mode === 'light' ? settings.lightColors : settings.darkColors,
  };
}

// Backward-compatible Applier for existing components
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
