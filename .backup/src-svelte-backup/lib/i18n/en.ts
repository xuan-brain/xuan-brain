export default {
  // Navigation
  navigation: {
    title: 'Navigation',
    library: 'Library',
    categories: 'Categories',
    tags: 'Tags',
    favorites: 'Favorites',
    trash: 'Trash'
  },

  // Main content
  main: {
    title: 'Library',
    importDocuments: 'Import Documents',
    search: 'Search',
    noDocuments: 'No documents yet'
  },

  // Details panel
  details: {
    title: 'Details',
    noSelection: 'Select a document to view details'
  },

  // Theme
  theme: {
    darkMode: 'Dark Mode',
    lightMode: 'Light Mode',
    themeSelector: 'Theme',
    colorTheme: 'Color Theme',
    selectTheme: 'Select Theme'
  },

  // Status bar
  status: {
    synced: 'Synced',
    syncing: 'Syncing...',
    unsynced: 'Unsynced',
    documents: 'Documents',
    searchStatus: 'Ready',
    memoryUsage: 'Memory',
    version: 'Version'
  },

  // Language
  language: {
    title: 'Language',
    selectLanguage: 'Select Language',
    english: 'English',
    chinese: '中文',
    japanese: '日本語',
    korean: '한국어'
  }
} as const;
