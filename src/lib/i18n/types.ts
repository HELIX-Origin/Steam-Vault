// Hand-rolled i18n type definitions for Steam-Vault.
// Supports all 30 official Steam platform languages.

export interface SteamLanguage {
  /** Standard web/ISO language code (e.g. "en", "de", "zh-CN", "ja"). */
  id: string;
  /** Steamworks API language code (e.g. "english", "german", "schinese", "japanese"). */
  steamCode: string;
  /** Steam Web API language code. */
  webCode: string;
  /** English display name of the language (e.g. "German", "Japanese"). */
  name: string;
  /** Native endonym in its own script (e.g. "Deutsch", "日本語", "العربية"). */
  nativeName: string;
  /** Right-to-left layout flag (true for Arabic). */
  rtl?: boolean;
}

export interface TranslationDictionary {
  nav: {
    brandTitle: string;
    brandSubtitle: string;
    dashboard: string;
    library: string;
    storage: string;
    artwork: string;
    collections: string;
    platforms: string;
    organize: string;
    settings: string;
  };
  status: {
    steamRunning: string;
    steamClosed: string;
    dismiss: string;
  };
  common: {
    save: string;
    saved: string;
    saving: string;
    cancel: string;
    delete: string;
    edit: string;
    remove: string;
    add: string;
    search: string;
    filter: string;
    scan: string;
    scanning: string;
    import: string;
    importing: string;
    refresh: string;
    refreshing: string;
    apply: string;
    applying: string;
    back: string;
    done: string;
    clear: string;
    close: string;
    loading: string;
    none: string;
    all: string;
    details: string;
    error: string;
    warning: string;
    success: string;
    browse: string;
    enabled: string;
    disabled: string;
  };
  dashboard: {
    title: string;
    subtitle: string;
    totalGames: string;
    installedGames: string;
    spaceUsed: string;
    collectionsCount: string;
    shortcutsCount: string;
    quickActions: string;
    openLibrary: string;
    scanPlatforms: string;
    manageArtwork: string;
    autoCategorize: string;
    recentGames: string;
  };
  library: {
    searchPlaceholder: string;
    filterPlatform: string;
    filterCategory: string;
    allPlatforms: string;
    allCategories: string;
    installedOnly: string;
    favoritesOnly: string;
    hiddenOnly: string;
    sortBy: string;
    sortName: string;
    sortSize: string;
    sortPlaytime: string;
    sortLastPlayed: string;
    noGamesFound: string;
    gamesCount: string;
    gameDetails: string;
    play: string;
    launch: string;
    exePath: string;
    installFolder: string;
    collections: string;
    favorite: string;
    unfavorite: string;
    sourcePlatform: string;
  };
  organize: {
    title: string;
    subtitle: string;
    mode: string;
    byGenre: string;
    byTags: string;
    byDeveloper: string;
    byPublisher: string;
    byYear: string;
    prefix: string;
    maxCategories: string;
    removeExisting: string;
    onlyInstalled: string;
    fetchMetadata: string;
    previewPlan: string;
    applyPlan: string;
    categorizedSummary: string;
    noPlan: string;
  };
  collections: {
    title: string;
    subtitle: string;
    newCollection: string;
    collectionName: string;
    create: string;
    dynamic: string;
    static: string;
    gamesCount: string;
    deleteCollection: string;
    confirmDelete: string;
  };
  platforms: {
    title: string;
    subtitle: string;
    scanAll: string;
    importAll: string;
    importSelected: string;
    scanning: string;
    detectedGames: string;
    launchUri: string;
    extraFolders: string;
    addFolder: string;
    findFolders: string;
    emulatorsTitle: string;
    selectEmulator: string;
    romDirectory: string;
  };
  storage: {
    title: string;
    subtitle: string;
    steamLibraries: string;
    drives: string;
    freeSpace: string;
    totalSpace: string;
    moveGames: string;
    targetLibrary: string;
    moveSelected: string;
    calculating: string;
  };
  artwork: {
    title: string;
    subtitle: string;
    autoFetchAll: string;
    autoFetching: string;
    gridPortrait: string;
    gridLandscape: string;
    heroBanner: string;
    logo: string;
    icon: string;
    applied: string;
    missing: string;
    uploadCustom: string;
    removeArtwork: string;
    apiKeyNotice: string;
  };
  settings: {
    title: string;
    subtitle: string;
    steamFolder: string;
    autoDetect: string;
    librariesCount: string;
    accountsCount: string;
    activeAccount: string;
    storeCountry: string;
    language: string;
    languageHelp: string;
    apiKeys: string;
    steamApiKey: string;
    steamgriddbApiKey: string;
    igdbClientId: string;
    igdbClientSecret: string;
    nonSteamSources: string;
    sourcesHelp: string;
    saveSettings: string;
    savedSuccess: string;
  };
}
