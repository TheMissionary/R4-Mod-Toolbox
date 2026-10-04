export type ActiveView = 'home' | 'archive' | 'cet' | 'red4ext' | 'redscript';

export interface XlItem {
  file_name: string;
  path: string;
  size_bytes: number;
  enabled: boolean;
  associated_archive?: string | null;
}

export interface ArchiveItem {
  name: string;
  file_name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  enabled: boolean;
  has_conflicts: boolean;
  conflicts_with: string[];
  wins: string[];
  loses: string[];
  associated_xl?: XlItem | null;
  is_delimiter?: boolean;
  category_name?: string | null;
}

export interface CategoryItem {
  id: string;
  name: string;
  enabled: boolean;
  collapsed: boolean;
  isEditing: boolean;
}

export type LoadOrderItem =
  | { type: 'category'; category: CategoryItem }
  | { type: 'archive'; archive: ArchiveItem };

export interface ArchiveScanReport {
  archives: ArchiveItem[];
  total_conflicts: number;
  affected_archives_count: number;
  unassociated_xl: XlItem[];
}

export interface ArchiveProfile {
  name: string;
  active_order: string[];
  disabled_list: string[];
}

export interface ProfilesConfig {
  preset_1?: ArchiveProfile | null;
  preset_2?: ArchiveProfile | null;
}

export interface CetPluginItem {
  name: string;
  path: string;
  has_init: boolean;
  size_bytes: number;
  enabled: boolean;
  is_dir?: boolean;
}

export interface Red4extPluginItem {
  name: string;
  path: string;
  size_bytes: number;
  enabled: boolean;
  is_dir?: boolean;
}

export interface RedScriptItem {
  name: string;
  path: string;
  reds_count: number;
  size_bytes: number;
  enabled: boolean;
  is_dir?: boolean;
}

export interface ScanResult {
  is_valid_game_path: boolean;
  game_version: string;
  archive_active: number;
  archive_total: number;
  cet_active: number;
  cet_total: number;
  red4ext_active: number;
  red4ext_total: number;
  redscript_active: number;
  redscript_total: number;
}

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

export interface ThemeSettings {
  mode: 'dark' | 'light';
  darkColors: AppThemeColors;
  lightColors: AppThemeColors;
  fontFamilyBase: string;
  fontFamilyMods: string;
  fontFamily?: string;
  isCompact?: boolean;
}

export interface AppConfig {
  targetGamePath: string;
  activeTab: string;
  showConflictSummary: boolean;
  windowWidth?: number;
  windowHeight?: number;
  windowX?: number;
  windowY?: number;
  activeProfileSlot?: 1 | 2 | null;
  theme: ThemeSettings;
}
