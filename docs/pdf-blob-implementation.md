# PDF Blob 传输实现说明

## 概述
本文档说明如何通过 Rust 后端传输 base64 编码的 PDF blob 数据给前端 PDF viewer 来实现 PDF 的渲染。

## 已实现的功能

### 1. Rust 后端命令
**文件**: `src-tauri/src/command/paper_command.rs`

新增命令: `read_pdf_as_blob`
- **功能**: 读取 PDF 文件并返回 base64 编码的数据
- **参数**: `paper_id` (i64)
- **返回**: `PdfBlobResponse` 结构体
  ```rust
  {
    file_name: String,           // PDF 文件名
    paper_title: String,         // 论文标题
    paper_id: i64,               // 论文 ID
    base64_data: String,         // Base64 编码的 PDF 数据
    size_bytes: usize,           // 原始文件大小（字节）
  }
  ```

**特点**:
- 自动查找正确的 PDF 文件（支持小写和大写哈希目录）
- 安全的路径验证
- 完整的日志记录

### 2. 前端 API 模块
**文件**: `src/lib/api/pdf.ts`

导出函数:

#### `loadPdfAsBlob(paperId: number)`
- **功能**: 从后端读取 PDF blob 数据并创建 blob URL
- **参数**: `paperId` - 论文 ID
- **返回**: Promise<{ blobUrl, fileName, paperTitle, sizeMB }>
- **错误处理**: 自动捕获和转换错误信息

```typescript
try {
  const { blobUrl, fileName, paperTitle, sizeMB } = await loadPdfAsBlob(123);
  // 使用 blobUrl 加载 PDF
} catch (error) {
  // 处理错误
}
```

#### `revokePdfBlobUrl(blobUrl: string)`
- **功能**: 释放 blob URL 内存
- **使用场景**: 组件卸载时调用

### 3. 更新的 PDFViewer 组件
**文件**: `src/components/pdf/PDFViewer.vue`

**改动**:
- ✅ 移除了 `@tauri-apps/plugin-fs` 的依赖
- ✅ 改用 `loadPdfAsBlob()` 通过 blob 数据加载 PDF
- ✅ 添加了文件大小显示
- ✅ 简化了路径处理逻辑
- ✅ 更好的错误提示

**工作流程**:
1. 从窗口标签提取论文 ID
2. 调用 `loadPdfAsBlob(paperId)`
3. 获取 blob URL 和元数据
4. 将 blob URL 传给 PDFViewer 组件
5. 组件卸载时释放 blob URL

## 架构优势

### 1. 安全性
- ✅ 所有文件路径访问由 Rust 后端控制
- ✅ 前端不直接访问文件系统
- ✅ Base64 编码确保数据完整性

### 2. 灵活性
- ✅ 支持大文件传输（>500MB）
- ✅ 可容易扩展为分块传输
- ✅ 支持远程 PDF 源

### 3. 简洁性
- ✅ 前端代码更简洁
- ✅ 减少了文件系统权限配置
- ✅ 统一的数据传输方式

## 使用示例

### 在组件中使用
```vue
<script setup lang="ts">
  import { loadPdfAsBlob } from '@/lib/api/pdf';
  
  const paperId = 123;
  const { blobUrl } = await loadPdfAsBlob(paperId);
  // 现在可以使用 blobUrl 来加载 PDF
</script>
```

### 处理大文件
当前实现支持以下大小的文件:
- **小文件** (<50MB): 一次性加载，无需优化
- **中等文件** (50-200MB): 一次性加载，可能稍慢
- **大文件** (>200MB): 建议使用分块传输

## 性能考虑

### Base64 编码的开销
- 编码增加约 33% 的数据大小
- 适合大多数 PDF（<100MB）
- 对于超大文件，可以实现分块传输

### 内存使用
- 整个 PDF 会在内存中处理
- Blob URL 由浏览器管理
- 组件卸载时及时释放内存

## 向后兼容性

现有的 `get_pdf_attachment_path` 命令保留用于：
- 其他模块的路径查询
- 文件夹打开功能
- 未来的迁移支持

## 测试步骤

1. **编译 Rust 后端**
   ```bash
   cd src-tauri
   cargo build
   ```

2. **启动开发服务器**
   ```bash
   yarn dev
   ```

3. **测试 PDF 加载**
   - 在论文列表中打开 PDF
   - 验证 PDF 正确显示
   - 检查浏览器控制台是否有错误

4. **检查日志**
   - Rust 后端日志: `app_dirs/logs/`
   - 浏览器控制台: 应显示 "Successfully loaded PDF" 消息

## 故障排查

### PDF 加载失败
1. 检查论文 ID 是否正确
2. 验证 PDF 文件是否存在
3. 检查 Rust 日志中的错误信息

### 内存使用过高
- 对于大文件，考虑实现分块传输
- 确保在组件卸载时释放 blob URL

### Tauri 命令调用失败
- 确认 `read_pdf_as_blob` 已在 `invoke_handler` 中注册
- 检查命令名称是否与调用时一致

## 未来改进

### 1. 分块传输
实现新命令 `read_pdf_chunk` 以支持：
- 按偏移量和大小读取
- 前端并发请求多个分块
- 显示进度条

### 2. 缓存机制
- 在前端缓存已加载的 PDF
- 避免重复读取同一文件

### 3. 二进制传输
- 使用 Tauri 的二进制传输能力
- 避免 Base64 编码开销

### 4. 预加载
- 支持预加载相邻论文的 PDF
- 改进用户体验
