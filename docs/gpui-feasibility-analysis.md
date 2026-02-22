# GPUI 可行性分析与落地方案（基于当前 dev 分支 Tauri 架构）

## 1. 当前架构现状（dev 分支）

当前项目是 **Tauri 2.x + Vue 3 + TypeScript + Vuetify + VxeTable**：

- 桌面壳与系统能力：`src-tauri/src/lib.rs`（Tauri commands、tray、window、plugin）
- 前端应用入口：`src/main.ts`（Vue + Pinia + Vuetify + VxeTable + Router）
- 页面主结构：
  - `src/layouts/MainLayout.vue`
  - `src/pages/PapersPage.vue`
  - `src/pages/PaperReaderPage.vue`
  - `src/pages/SettingsPage.vue`
- 核心前端组件：
  - 文献列表与工具栏：`src/components/paper/PaperList.vue` / `PaperToolbar.vue`
  - 分类导航：`src/components/navigation/Navigation.vue` / `CategoryTree.vue`
  - 详情与设置：`src/components/paper/PaperDetails.vue`、`src/components/settings/*`
  - 对话框：`src/components/dialogs/*`

结论：目前是典型 **WebView 前端 + Rust 后端命令层** 的 Tauri 分层。

---

## 2. 能否用 GPUI 实现？

**可以，但不是“直接替换组件库”的级别，而是“前端技术栈重写”的级别。**

GPUI 是 Rust 原生 UI 框架，不是 Vue 组件库。当前 Vue 组件（Vuetify、VxeTable、Router、Pinia）不能直接复用，需要改写为 GPUI 组件与状态模型。

### 可行性判断

- ✅ **业务能力可复用**：现有 Rust 数据库、服务、命令逻辑可继续使用（尤其 `src-tauri/src/command`、`service`、`database`、`papers`）。
- ⚠️ **前端不可直接复用**：`*.vue` 组件、前端路由/状态管理需要重构。
- ⚠️ **PDF/富交互需要专项方案**：当前依赖 Web 技术生态（viewer/annotation），迁移到 GPUI 需独立验证渲染与标注能力。

因此：**技术上可行，工程上属于中高成本迁移。**

---

## 3. 前端组件级迁移评估（重点）

| 当前模块 | 现状实现 | GPUI 迁移可行性 | 迁移难度 | 说明 |
| --- | --- | --- | --- | --- |
| MainLayout 三栏/可拖拽宽度 | `MainLayout.vue` + 本地状态 | ✅ 可行 | 中 | GPUI 原生布局 + 拖拽手柄可重建 |
| 文献列表表格（排序、展开、右键） | `PaperList.vue` + VxeTable | ✅ 可行 | 中-高 | 需自行实现表格交互（虚拟滚动/上下文菜单） |
| 分类树 | `CategoryTree.vue` | ✅ 可行 | 中 | 树结构与拖拽重排需要自定义组件 |
| 设置页多 Tab | `SettingsPage.vue` + `settings/*` | ✅ 可行 | 低-中 | 表单型 UI 迁移相对简单 |
| 导入/编辑弹窗 | `dialogs/*` | ✅ 可行 | 低-中 | 映射为 GPUI modal/dialog |
| 状态栏/侧边栏 | `StatusBar.vue`/`GlobalSidebar.vue` | ✅ 可行 | 低 | 纯展示组件 |
| PDF 阅读与标注 | `PDFViewer.vue` + Web 生态 | ⚠️ 有条件可行 | 高 | 需先验证 GPUI 侧 PDF 渲染与批注能力 |
| i18n/主题 | vue-i18n + Vuetify theme | ✅ 可行 | 中 | 需重建语言资源加载与主题变量系统 |

---

## 4. 推荐落地路线（具体方案）

## 目标原则

1. **先复用 Rust 业务，再替换 UI。**
2. **分阶段双轨，避免一次性替换。**
3. **先迁“文献管理主流程”，后迁“PDF 标注”等高风险功能。**

### Phase 0：后端能力解耦（1~2 周）

- 将可复用业务逻辑从 Tauri command 边界进一步下沉到独立 Rust service（已具备基础）。
- 为 GPUI 前端准备统一接口层（可直接调用 service，或保持 command 风格的 facade）。
- 输出接口清单：papers/category/labels/config/import。

**验收**：不改功能，仅完成“前端无关”的 Rust 业务边界。

### Phase 1：GPUI 应用骨架（1~2 周）

- 新建 GPUI 桌面壳（建议与现有仓库同 mono-repo，新增 `apps/gpui-desktop`）。
- 接入全局状态、主题、快捷键、窗口生命周期。
- 先实现 MainLayout（左导航 + 列表区 + 详情区空壳）。

**验收**：可启动、可切换页面、可读取基础配置。

### Phase 2：核心功能迁移（3~6 周）

优先级顺序：

1. 分类树 + 文献列表 + 详情面板（MVP 主流程）
2. 导入对话框（DOI/arXiv/PubMed/PDF）
3. 设置页（系统/AI/文献）
4. 右键菜单、拖拽、排序、批量操作

**验收**：与当前 `PapersPage`、`Navigation`、`SettingsPage` 主能力等价。

### Phase 3：PDF 与高级交互（2~6 周，取决于技术路线）

- 先做阅读（跳页、缩放、目录）
- 再做标注（高亮、注释、保存）
- 若原生 PDF 路线风险过高，可保留“独立 WebView 阅读器”作为过渡

**验收**：阅读器达到当前可用水平，标注链路闭环。

### Phase 4：切换与收敛（1~2 周）

- 完成配置迁移、用户数据兼容
- 双端灰度（Vue 端与 GPUI 端并行一段周期）
- 达标后下线旧前端

---

## 5. 架构建议（避免大重写风险）

建议将项目抽象为三层：

1. **Domain/Core（Rust）**：数据模型、业务规则、数据库访问（可复用）
2. **Adapter（Rust）**：Tauri command / GPUI action 两套适配层
3. **Presentation（Vue or GPUI）**：纯 UI 层

这样可以让 GPUI 与 Tauri-Vue 在一段时间内共存，显著降低迁移失败风险。

---

## 6. 风险与应对

1. **PDF/标注能力缺口**（最高风险）
   - 应对：先做阅读后做标注；必要时保留过渡 WebView。
2. **表格交互工作量大**（VxeTable 替代）
   - 应对：先交付 MVP（排序/选择/展开），虚拟滚动后置。
3. **迁移周期影响迭代节奏**
   - 应对：双轨开发，优先迁核心路径，非核心功能延后。

---

## 7. 结论

- **结论一**：从技术上，项目功能（含大部分前端组件）可以迁移到 GPUI。  
- **结论二**：这不是“组件替换”，而是“前端重构 + 架构分层优化”。  
- **结论三**：推荐采用“分阶段、双轨并行、先主流程后 PDF 标注”的方案，以最小业务风险推进。

如果你同意，我可以下一步直接给出 **按当前文件结构映射的详细任务拆解清单（到组件/命令级）**，用于创建实际迭代计划（Sprint Backlog）。
