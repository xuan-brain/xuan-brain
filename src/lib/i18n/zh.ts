export default {
  // Navigation
  navigation: {
    title: "导航",
    library: "文献库",
    categories: "分类",
    tags: "标签",
    favorites: "收藏",
    trash: "回收站",
  },

  // Main content
  main: {
    title: "文献库",
    importDocuments: "导入文档",
    search: "搜索",
    noDocuments: "暂无文档",
    startUsing: "点击「导入文档」按钮开始使用",
    importFirst: "导入第一篇文献",
  },

  // Statistics
  statistics: {
    totalDocuments: "总文献数",
    totalCategories: "分类数量",
    totalTags: "标签数量",
  },

  // Details panel
  details: {
    title: "详情",
    noSelection: "选择一个文档以查看详情",
  },

  // Theme
  theme: {
    darkMode: "深色模式",
    lightMode: "浅色模式",
    themeSelector: "主题",
    colorTheme: "颜色主题",
    selectTheme: "选择主题",
    accentColor: "主色调",
  },

  // Status bar
  status: {
    synced: "已同步",
    syncing: "同步中...",
    unsynced: "未同步",
    documents: "文档",
    searchStatus: "就绪",
    memoryUsage: "内存",
    version: "版本",
  },

  // Language
  language: {
    title: "语言",
    selectLanguage: "选择语言",
    english: "English",
    chinese: "中文",
  },

  // Color themes
  colorThemes: {
    blue: "蓝色",
    purple: "紫色",
    pink: "粉色",
    red: "红色",
    orange: "橙色",
    green: "绿色",
    teal: "青色",
    cyan: "青蓝色",
  },

  // Dialogs
  dialog: {
    add: "添加",
    edit: "编辑",
    delete: "删除",
    cancel: "取消",
    save: "保存",
    close: "关闭",
    confirm: "确认",
    addCategory: "添加分类",
    addSubcategory: "添加子分类",
    editCategory: "编辑分类",
    deleteCategory: "删除分类",
    addTag: "添加标签",
    editTag: "编辑标签",
    deleteTag: "删除标签",
    categoryName: "分类名称",
    tagName: "标签名称",
    parentCategory: "父分类",
    selectColor: "选择颜色",
    enterCategoryName: "请输入分类名称",
    enterTagName: "请输入标签名称",
    categoryNameRequired: "名称不能为空",
    categoryNameMaxLength: "名称最多50个字符",
    categoryNameRules: "分类名称不能为空，最多50个字符",
    tagNameRequired: "标签名称不能为空",
    tagNameMaxLength: "标签名称最多30个字符",
    tagNameRules: "标签名称不能为空，最多30个字符",
    adding: "添加中...",
    saving: "保存中...",
    deleting: "删除中...",
    rename: "重命名",
  },
} as const;
