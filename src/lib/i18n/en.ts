export default {
  // Navigation
  navigation: {
    title: "Navigation",
    library: "Library",
    categories: "Categories",
    tags: "Tags",
    favorites: "Favorites",
    trash: "Trash",
    settings: "Settings",
    user: "User",
    system: "System",
    ai: "AI",
    papers: "Papers",
    clips: "Clips",
    writing: "Writing",
    subscriptions: "Subscriptions",
    about: "About",
  },

  // Main content
  main: {
    title: "Library",
    importDocuments: "Import Documents",
    search: "Search",
    noDocuments: "No documents yet",
    startUsing: "Click 'Import Documents' to get started",
    importFirst: "Import your first document",
  },

  // Statistics
  statistics: {
    totalDocuments: "Total Documents",
    totalCategories: "Categories",
    totalTags: "Tags",
  },

  // Details panel
  details: {
    title: "Details",
    noSelection: "Select a document to view details",
  },

  // Document list
  document: {
    title: "Title",
    authors: "Authors",
    source: "Journal/Conference",
    year: "Year",
    labels: "Labels",
  },

  // Theme
  theme: {
    darkMode: "Dark Mode",
    lightMode: "Light Mode",
    themeSelector: "Theme",
    colorTheme: "Color Theme",
    selectTheme: "Select Theme",
    accentColor: "Accent Color",
  },

  // Status bar
  status: {
    synced: "Synced",
    syncing: "Syncing...",
    unsynced: "Unsynced",
    documents: "Documents",
    searchStatus: "Ready",
    memoryUsage: "Memory",
    version: "Version",
  },

  // Language
  language: {
    title: "Language",
    selectLanguage: "Select Language",
    english: "English",
    chinese: "中文",
  },

  // Color themes
  colorThemes: {
    blue: "Blue",
    purple: "Purple",
    pink: "Pink",
    red: "Red",
    orange: "Orange",
    green: "Green",
    teal: "Teal",
    cyan: "Cyan",
  },

  // Dialogs
  dialog: {
    add: "Add",
    edit: "Edit",
    delete: "Delete",
    cancel: "Cancel",
    save: "Save",
    close: "Close",
    confirm: "Confirm",
    addCategory: "Add Category",
    addSubcategory: "Add Subcategory",
    editCategory: "Edit Category",
    deleteCategory: "Delete Category",
    addTag: "Add Tag",
    editTag: "Edit Tag",
    deleteTag: "Delete Tag",
    categoryName: "Category Name",
    tagName: "Tag Name",
    parentCategory: "Parent Category",
    selectColor: "Select Color",
    enterCategoryName: "Enter category name",
    enterTagName: "Enter tag name",
    categoryNameRequired: "Category name is required",
    categoryNameMaxLength: "Category name cannot exceed 50 characters",
    categoryNameRules: "Category name is required, maximum 50 characters",
    selectCategory: "Select Category",
    tagNameRequired: "Tag name is required",
    tagNameMaxLength: "Tag name cannot exceed 30 characters",
    tagNameRules: "Tag name is required, maximum 30 characters",
    adding: "Adding...",
    saving: "Saving...",
    deleting: "Deleting...",
    rename: "Rename",
    restore: "Restore",
    permanentlyDelete: "Permanently Delete",
    confirmPermanentlyDelete:
      "Are you sure you want to permanently delete this document? This action cannot be undone.",
    restoreFailed: "Restore Failed",
    deleteFailed: "Delete Failed",
  },

  // Toolbar
  toolbar: {
    doi: "DOI",
    arxiv: "arXiv",
    importPdf: "Import PDF",
    importByDoi: "Import Paper by DOI",
    doiDescription:
      "Enter the DOI (Digital Object Identifier) to import the paper.",
    doiPlaceholder: "e.g., 10.1038/nature12373",
    importByArxiv: "Import Paper from arXiv",
    arxivDescription:
      "Enter the arXiv ID (e.g., 2301.12345) or URL to import the paper.",
    arxivPlaceholder: "e.g., 2301.12345 or https://arxiv.org/abs/2301.12345",
    import: "Import",
    doiRequired: "DOI is required",
    arxivRequired: "arXiv ID is required",
  },

  // Settings
  settings: {
    appearance: "Appearance",
    llmProviders: "LLM Providers",
    grobidServers: "GROBID Servers",
    addProvider: "Add Provider",
    addServer: "Add Server",
    noProviders: "No LLM providers configured",
    noServers: "No GROBID servers configured",
    default: "Default",
    active: "Active",
    model: "Model",
    version: "Version",
    license: "License",
    database: "Database",
    orm: "ORM",
    appVersion: "App Version",
    tauriVersion: "Tauri Version",
    vueVersion: "Vue Version",
    vuetifyVersion: "Vuetify Version",
    appDescription:
      "AI-driven research literature management desktop application",
    github: "GitHub",
    reportIssue: "Report Issue",
  },
} as const;
