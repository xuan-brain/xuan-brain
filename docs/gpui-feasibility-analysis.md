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

### 目标原则

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

---

## 8. 参考 `longbridge/gpui-component` 后，仍需自定义实现的 UI 控件清单

基于 `gpui-component` 当前公开模块（如 `button/dialog/menu/select/input/table/tree/resizable/sidebar/tab/color_picker` 等）与当前 `xuan-brain` 前端组件对照，以下能力建议作为**自定义控件**实现：

### P0（必须先做，主流程阻塞）

1. **文献工作台分栏控件（LibraryWorkbench）**
   - 对应现状：`MainLayout.vue` + `PapersPage.vue`
   - 原因：虽然有 `resizable` 与 `sidebar`，但“三栏联动 + 宽度持久化 + 路由态同步”是业务特化容器。

2. **文献表格控件（PaperTable）**
   - 对应现状：`PaperList.vue`（VxeTable）
   - 原因：需要组合实现“行双击打开阅读器 + 行右键菜单 + Trash/Library 双模式 + 附件展开行 + 标签列渲染”，属于领域表格而非通用 Table。

3. **分类树管理控件（CategoryTreeManager）**
   - 对应现状：`CategoryTree.vue`
   - 原因：除 Tree 展示外，还要支持拖拽重排后调用 `reorder_tree`、节点右键增删改、选中/取消选中联动筛选。

4. **标签面板控件（TagPalettePanel）**
   - 对应现状：`Navigation.vue` 标签区域
   - 原因：需要“标签 chip 列表 + 右键菜单 + 颜色面板 + 与后端 label API 联动”，是业务化组合控件。

### P1（高价值，建议第二阶段）

5. **文献详情编辑器（PaperDetailEditor）**
   - 对应现状：`PaperDetails.vue`
   - 原因：包含 metadata 表单、分类绑定、标签增删、阅读状态切换与保存链路，属于复杂业务表单。

6. **文献导入工具条（PaperImportToolbar）**
   - 对应现状：`PaperToolbar.vue`
   - 原因：整合 DOI/arXiv/PubMed/PDF 多入口导入弹窗、输入校验、异步反馈与刷新联动。

7. **状态栏服务切换器（ServiceSwitcherStatusBar）**
   - 对应现状：`StatusBar.vue`
   - 原因：需要“LLM Provider/GROBID 选择 + 默认项切换 + 配置持久化”，是应用级状态组件。

### P2（按产品节奏推进）

8. **PDF 阅读器壳层控件（PdfReaderShell）**
   - 对应现状：`PDFViewer.vue`
   - 原因：即使底层继续使用第三方 PDF 渲染引擎（如现有 Web/PDF 插件或后续 Rust PDF 库），也需要自定义“文档加载状态/错误态/保存流程/窗口行为”壳层。

9. **批量导入反馈控件（ImportTaskToast/Panel）**
   - 对应现状：当前散落在多个对话框与 console
   - 原因：迁移到桌面原生后建议统一异步任务反馈（进度、成功/失败、重试）。

10. **统一错误与空态控件（DomainEmptyState/DomainErrorState）**
    - 对应现状：各组件内分散实现
    - 原因：分类空态、列表空态、加载失败态可抽为统一业务控件，减少重复逻辑。

### 可直接复用/轻封装（一般不需要重写为“新控件”）

- 通用按钮、输入框、下拉、菜单、对话框、Tooltip、Switch、Tabs、基础 Tree/Table/Resizable 等，优先使用 `gpui-component` 原生组件，并仅做主题样式与品牌设计（颜色、圆角、间距、字体）层面的统一封装。
