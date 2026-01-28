import type { ThemeOptions } from "@mui/material/styles";

/**
 * 紧凑型桌面客户端样式配置
 * 适用于文献管理应用，注重信息密度和可读性
 * 基础字体大小：15px，适合桌面应用阅读
 */
const compactComponents = {
  components: {
    /**
     * 按钮组件样式配置
     * - 默认使用 small 尺寸以节省空间
     * - 取消自动大写转换，使用原始文本大小写
     * - 字体大小：常规 15px，small 13px
     */
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none" as const, // 不自动转换为大写
          fontSize: "15px", // 常规按钮字体
          padding: "4px 12px", // 紧凑的内边距
          minHeight: "32px", // 最小高度确保可点击区域
        },
        sizeSmall: {
          fontSize: "13px", // 小号按钮字体
          padding: "2px 8px", // 更紧凑的内边距
          minHeight: "24px", // 更小的最小高度
        },
      },
      defaultProps: {
        size: "small" as const, // 默认使用小尺寸
      },
    },

    /**
     * 图标按钮组件样式配置
     * - 默认使用 small 尺寸
     * - 紧凑的内边距以节省空间
     */
    MuiIconButton: {
      styleOverrides: {
        root: {
          padding: "6px", // 常规图标按钮内边距
        },
        sizeSmall: {
          padding: "4px", // 小号图标按钮内边距
        },
      },
      defaultProps: {
        size: "small" as const, // 默认使用小尺寸
      },
    },

    /**
     * 列表项组件样式配置
     * - 使用 dense 模式减少垂直间距
     * - 适合显示大量列表项
     */
    MuiListItem: {
      styleOverrides: {
        root: {
          paddingTop: "4px", // 减少上边距
          paddingBottom: "4px", // 减少下边距
        },
      },
      defaultProps: {
        dense: true, // 启用紧凑模式
      },
    },

    /**
     * 列表项按钮组件样式配置
     * - 用于导航栏、菜单等可点击列表项
     * - 高度：36px，配合 15px 字体保持良好比例
     */
    MuiListItemButton: {
      styleOverrides: {
        root: {
          paddingTop: "6px", // 上下边距
          paddingBottom: "6px",
          minHeight: "36px", // 最小高度，确保可点击区域
        },
      },
    },

    /**
     * 列表项图标容器样式配置
     * - 最小宽度 36px 确保图标对齐
     * - 图标大小 20px (1.25rem) 与文字协调
     */
    MuiListItemIcon: {
      styleOverrides: {
        root: {
          minWidth: "36px", // 图标容器最小宽度
          "& .MuiSvgIcon-root": {
            fontSize: "1.25rem", // 图标大小（约 20px）
          },
        },
      },
    },

    /**
     * 列表项文本样式配置
     * - 主文本：15px，与导航文字一致
     * - 次要文本：12px，用于补充说明
     */
    MuiListItemText: {
      styleOverrides: {
        root: {
          margin: "0", // 移除默认边距
        },
        primary: {
          fontSize: "15px", // 主文本字体大小
          lineHeight: 1.4, // 行高，确保可读性
        },
        secondary: {
          fontSize: "12px", // 次要文本字体大小
        },
      },
    },

    /**
     * 文本输入框组件样式配置
     * - 字体大小 13px，与表单组件保持一致
     * - 默认使用 small 尺寸
     */
    MuiTextField: {
      styleOverrides: {
        root: {
          "& .MuiInputBase-root": {
            fontSize: "13px", // 输入文字字体
          },
          "& .MuiInputLabel-root": {
            fontSize: "13px", // 标签字体
          },
        },
      },
      defaultProps: {
        size: "small" as const, // 默认使用小尺寸
      },
    },

    /**
     * 标签芯片组件样式配置
     * - 用于显示分类、标签等
     * - 高度 24px，字体 12px
     */
    MuiChip: {
      styleOverrides: {
        root: {
          height: "24px", // 芯片高度
          fontSize: "12px", // 字体大小
        },
        label: {
          padding: "0 8px", // 文字内边距
        },
      },
      defaultProps: {
        size: "small" as const, // 默认使用小尺寸
      },
    },

    /**
     * 菜单项组件样式配置
     * - 用于下拉菜单
     * - 字体大小 13px，高度 32px
     */
    MuiMenuItem: {
      styleOverrides: {
        root: {
          fontSize: "13px", // 菜单项字体
          minHeight: "32px", // 最小高度
          paddingTop: "4px",
          paddingBottom: "4px",
        },
      },
    },

    /**
     * 对话框组件样式配置
     * - 圆角 6px，现代简洁风格
     */
    MuiDialog: {
      styleOverrides: {
        paper: {
          borderRadius: "6px", // 对话框圆角
        },
      },
    },

    /**
     * 对话框标题组件样式配置
     * - 字体 15px，加粗显示
     * - 内边距 12px 16px
     */
    MuiDialogTitle: {
      styleOverrides: {
        root: {
          padding: "12px 16px",
          fontSize: "15px",
          fontWeight: 600, // 标题加粗
        },
      },
    },

    /**
     * 对话框内容区域样式配置
     * - 内边距 12px 16px
     */
    MuiDialogContent: {
      styleOverrides: {
        root: {
          padding: "12px 16px",
        },
      },
    },

    /**
     * 对话框操作按钮区域样式配置
     * - 内边距 8px 16px，通常用于确认/取消按钮
     */
    MuiDialogActions: {
      styleOverrides: {
        root: {
          padding: "8px 16px",
        },
      },
    },

    /**
     * 排版组件样式配置
     * - 定义不同文本变体的字体大小
     */
    MuiTypography: {
      styleOverrides: {
        body1: {
          fontSize: "13px", // 正文文本
        },
        body2: {
          fontSize: "13px", // 次要正文（与 body1 相同以统一）
        },
        caption: {
          fontSize: "12px", // 说明文字
        },
        h6: {
          fontSize: "15px", // 小标题（与 body1 不同，通常用于章节标题）
        },
      },
    },

    /**
     * 表格单元格组件样式配置
     * - 字体 13px，内边距 8px 12px
     * - 表头加粗显示
     */
    MuiTableCell: {
      styleOverrides: {
        root: {
          padding: "8px 12px",
          fontSize: "13px",
        },
        head: {
          fontWeight: 600, // 表头加粗
          fontSize: "13px",
        },
      },
    },

    /**
     * 纸张组件样式配置
     * - 圆角 6px，用于卡片、对话框等容器
     */
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: "6px", // 纸张圆角
        },
      },
    },

    /**
     * 树形项组件样式配置
     * - 用于分类树、文件树等
     * - 字体 13px，图标 19px
     */
    MuiTreeItem: {
      styleOverrides: {
        content: {
          padding: "4px 0", // 内容上下边距
        },
        label: {
          fontSize: "13px", // 树节点文本字体
        },
        iconContainer: {
          "& .MuiSvgIcon-root": {
            fontSize: "19px", // 展开/折叠图标大小
          },
        },
      },
    },
  },
  spacing: 8, // 默认间距单位（8px 的倍数）
  shape: {
    borderRadius: 4, // 默认圆角大小
  },
};

/**
 * 亮色主题配置
 * - 使用 Material Design 默认蓝色主色调
 * - 浅色背景：#f5f5f5
 * - 纸张背景：#ffffff
 */
export const lightTheme: ThemeOptions = {
  palette: {
    mode: "light", // 亮色模式
    primary: {
      main: "#1976d2", // 主蓝色
      light: "#42a5f5", // 浅蓝色
      dark: "#1565c0", // 深蓝色
      contrastText: "#fff", // 文字颜色
    },
    secondary: {
      main: "#dc004e", // 主粉色
      light: "#ff5c8d", // 浅粉色
      dark: "#9a0036", // 深粉色
      contrastText: "#fff", // 文字颜色
    },
    background: {
      default: "#f5f5f5", // 默认背景色
      paper: "#ffffff", // 纸张/卡片背景色
    },
  },
  typography: {
    // 使用系统默认字体栈，确保在各平台上的最佳显示效果
    fontFamily: [
      "-apple-system", // macOS/iOS
      "BlinkMacSystemFont", // Chrome
      '"Segoe UI"', // Windows
      "Roboto", // Android/Material Design
      '"Helvetica Neue"', // iOS
      "Arial", // 通用无衬线字体
      "sans-serif",
      '"Apple Color Emoji"', // Emoji 表情
      '"Segoe UI Emoji"', // Windows Emoji
      '"Segoe UI Symbol"', // 符号
      '"Segoe UI Symbol"',
    ].join(","),
    fontSize: 15, // 基础字体大小（rem单位，约15px），适合桌面应用阅读
  },
  ...compactComponents, // 应用紧凑型组件样式
};

/**
 * 暗色主题配置
 * - 使用浅色调主色以确保在暗色背景上的对比度
 * - 深色背景：#121212
 * - 纸张背景：#1e1e1e
 */
export const darkTheme: ThemeOptions = {
  palette: {
    mode: "dark", // 暗色模式
    primary: {
      main: "#90caf9", // 主浅蓝色
      light: "#b3e5fc", // 浅蓝色
      dark: "#648dae", // 深蓝色
      contrastText: "#000", // 文字颜色（黑色以增加对比度）
    },
    secondary: {
      main: "#f48fb1", // 主浅粉色
      light: "#ffbee3", // 浅粉色
      dark: "#bc5b8a", // 深粉色
      contrastText: "#000", // 文字颜色
    },
    background: {
      default: "#121212", // 默认背景色（Material Design 暗色主题标准）
      paper: "#1e1e1e", // 纸张/卡片背景色
    },
  },
  typography: {
    // 使用与亮色主题相同的字体栈
    fontFamily: [
      "-apple-system",
      "BlinkMacSystemFont",
      '"Segoe UI"',
      "Roboto",
      '"Helvetica Neue"',
      "Arial",
      "sans-serif",
      '"Apple Color Emoji"',
      '"Segoe UI Emoji"',
      '"Segoe UI Symbol"',
    ].join(","),
    fontSize: 15, // 基础字体大小（rem单位，约15px），与亮色主题保持一致
  },
  ...compactComponents, // 应用紧凑型组件样式
};
