# 主色调功能说明

## 功能概述

在左下角添加了主色调修改功能，用户可以自定义应用的主色调，提升个性化体验。

## 使用方法

### 1. 主色调选择器

位置：左侧边栏底部，Dark Mode 开关旁边

**按钮样式**：
- 圆形按钮（28x28px）
- 显示当前选中的主色调背景色
- 白色调色板图标（Palette icon from Lucide）

### 2. 可选颜色

提供 8 种预设主色调：

| 颜色名称 | 色值 | Tailwind 类名 |
|---------|------|--------------|
| Blue    | #3b82f6 | bg-blue-500 |
| Purple  | #a855f7 | bg-purple-500 |
| Pink    | #ec4899 | bg-pink-500 |
| Red     | #ef4444 | bg-red-500 |
| Orange  | #f97316 | bg-orange-500 |
| Green   | #22c55e | bg-green-500 |
| Teal    | #14b8a6 | bg-teal-500 |
| Cyan    | #06b6d4 | bg-cyan-500 |

### 3. 操作流程

1. 点击左下角的圆形主色调按钮
2. 在弹出菜单中选择你喜欢的颜色
3. 颜色会立即应用到整个应用
4. 选择会自动保存到 localStorage

## 技术实现

### CSS 变量

主色调通过 CSS 自定义属性实现：

```css
:root {
  --accent-color: #3b82f6;           /* 主色调 */
  --accent-color-hover: #2563eb;     /* 悬停时的颜色（自动计算） */
}
```

### 使用主色调的 CSS 类

在组件中可以使用以下工具类：

- `.bg-accent` - 主色调背景
- `.bg-accent-hover` - 主色调悬停背景
- `.text-accent` - 主色调文字
- `.border-accent` - 主色调边框
- `.hover:bg-accent-hover:hover` - 悬停时主色调背景

### JavaScript API

也可以在代码中直接使用：

```javascript
// 获取当前主色调
const accentColor = getComputedStyle(document.documentElement)
  .getPropertyValue('--accent-color');

// 动态设置主色调
document.documentElement.style.setProperty('--accent-color', '#ff0000');
```

### 自动持久化

主色调选择会自动保存到浏览器的 localStorage：

```javascript
localStorage.setItem("accent-color", "#3b82f6");
```

页面刷新后会自动恢复上次选择的主色调。

### 亮度调整

系统会自动计算悬停时的颜色，比主色调深 10%：

```javascript
function adjustBrightness(color, percent) {
  // 颜色亮度计算算法
  // ...
}
```

## 设计规范

### 颜色选择器

- **布局**：4x2 网格布局
- **尺寸**：每个颜色按钮 24x24px
- **间距**：6px (gap-1.5)
- **圆角**：完全圆形
- **边框**：2px，选中时加深显示
- **悬停效果**：放大 1.1 倍 (scale-110)

### 下拉菜单

- **宽度**：128px (w-32)
- **内边距**：8px (p-2)
- **背景**：白色/暗色（根据主题）
- **阴影**：lg 级别阴影
- **位置**：按钮上方，底部对齐

### 主按钮

- **尺寸**：28x28px (w-7 h-7)
- **圆角**：完全圆形
- **边框**：2px 灰色边框
- **图标**：12px Lucide Palette 图标
- **背景**：当前主色调
- **图标颜色**：白色带阴影

## 浏览器兼容性

- ✅ Chrome/Edge 88+
- ✅ Firefox 85+
- ✅ Safari 14+
- ✅ 支持所有现代浏览器

## 示例效果

选择不同主色调后，以下元素会自动更新：

1. 按钮悬停效果
2. 导航项悬停背景
3. 选中状态的边框和背景
4. 链接颜色
5. 焦点环颜色

## 注意事项

1. 主色调是全局设置，会影响整个应用
2. 颜色变化有平滑过渡动画（0.3s ease）
3. 主题切换不会影响主色调选择
4. 主色调与 Skeleton 主题系统独立工作
