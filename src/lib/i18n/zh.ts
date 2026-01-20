export default {
  // Navigation
  navigation: {
    title: '导航',
    library: '文献库',
    categories: '分类',
    tags: '标签',
    favorites: '收藏',
    trash: '回收站'
  },

  // Main content
  main: {
    title: '文献库',
    importDocuments: '导入文档',
    search: '搜索',
    noDocuments: '暂无文档'
  },

  // Details panel
  details: {
    title: '详情',
    noSelection: '选择一个文档以查看详情'
  },

  // Theme
  theme: {
    darkMode: '深色模式',
    lightMode: '浅色模式',
    themeSelector: '主题',
    colorTheme: '颜色主题',
    selectTheme: '选择主题'
  },

  // Status bar
  status: {
    synced: '已同步',
    syncing: '同步中...',
    unsynced: '未同步',
    documents: '文档',
    searchStatus: '就绪',
    memoryUsage: '内存',
    version: '版本'
  },

  // Language
  language: {
    title: '语言',
    selectLanguage: '选择语言',
    english: 'English',
    chinese: '中文',
    japanese: '日本語',
    korean: '한국어'
  }
} as const;
