import type {
  CloudSyncStatus,
  PlatformAchievement,
  PlatformFriend,
  PluginSettingField,
  PluginSettingsConfig,
  PluginType,
} from "$lib/types";

export interface TestConnectionResult {
  success: boolean;
  message: string;
}

export interface PluginArtworkResult {
  headerImage?: string | null;
  capsuleImage?: string | null;
  heroImage?: string | null;
  logoImage?: string | null;
  iconImage?: string | null;
  screenshots?: string[];
}

export interface PluginMetadataResult {
  name?: string;
  shortDescription?: string;
  detailedDescription?: string;
  releaseDate?: string;
  releaseYear?: number;
  developers?: string[];
  publishers?: string[];
  genres?: string[];
  tags?: string[];
  categories?: string[];
  rating?: number;
  userScore?: number;
  criticScore?: number;
  communityRating?: number;
  players?: string;
  ageRating?: string;
  series?: string;
  headerImage?: string;
  capsuleImage?: string;
  heroImage?: string;
  logoImage?: string;
  iconImage?: string;
  screenshots?: string[];
  movies?: string[];
  soundtracks?: string[];
  websiteUrl?: string;
  playtimeHours?: number;
  compatibilityTier?: string;
  extra?: Record<string, any>;
}

export interface PlatformLaunchSpec {
  protocolUri?: string;
  executablePath?: string;
  args?: string[];
}

export interface PlatformScanResult {
  installed: boolean;
  clientPath?: string | null;
  foundGamesCount?: number;
}

export interface LauncherDetectionResult {
  installed: boolean;
  executablePath?: string | null;
  detectedBinary?: string | null;
  method?: "path_binary" | "custom_path" | "system_store" | "cli_tool" | "standalone" | "not_installed";
  message?: string;
  downloadUrl?: string;
  alternativeMethod?: string | null;
  supportsSilentInstall?: boolean;
  wingetPackageId?: string;
}

export interface LauncherInstallOptions {
  targetDir?: string;
  silent?: boolean;
}

export interface LauncherInstallResult {
  success: boolean;
  message: string;
  installedPath?: string | null;
}

export interface UniversalPlugin {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly pluginType: PluginType;
  readonly author?: string;
  readonly description?: string;
  readonly icon?: string;
  readonly homepage?: string;
  readonly features?: string[];

  // Dynamic platform metadata for platform plugins
  readonly platformId?: string;
  readonly platformLabel?: string;
  readonly platformColor?: string;
  readonly defaultStoreUrl?: string;

  // Provider launcher capabilities & options
  readonly launcherBinaryNames?: string[];
  readonly wingetPackageId?: string;
  readonly downloadUrl?: string;
  readonly alternativeUrl?: string;
  readonly alternativeLabel?: string;
  readonly supportsSilentInstall?: boolean;
  readonly supportsCustomInstallPath?: boolean;

  getSettingsSchema?(): PluginSettingsConfig;
  testConnection?(config: Record<string, any>): Promise<TestConnectionResult>;
  queryArtwork?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginArtworkResult | null>;
  queryMetadata?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginMetadataResult | null>;

  // Launcher detection, download, and silent installation options
  detectLauncher?(config?: Record<string, any>): Promise<LauncherDetectionResult>;
  installLauncher?(options?: LauncherInstallOptions, config?: Record<string, any>): Promise<LauncherInstallResult>;
  getLauncherDownloadUrl?(): string | null;

  // Platform launcher integration hooks
  getLaunchSpec?(
    game: { appId?: number | string; appName?: string; installPath?: string; exe?: string; launchArgs?: string },
    config?: Record<string, any>
  ): PlatformLaunchSpec | null;
  getInstallUri?(game: { appId?: number | string; appName?: string }): string | null;
  getUninstallUri?(game: { appId?: number | string; appName?: string }): string | null;
  getStorePageUri?(game: { appId?: number | string; appName?: string }): string | null;

  // Social & Friends list hooks
  getFriendsListUri?(): string | null;
  getFriendsList?(config?: Record<string, any>): Promise<PlatformFriend[] | null>;

  // Achievements tracking hooks
  getAchievementsUri?(game: { appId?: number | string; appName?: string }): string | null;
  getAchievements?(game: { appId?: number | string; appName?: string }, config?: Record<string, any>): Promise<PlatformAchievement[] | null>;

  // Cloud saves & sync hooks
  getCloudSyncStatus?(game: { appId?: number | string; appName?: string }, config?: Record<string, any>): Promise<CloudSyncStatus | null>;
  syncCloudSaves?(game: { appId?: number | string; appName?: string }, config?: Record<string, any>): Promise<boolean>;

  // Game scanning & folder hooks
  scanPlatform?(roots?: string[], config?: Record<string, any>): Promise<PlatformScanResult>;
  getDefaultKnownFolders?(): string[];
}

export interface PlatformMetadata {
  id: string;
  label: string;
  icon: string;
  color: string;
  defaultUrl: string;
  pluginId: string;
}
