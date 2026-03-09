# 前端迁移计划：React → Vue 3 + Vuetify 3

> 迁移状态：已完成（2026-02-11）。本文件保留作为历史记录与参考，实际项目已采用 Vue 3 + Vuetify 3 + Pinia + Vue Router 4 技术栈。

## Context

当前 xuan-brain 项目使用 React 19 + Ant Design + Mantine 作为前端技术栈。由于用户只熟悉 Vue 框架，需要将整个前端迁移到 Vue 3 + Vuetify 3 技术栈，并采用 Vuetify 的 Discord 风格预置布局。

### 当前技术栈

- **框架**: React 19 with TypeScript
- **路由**: React Router v7
- **UI组件库**: Ant Design + Mantine (混合使用)
- **状态管理**: Zustand + persist 中间件
- **构建工具**: Vite 6
- **国际化**: 自定义 i18n 实现
- **主题**: CSS 变量 + Ant Design 主题系统

### 目标技术栈

- **框架**: Vue 3 with Composition API + TypeScript
- **路由**: Vue Router 4
- **UI组件库**: Vuetify 3 (Material Design 3)
- **状态管理**: Pinia + pinia-plugin-persistedstate
- **构建工具**: Vite 6 (继续使用)
- **国际化**: Vue I18n 10
- **主题**: Vuetify 3 内置主题系统
- **PDF 查看器**: vue-pdf-embed
- **分类树**: Vuetify v-treeview
- **数据表格**: Vuetify v-data-table (支持虚拟滚动)

### 用户确认的技术选择

- **迁移策略**: A (完全替换) - 一次性将所有 React 代码替换为 Vue
- **分类树组件**: A (Vuetify v-treeview) - 原生 Vuetify 集成
- **PDF 查看器**: A (vue-pdf-embed) - Vue 原生 PDF 查看器
- **数据表格优先级**: 虚拟滚动(1) > 排序(2) > 行选择(3) > 分页(4) > 列宽(5) > 过滤(6)

---

## 阶段 1：基础设施迁移

### 1.1 依赖更新

**安装依赖**:

```bash
# Vue 核心
yarn add vue@^3.4.0 vue-router@^4.3.0 pinia@^2.2.0

# Vuetify 3
yarn add vuetify@^3.7.0
yarn add -D sass vite-plugin-vuetify

# Pinia 持久化
yarn add pinia-plugin-persistedstate

# Vue I18n
yarn add vue-i18n@^10.0.0

# PDF 查看器 (Vue 版本)
yarn add vue-pdf-embed@^2.0.0

# Material Design Icons
yarn add @mdi/font

# Tauri (保持不变)
# @tauri-apps/api 及相关插件保持不变

# 开发依赖
yarn add -D @vitejs/plugin-vue typescript vue-tsc @vue/tsconfig
```

**删除 React 依赖**:

```bash
yarn remove react react-dom react-router-dom antd @ant-design/icons \
  @mantine/core @mantine/hooks @tabler/icons-react \
  @headless-tree/core @headless-tree/react \
  zustand @tanstack/react-query react-hook-form \
  @vitejs/plugin-react @embedpdf/react-pdf-viewer
```

### 1.2 Vite 配置更新

**文件**: `vite.config.ts`

```typescript
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import vuetify from 'vite-plugin-vuetify';
import type { UserConfig, ConfigEnv } from 'vite';
import path from 'node:path';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(
  async (_: ConfigEnv): Promise<UserConfig> => ({
    plugins: [
      vue(),
      vuetify({ autoImport: true }), // Vuetify 自动导入
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host || '127.0.0.1',
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**'],
      },
    },
    build: {
      outDir: 'dist',
      emptyOutDir: true,
      rollupOptions: {
        input: {
          main: './index.html',
          'pdf-viewer': './src/pdf-viewer.html',
        },
      },
    },
  })
);
```

### 1.3 TypeScript 配置更新

**文件**: `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

---

## 阶段 2：核心架构迁移

### 2.1 应用入口重构

**文件**: `src/main.ts` (新建，替代 `src/main.tsx`)

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import 'vuetify/styles';
import '@mdi/font/css/materialdesignicons.css';

import App from './App.vue';
import { i18n } from './lib/i18n';
import { vuetifyKey } from './lib/vuetify';

const app = createApp(App);

// Pinia
const pinia = createPinia();
app.use(pinia);

// Vuetify
const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        colors: {
          primary: '#90caf9',
          surface: '#1f1f1f',
          background: '#141414',
        },
      },
      light: {
        colors: {
          primary: '#1976d2',
          surface: '#ffffff',
          background: '#f5f5f5',
        },
      },
    },
  },
});
app.use(vuetify);

// Vue Router
import router from './router';
app.use(router);

// i18n
app.use(i18n);

app.mount('#app');
```

### 2.2 主应用组件

**文件**: `src/App.vue` (替代 `src/App.tsx`)

```vue
<script setup lang="ts">
  import { useTheme } from 'vuetify';
  import { useAppStore } from '@/stores/useAppStore';
  import { onMounted, watch } from 'vue';

  const appStore = useAppStore();
  const theme = useTheme();

  // 初始化主题
  onMounted(() => {
    theme.global.name.value = appStore.isDark ? 'dark' : 'light';
  });

  // 监听主题变化
  watch(
    () => appStore.isDark,
    (isDark) => {
      theme.global.name.value = isDark ? 'dark' : 'light';
    }
  );
</script>

<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<style>
  /* 全局样式 */
</style>
```

---

## 阶段 3：布局系统迁移 (Discord 风格)

### 3.1 Discord 风格布局结构

Vuetify 的 Discord 风格布局使用以下组件：

```
<v-app>
  <!-- 左侧全局导航栏 (Rail 模式) -->
  <v-navigation-drawer location="left" width="80" rail>
    <!-- 全局导航菜单 -->
  </v-navigation-drawer>

  <!-- 左侧抽屉 (分类树) -->
  <v-navigation-drawer location="left" width="280">
    <!-- 分类树、标签 -->
  </v-navigation-drawer>

  <!-- 主内容区域 -->
  <v-main>
    <!-- 文档列表 -->
    <!-- 或文档详情页 -->
  </v-main>

  <!-- 右侧抽屉 (文档详情) -->
  <v-navigation-drawer location="right" width="400">
    <!-- 文档详情 -->
  </v-navigation-drawer>

  <!-- 底部状态栏 -->
  <v-footer height="36">
    <!-- 状态信息 -->
  </v-footer>
</v-app>
```

### 3.2 主布局组件

**文件**: `src/layouts/MainLayout.vue` (替代 `src/components/layout/Layout.tsx`)

```vue
<script setup lang="ts">
  import { computed } from 'vue';
  import { useRoute } from 'vue-router';
  import { useDisplay } from 'vuetify';
  import GlobalSidebar from '@/components/layout/GlobalSidebar.vue';
  import Navigation from '@/components/navigation/Navigation.vue';
  import StatusBar from '@/components/layout/StatusBar.vue';

  const route = useRoute();
  const { mdAndDown } = useDisplay();

  // 根据路由判断是否显示三栏布局
  const isPapersPage = computed(() => route.path.startsWith('/papers'));
</script>

<template>
  <v-layout class="main-layout">
    <!-- 全局侧边栏 (Rail 模式) -->
    <GlobalSidebar />

    <!-- 左侧导航抽屉 (仅文献管理页面) -->
    <v-navigation-drawer
      v-if="isPapersPage"
      location="left"
      permanent
      width="280"
      class="category-drawer"
    >
      <Navigation />
    </v-navigation-drawer>

    <!-- 主内容区域 -->
    <v-main>
      <router-view />
    </v-main>

    <!-- 底部状态栏 -->
    <v-footer height="36" class="status-bar">
      <StatusBar />
    </v-footer>
  </v-layout>
</template>

<style scoped>
  .main-layout {
    height: 100vh;
  }

  .status-bar {
    padding: 0 8px;
    display: flex;
    align-items: center;
    font-size: 12px;
  }
</style>
```

### 3.3 全局侧边栏 (Rail 模式)

**文件**: `src/components/layout/GlobalSidebar.vue` (替代 `GlobalSidebar.tsx`)

```vue
<script setup lang="ts">
  import { ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { useI18n } from '@/lib/i18n';

  const router = useRouter();
  const { t } = useI18n();

  const menuItems = [
    { icon: 'mdi-file-document', value: 'papers', title: 'navigation.papers' },
    { icon: 'mdi-content-cut', value: 'clips', title: 'navigation.clips' },
    { icon: 'mdi-pencil', value: 'writing', title: 'navigation.writing' },
    { icon: 'mdi-rss', value: 'subscriptions', title: 'navigation.subscriptions' },
  ];
</script>

<template>
  <v-navigation-drawer permanent rail width="72" class="global-sidebar">
    <v-list density="compact">
      <!-- 用户头像 -->
      <v-list-item class="user-avatar" rounded="lg">
        <template #prepend>
          <v-avatar color="primary">
            <span class="text-h6">U</span>
          </v-avatar>
        </template>
      </v-list-item>

      <v-divider class="my-2" />

      <!-- 导航菜单 -->
      <v-list-item
        v-for="item in menuItems"
        :key="item.value"
        :prepend-icon="item.icon"
        :value="item.value"
        :title="t(item.title)"
        rounded="lg"
        @click="router.push(`/${item.value}`)"
      />
    </v-list>

    <template #append>
      <v-list density="compact">
        <v-list-item
          prepend-icon="mdi-cog"
          value="settings"
          :title="t('navigation.settings')"
          rounded="lg"
          @click="router.push('/settings')"
        />
      </v-list>
    </template>
  </v-navigation-drawer>
</template>

<style scoped>
  .global-sidebar {
    border-right: 1px solid rgba(255, 255, 255, 0.12);
  }

  .user-avatar {
    margin: 8px 0;
  }
</style>
```

---

## 阶段 4：状态管理迁移 (Zustand → Pinia)

### 4.1 应用状态 Store

**文件**: `src/stores/useAppStore.ts`

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useTheme } from 'vuetify';

export interface Document {
  id: number;
  title: string;
  authors: string[];
  year: number;
  abstract?: string;
  keywords?: string[];
  fileType?: string;
  fileSize?: string;
  addedDate?: string;
  tags?: Tag[];
}

export interface Tag {
  id: number;
  name: string;
  color: string;
}

export const useAppStore = defineStore(
  'app',
  () => {
    // State
    const isDark = ref(true);
    const accentColor = ref('#3b82f6');
    const selectedDocument = ref<Document | null>(null);

    // Getters
    const currentTheme = computed(() => (isDark.value ? 'dark' : 'light'));

    // Actions
    function toggleTheme() {
      isDark.value = !isDark.value;
      const theme = useTheme();
      theme.global.name.value = currentTheme.value;
    }

    function setTheme(value: boolean) {
      isDark.value = value;
      const theme = useTheme();
      theme.global.name.value = currentTheme.value;
    }

    function setAccentColor(color: string) {
      accentColor.value = color;
    }

    function setSelectedDocument(doc: Document | null) {
      selectedDocument.value = doc;
    }

    return {
      isDark,
      accentColor,
      selectedDocument,
      currentTheme,
      toggleTheme,
      setTheme,
      setAccentColor,
      setSelectedDocument,
    };
  },
  {
    persist: {
      key: 'xuan-brain-app-storage',
      storage: localStorage,
      pick: ['isDark', 'accentColor'],
    },
  }
);
```

### 4.2 标签页 Store

**文件**: `src/stores/useTabsStore.ts`

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface Tab {
  id: string;
  paperId: number;
  title: string;
  path: string;
  isActive: boolean;
}

export const useTabsStore = defineStore(
  'tabs',
  () => {
    const tabs = ref<Tab[]>([]);
    const activeTabId = ref<string | null>(null);

    function addTab(paperId: number, title: string, path: string) {
      const existingTab = tabs.value.find((t) => t.paperId === paperId);
      if (existingTab) {
        setActiveTab(existingTab.id);
        return;
      }

      const newTab: Tab = {
        id: `tab-${Date.now()}`,
        paperId,
        title,
        path,
        isActive: true,
      };

      // Deactivate all other tabs
      tabs.value.forEach((t) => (t.isActive = false));
      tabs.value.push(newTab);
      activeTabId.value = newTab.id;
    }

    function removeTab(tabId: string) {
      const index = tabs.value.findIndex((t) => t.id === tabId);
      if (index === -1) return;

      const wasActive = tabs.value[index].isActive;
      tabs.value.splice(index, 1);

      if (wasActive && tabs.value.length > 0) {
        const newIndex = Math.min(index, tabs.value.length - 1);
        setActiveTab(tabs.value[newIndex].id);
      } else if (tabs.value.length === 0) {
        activeTabId.value = null;
      }
    }

    function setActiveTab(tabId: string) {
      tabs.value.forEach((t) => (t.isActive = t.id === tabId));
      activeTabId.value = tabId;
    }

    function closeOtherTabs(tabId: string) {
      tabs.value = tabs.value.filter((t) => t.id === tabId);
      setActiveTab(tabId);
    }

    function closeAllTabs() {
      tabs.value = [];
      activeTabId.value = null;
    }

    return {
      tabs,
      activeTabId,
      addTab,
      removeTab,
      setActiveTab,
      closeOtherTabs,
      closeAllTabs,
    };
  },
  {
    persist: {
      key: 'xuan-brain-tabs-storage',
      storage: localStorage,
    },
  }
);
```

---

## 阶段 5：路由系统迁移

### 5.1 路由配置

**文件**: `src/router/index.ts` (新建)

```typescript
import { createRouter, createWebHistory } from 'vue-router';
import MainLayout from '@/layouts/MainLayout.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      redirect: '/papers',
      children: [
        {
          path: 'papers',
          name: 'papers',
          component: () => import('@/pages/PapersPage.vue'),
        },
        {
          path: 'papers/:paperId',
          name: 'paper-reader',
          component: () => import('@/pages/PaperReaderPage.vue'),
        },
        {
          path: 'clips',
          name: 'clips',
          component: () => import('@/pages/ClipsPage.vue'),
        },
        {
          path: 'writing',
          name: 'writing',
          component: () => import('@/pages/WritingPage.vue'),
        },
        {
          path: 'subscriptions',
          name: 'subscriptions',
          component: () => import('@/pages/SubscriptionPage.vue'),
        },
        {
          path: 'settings',
          name: 'settings',
          component: () => import('@/pages/SettingsPage.vue'),
        },
      ],
    },
  ],
});

export default router;
```

---

## 阶段 6：国际化迁移

### 6.1 Vue I18n 配置

**文件**: `src/lib/i18n/index.ts` (重构)

```typescript
import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import zh from './locales/zh.json';

export type LocaleCode = 'en' | 'zh';

export const availableLocales: Record<
  LocaleCode,
  { name: string; nativeName: string; flag: string }
> = {
  en: { name: 'English', nativeName: 'English', flag: '🇺🇸' },
  zh: { name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
};

const STORAGE_KEY = 'xuan-brain-locale';

function getInitialLocale(): LocaleCode {
  if (typeof window === 'undefined') return 'en';
  const saved = localStorage.getItem(STORAGE_KEY) as LocaleCode | null;
  if (saved && availableLocales[saved]) return saved;
  const browserLang = navigator.language.split('-')[0] as LocaleCode;
  if (browserLang && availableLocales[browserLang]) return browserLang;
  return 'en';
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: 'en',
  messages: { en, zh },
});

export function useI18n() {
  return i18n.global;
}

export function setLocale(locale: LocaleCode) {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
}
```

### 6.2 翻译文件迁移

**源文件**:

- `src/lib/i18n/en.ts` → `src/lib/i18n/locales/en.json`
- `src/lib/i18n/zh.ts` → `src/lib/i18n/locales/zh.json`

---

## 阶段 7：组件迁移映射

### 7.1 Ant Design → Vuetify 组件对照

| Ant Design  | Vuetify 3                             | 说明     |
| ----------- | ------------------------------------- | -------- |
| Layout      | v-layout, v-navigation-drawer, v-main | 布局系统 |
| Button      | v-btn                                 | 按钮     |
| Input       | v-text-field                          | 文本输入 |
| InputNumber | v-text-field type="number"            | 数字输入 |
| Select      | v-select                              | 下拉选择 |
| TreeSelect  | v-select (自定义)                     | 树形选择 |
| Table       | v-data-table                          | 数据表格 |
| Modal       | v-dialog                              | 对话框   |
| Dropdown    | v-menu                                | 下拉菜单 |
| Tag         | v-chip                                | 标签     |
| Tabs        | v-tabs                                | 标签页   |
| Tooltip     | v-tooltip                             | 提示     |
| Form        | v-form                                | 表单     |

### 7.2 组件迁移优先级

**第一批** (核心布局):

1. `MainLayout.tsx` → `MainLayout.vue`
2. `GlobalSidebar.tsx` → `GlobalSidebar.vue`
3. `StatusBar.tsx` → `StatusBar.vue`

**第二批** (导航系统): 4. `Navigation.tsx` → `Navigation.vue` 5. `CategoryTree.tsx` → `CategoryTree.vue` (使用 v-treeview) 6. `TagsSection.tsx` → `TagsSection.vue`

**第三批** (文档功能): 7. `DocumentListMantine.tsx` → `DocumentList.vue` (使用 v-data-table-server) 8. `DocumentDetails.tsx` → `DocumentDetails.vue` 9. `DocumentToolbar.tsx` → `DocumentToolbar.vue`

**第四批** (对话框): 10. `AddCategoryDialog.tsx` → `AddCategoryDialog.vue` 11. `EditCategoryDialog.tsx` → `EditCategoryDialog.vue` 12. `AddTagDialog.tsx` → `AddTagDialog.vue`

**第五批** (页面): 13. `PapersPage.tsx` → `PapersPage.vue` 14. `PaperReaderPage.tsx` → `PaperReaderPage.vue` (使用 vue-pdf-embed) 15. 其他页面组件 (ClipsPage, WritingPage, SubscriptionPage, SettingsPage)

---

## 阶段 8：Tauri 集成保持不变

### 8.1 invokeCommand 辅助函数

**文件**: `src/lib/tauri.ts`

```typescript
/**
 * Lazy load Tauri invoke command
 * Compatible with both Tauri and browser environments
 */
export async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}
```

### 8.2 使用示例

```vue
<script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { invokeCommand } from '@/lib/tauri';

  interface PaperDto {
    id: number;
    title: string;
    authors: string[];
    // ...
  }

  const papers = ref<PaperDto[]>([]);
  const loading = ref(false);

  async function loadPapers() {
    loading.value = true;
    try {
      papers.value = await invokeCommand<PaperDto[]>('get_all_papers');
      console.info('Papers loaded successfully:', papers.value.length);
    } catch (error) {
      console.error('Failed to load papers:', error);
    } finally {
      loading.value = false;
    }
  }

  onMounted(() => {
    loadPapers();
  });
</script>
```

---

## 阶段 9：样式迁移

### 9.1 全局样式

**文件**: `src/assets/styles/main.css` (新建)

```css
/* CSS Reset */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

/* Vuetify 覆盖样式 */
.v-navigation-drawer {
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

/* 深色模式覆盖 */
.v-theme--dark {
  --v-border-color: rgba(255, 255, 255, 0.12);
  --v-background-opacity: 1;
}

/* 浅色模式覆盖 */
.v-theme--light {
  --v-border-color: rgba(0, 0, 0, 0.12);
}

/* 自定义滚动条 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
```

---

## 阶段 10：验证与测试

### 10.1 类型检查

```bash
# Vue 组件类型检查
yarn vue-tsc --noEmit

# TypeScript 类型检查
yarn tsc --noEmit
```

### 10.2 构建验证

```bash
# 开发模式
yarn tauri dev

# 生产构建
yarn build
yarn tauri build
```

### 10.3 功能验证清单

**基础功能**:

- [ ] 路由导航正常工作（页面跳转）
- [ ] 主题切换正常（亮色/暗色模式）
- [ ] 状态持久化（主题设置、标签页）

**文献管理**:

- [ ] 文档列表加载（虚拟滚动）
- [ ] 列排序功能
- [ ] 行选择功能
- [ ] 分页功能

**分类树**:

- [ ] 分类树渲染
- [ ] 节点展开/折叠
- [ ] 右键菜单操作
- [ ] 添加/编辑/删除分类

**文档详情**:

- [ ] 文档详情显示
- [ ] 编辑模式切换
- [ ] 标签添加/移除
- [ ] 表单验证

**PDF 查看**:

- [ ] PDF 文件加载
- [ ] 页面导航
- [ ] 缩放功能

**对话框**:

- [ ] 添加分类对话框
- [ ] 编辑分类对话框
- [ ] 添加标签对话框

**国际化**:

- [ ] 中英文切换
- [ ] 翻译正确显示

**Tauri 集成**:

- [ ] 所有命令正常调用
- [ ] 错误处理正确

---

## 阶段 11：迁移执行步骤

### 步骤 1：准备工作（1天）

1. 备份当前代码分支
2. 创建新的迁移分支 `feature/vue-migration`
3. 阅读本文档，熟悉所有步骤

### 步骤 2：基础设施搭建（2-3天）

1. 更新 `package.json` 依赖
2. 更新 `vite.config.ts` 配置
3. 更新 `tsconfig.json` 配置
4. 创建 `src/main.ts` 入口文件
5. 创建 `src/App.vue` 根组件
6. 运行 `yarn tauri dev` 确保基础设施正常

### 步骤 3：核心系统迁移（3-4天）

1. 创建 `src/router/index.ts`
2. 创建 Pinia stores
3. 配置 Vue I18n
4. 创建 Tauri 辅助函数
5. 测试路由和状态管理

### 步骤 4-8：组件迁移（按优先级逐步进行）

### 步骤 9：后端扩展（1天）

1. 创建 `pdf_command.rs`
2. 添加 `read_pdf_file` 命令
3. 测试 PDF 文件读取

### 步骤 10：清理和优化（2-3天）

1. 删除所有 React 相关文件
2. 清理未使用的依赖
3. 代码格式化
4. 类型检查
5. 性能优化

### 步骤 11：测试和修复（3-5天）

1. 完整功能测试
2. 修复发现的 bug
3. 跨平台测试（Windows/macOS/Linux）
4. 用户验收测试

---

## 阶段 12：风险和回滚计划

### 风险评估

| 风险                 | 影响 | 概率 | 缓解措施                              |
| -------------------- | ---- | ---- | ------------------------------------- |
| Vuetify 组件功能不足 | 高   | 中   | 提前验证关键组件，准备自定义方案      |
| PDF 查看器迁移问题   | 高   | 低   | 使用成熟库 `vue-pdf-embed`            |
| 状态管理迁移复杂度   | 中   | 中   | Pinia 与 Zustand 模式相似             |
| 虚拟滚动性能问题     | 高   | 低   | 使用 `v-data-table-server` 服务端分页 |
| 国际化迁移           | 低   | 低   | Vue I18n 成熟稳定                     |

### 回滚计划

如果迁移遇到无法解决的问题，可以：

1. **保留原分支**：`feature/pdf-headless` 分支保持可工作状态
2. **Git 回滚**：`git reset --hard` 回到迁移前的提交
3. **渐进回退**：如果已完成部分迁移，可以保留已迁移的部分

---

## 阶段 13：后续优化建议

迁移完成后，可以考虑以下优化：

1. **性能优化**:
   - 使用 `v-memo` 优化列表渲染
   - 路由懒加载（已实现）
   - 组件懒加载

2. **用户体验**:
   - 添加加载骨架屏
   - 优化错误提示
   - 添加快捷键支持

3. **功能增强**:
   - 拖拽排序分类树
   - 批量操作文档
   - 高级搜索过滤

4. **代码质量**:
   - 添加单元测试（Vitest）
   - 添加 E2E 测试（Playwright）
   - 代码覆盖率检查

---

## 关键文件变更清单

### 需要创建的文件

- `src/main.ts` - Vue 应用入口
- `src/App.vue` - 根组件
- `src/router/index.ts` - 路由配置
- `src/lib/tauri.ts` - Tauri 辅助函数
- `src/lib/vuetify.ts` - Vuetify 配置
- `src/lib/i18n/index.ts` - i18n 配置
- `src/lib/i18n/locales/en.json` - 英文翻译
- `src/lib/i18n/locales/zh.json` - 中文翻译
- `src/assets/styles/main.css` - 全局样式

### 需要修改的文件

- `vite.config.ts` - 更新插件配置
- `tsconfig.json` - 添加 Vue 支持
- `package.json` - 更新依赖
- `index.html` - 更新入口文件引用

### 需要迁移的组件 (按优先级)

1. 布局组件 (3个)
2. 导航组件 (3个)
3. 文档组件 (3个)
4. 对话框组件 (3个)
5. 页面组件 (6个)

---

## 预期工作量

- **阶段 1-3**: 基础设施和布局 (核心工作)
- **阶段 4-5**: 状态管理和路由
- **阶段 6-7**: 国际化和组件迁移
- **阶段 8-10**: 集成、样式和测试

建议采用增量迁移策略，先确保基础设施正常，再逐步迁移各个功能模块。
