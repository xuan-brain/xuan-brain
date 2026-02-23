# Vue 3 前端 (src)

**职责:** UI、状态管理、路由、国际化

## 结构

```
src/
├── main.ts             # 入口，Vuetify/VxeTable 配置
├── App.vue             # 根组件
├── components/         # UI 组件
│   ├── paper/          # 文献相关（List, Details, Toolbar）
│   ├── navigation/     # 导航（CategoryTree, Navigation）
│   ├── settings/       # 设置页面组件
│   ├── dialogs/        # 对话框
│   ├── layout/         # 布局（StatusBar, GlobalSidebar）
│   └── pdf/            # PDF 查看器
├── pages/              # 路由页面
├── stores/             # Pinia 状态
├── lib/                # 工具
│   ├── api/            # Tauri 命令调用
│   └── i18n/           # 国际化
├── router/             # Vue Router
├── layouts/            # 布局模板
└── assets/             # 静态资源
```

## 调用后端

```typescript
// 使用惰性加载辅助函数
async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}
```

## 状态管理

```typescript
// Pinia + 持久化
export const useAppStore = defineStore(
  'app',
  () => {
    const isDark = ref(true);
    return { isDark };
  },
  { persist: { storage: localStorage } }
);
```

## 关键文件

| 文件                    | 用途                        |
| ----------------------- | --------------------------- |
| `main.ts`               | Vuetify、VxeTable、主题配置 |
| `stores/useAppStore.ts` | 全局状态                    |
| `lib/api/*.ts`          | 后端 API 封装               |
| `router/index.ts`       | 路由定义                    |

## UI 组件

- Vuetify 3（v-treeview, v-data-table）
- VxeTable（数据表格）
- Tailwind CSS（工具类）

## 代码规范

- 组合式 API + `<script setup>`
- 禁止 `as any` / `@ts-ignore`
- 禁止模块顶层调用 Tauri API
- 日志用 `console.info`/`console.error`
