export type ActiveView = 'home' | 'archive' | 'cet' | 'red4ext' | 'redscript' | 'r6tweaks';

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
  associated_xls?: XlItem[];
  is_delimiter?: boolean;
  category_name?: string | null;
  created_at_ms?: number;
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

export interface CetPluginItem {
  name: string;
  path: string;
  has_init: boolean;
  size_bytes: number;
  file_count: number;
  enabled: boolean;
  is_dir?: boolean;
  created_at_ms?: number;
}

export interface Red4extPluginItem {
  name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  enabled: boolean;
  is_dir?: boolean;
  created_at_ms?: number;
}

export interface RedScriptItem {
  name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  enabled: boolean;
  is_dir?: boolean;
  created_at_ms?: number;
}

export interface R6TweaksItem {
  name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  enabled: boolean;
  is_dir?: boolean;
  created_at_ms?: number;
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
  r6tweaks_active: number;
  r6tweaks_total: number;
}

export interface LedgerEntry {
  name: string;
  mod_type: string;
  path: string;
  first_seen: number;
  os_created?: number;
  os_modified?: number;
  enabled?: boolean;
}

export interface FileNode {
  name: string;
  path: string;
  size_bytes: number;
  is_dir: boolean;
  extension: string;
}

export interface AppThemeColors {
  accent: string;
  accentHover: string;
  accentContrast: string;
  bg: string;
  surface: string;
  card: string;
  border: string;
  textMuted: string;
  textPrimary: string;
}

export interface ThemeSegment {
  background?: string;
  opacity?: number;
  blur?: string;
  border?: string;
  boxShadow?: string;
}

export interface ThemeSettings {
  mode: 'dark' | 'light';
  atmosphere: string;
  accentHex: string;
  fontFamilyMods: string;
  isCompact?: boolean;
  heroBannerPath?: string;
  sidebarImage?: string;
  surfaceOpacity?: number;
  backdropBlur?: 'none' | 'sm' | 'md' | 'lg';
  uiRounding?: 'sharp' | 'standard' | 'soft';
  borderThickness?: 'thin' | 'standard' | 'thick';
  neonIntensity?: 'none' | 'low' | 'high';
  groupBackgroundStyle?: 'default' | 'accent-wash' | 'gradient-flare';
  
  // Modular Theme Segments (For imported .r4theme packs)
  sidebar?: ThemeSegment;
  header?: ThemeSegment;
  footer?: ThemeSegment;
  canvas?: ThemeSegment;
  drawer?: ThemeSegment;
}

export interface CategoryLabel {
  id: string;
  name: string;
  color: string;
  stashed?: boolean;
}

export interface LabelsConfig {
  categories: CategoryLabel[];
  assignments: Record<string, string>;
  orders?: Record<string, string[]>;
}

export interface AppConfig {
  targetGamePath: string;
  activeTab: string;
  showConflictSummary: boolean;
  windowWidth?: number;
  windowHeight?: number;
  windowX?: number;
  windowY?: number;
  recentDaysThreshold?: number;
  newModsHours?: number;
  showNewMods?: boolean;
  customTextEditorPath?: string;
  customPhotoPath?: string;
  customLaunchPath?: string;
  enableOrganizerView?: boolean;
  showCategoryLabelsInLoadOrder?: boolean;
  organizerColumns?: 'auto' | '1' | '2' | '3';
  theme: ThemeSettings;
}

export interface MultiLinkMember {
  mod_type: string;
  name: string;
}

export interface MultiLinkCluster {
  id: string;
  name: string;
  members: MultiLinkMember[];
}