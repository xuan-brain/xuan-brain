# PDF Blob 实现 - 测试指南

## 🔍 错误修复

### 问题描述
```
Failed to load PDF: invalid args `paperId` for command `read_pdf_as_blob`: 
command read_pdf_as_blob missing required key paperId
```

### 根本原因
Tauri 2.x 命令处理器对参数顺序有特定要求：
- **显式参数**（来自前端的参数）必须**放在前面**
- **依赖注入参数**（`State<T>`）必须**放在后面**

### 修复方案
修改 `read_pdf_as_blob` 函数签名：

```rust
// ❌ 错误的顺序
pub async fn read_pdf_as_blob(
    db: State<'_, DatabaseConnection>,      // ← 依赖注入在前
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,                          // ← 显式参数在后
) -> Result<PdfBlobResponse> { ... }

// ✅ 正确的顺序
pub async fn read_pdf_as_blob(
    paper_id: i64,                          // ← 显式参数在前
    db: State<'_, DatabaseConnection>,      // ← 依赖注入在后
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfBlobResponse> { ... }
```

---

## ✅ 验证修复

### 1. 编译检查
```bash
cd src-tauri
cargo check
cargo build
```
- [ ] 编译无错误
- [ ] 无警告信息

### 2. 函数测试
在 PDFViewer.vue 中，应该看到：
```
✓ Loading PDF for paper: 123
✓ Successfully loaded PDF: xxx.pdf (5.23 MB)
```

### 3. 浏览器控制台
应该显示：
```
Loading PDF for paper: 123
Successfully loaded PDF: example.pdf (5.23 MB)
```

### 4. Rust 日志
在 `logs/xuan-brain.log` 中应该看到：
```
Reading PDF as blob for paper 123
Successfully read PDF as blob for paper 123: 5461504 bytes, encoded as base64
```

---

## 📊 性能指标

### 预期性能
| 文件大小 | 加载时间 | 内存占用 |
|---------|---------|---------|
| <10MB | <500ms | ~30MB |
| 10-50MB | <2s | ~80MB |
| 50-100MB | <5s | ~150MB |
| >100MB | 10-30s | >200MB |

### 监控方法
1. **浏览器开发者工具** → Network 标签
2. **Chrome DevTools** → Performance 标签
3. **内存监控** → Memory 标签

---

## 🚀 快速验证流程

### 步骤 1: 编译
```bash
cd src-tauri
cargo build
cd ..
```

### 步骤 2: 启动开发服务
```bash
yarn dev
```

### 步骤 3: 打开 PDF
1. 打开应用
2. 选择一篇有 PDF 的论文
3. 点击打开 PDF

### 步骤 4: 检查结果
- [ ] PDF 成功加载
- [ ] 标题和大小显示正确
- [ ] 没有错误信息
- [ ] 可以翻页查看内容

---

## 🐛 常见问题排查

### 问题 1: "missing required key paperId"
**原因**: 参数顺序错误或命令未重新编译

**解决方案**:
```bash
# 清理并重新编译
cd src-tauri
cargo clean
cargo build
```

### 问题 2: "Paper not found"
**原因**: 论文 ID 不存在

**解决方案**:
1. 确认论文确实存在于数据库
2. 检查论文 ID 是否正确
3. 查看 Rust 日志获取详细错误

### 问题 3: "PDF file not found"
**原因**: PDF 文件未保存或路径错误

**解决方案**:
1. 检查文件是否存在在 `files/` 目录
2. 验证文件名和哈希值
3. 查看日志中的完整路径

### 问题 4: 加载非常慢
**原因**: 大文件或网络延迟

**解决方案**:
1. 对于大文件，考虑实现分块传输
2. 检查磁盘速度和 CPU 使用率
3. 在 Rust 日志中查看编码时间

---

## 📋 完整检查清单

- [ ] Rust 代码已编译
- [ ] `read_pdf_as_blob` 已在 `invoke_handler` 中注册
- [ ] 前端 `loadPdfAsBlob` 调用参数正确
- [ ] PDFViewer.vue 使用新的 API
- [ ] 没有 TypeScript 类型错误
- [ ] PDF 成功加载显示
- [ ] 文件大小正确显示
- [ ] 窗口标题更新正确
- [ ] 内存正确释放（关闭窗口后）
- [ ] 大文件也能正常加载

---

## 💡 性能优化建议

### 当前实现 (适用 <500MB)
- 一次性加载整个文件
- Base64 编码传输
- 简单高效

### 未来优化 (>500MB)
- 实现分块传输
- 添加进度条
- 并发加载多个分块

### 内存优化
- 使用 Worker 处理大文件
- 实现虚拟滚动
- 按需加载页面

---

## 📞 调试命令

### 查看 Rust 日志
```bash
# 实时日志（Linux/Mac）
tail -f logs/xuan-brain.log

# 实时日志（Windows）
Get-Content logs/xuan-brain.log -Wait

# 搜索特定日志
grep "read_pdf_as_blob" logs/xuan-brain.log
```

### 查看浏览器控制台
```javascript
// 在浏览器控制台运行
localStorage.setItem('debug', 'xuan-brain:*');  // 启用调试
location.reload();  // 刷新
```

### Tauri 调试
```bash
# 启用 Tauri 调试日志
set RUST_LOG=debug
# 或
export RUST_LOG=debug

# 运行
yarn dev
```

---

## 🎯 下一步

1. **验证修复有效**
   - 编译并测试
   - 确认 PDF 能正常加载

2. **性能测试**
   - 测试不同大小的 PDF
   - 监控内存使用

3. **文档更新**
   - 更新用户文档
   - 添加故障排查指南

4. **提交代码**
   - 代码审查
   - 合并到主分支
