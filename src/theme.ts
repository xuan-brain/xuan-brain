import type { ThemeConfig } from "antd";
import { theme } from "antd";

/**
 * Ant Design 主题配置
 * 紧凑型桌面客户端样式，适用于文献管理应用
 */

// 全局字体配置
const fontFamily = [
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
].join(",");

/**
 * 亮色主题配置
 */
export const lightTheme: ThemeConfig = {
  token: {
    // 主色调
    colorPrimary: "#1976d2",
    colorSuccess: "#52c41a",
    colorWarning: "#faad14",
    colorError: "#ff4d4f",
    colorInfo: "#1976d2",

    // 字体配置
    fontFamily,
    fontSize: 15,

    // 圆角
    borderRadius: 4,

    // 间距
    paddingXS: 8,
    paddingSM: 12,
    padding: 16,
    paddingMD: 20,
    paddingLG: 24,

    // 组件大小
    controlHeight: 32,
    controlHeightSM: 24,
    controlHeightLG: 40,

    // 颜色
    colorBgBase: "#ffffff",
    colorBgContainer: "#ffffff",
    colorBgElevated: "#ffffff",
    colorBgLayout: "#f5f5f5",
    colorBgSpotlight: "#ffffff",
    colorPrimaryBgHover: "rgba(24, 144, 255, 0.15)",
  },
  components: {
    // 按钮组件
    Button: {
      controlHeight: 32,
      paddingInline: 12,
      fontSize: 15,
    },

    // 输入框组件
    Input: {
      controlHeight: 32,
      paddingInline: 12,
      fontSize: 13,
    },

    // 标签/Tag 组件
    Tag: {
      borderRadius: 4,
      fontSize: 12,
    },

    // 表格组件
    Table: {
      fontSize: 13,
      padding: 12,
      paddingSM: 8,
      paddingXS: 4,
      headerBg: "#fafafa",
      headerColor: "rgba(0, 0, 0, 0.88)",
    },

    // 模态框组件
    Modal: {
      borderRadiusLG: 6,
      padding: 16,
      paddingLG: 24,
    },

    // 菜单组件
    Menu: {
      itemBorderRadius: 4,
      paddingXS: 12,
      fontSize: 13,
    },

    // 排版组件
    Typography: {
      fontSize: 13,
      fontSizeHeading1: 32,
      fontSizeHeading2: 24,
      fontSizeHeading3: 20,
      fontSizeHeading4: 16,
      fontSizeHeading5: 15,
    },
  },
};

/**
 * 暗色主题配置
 */
export const darkTheme: ThemeConfig = {
  token: {
    // 主色调（暗色模式下使用浅色调）
    colorPrimary: "#90caf9",
    colorSuccess: "#73d13d",
    colorWarning: "#ffc53d",
    colorError: "#ff7875",
    colorInfo: "#40a9ff",

    // 字体配置
    fontFamily,
    fontSize: 15,

    // 圆角
    borderRadius: 4,

    // 间距
    paddingXS: 8,
    paddingSM: 12,
    padding: 16,
    paddingMD: 20,
    paddingLG: 24,

    // 组件大小
    controlHeight: 32,
    controlHeightSM: 24,
    controlHeightLG: 40,

    // 颜色
    colorBgBase: "#141414",
    colorBgContainer: "#1f1f1f",
    colorBgElevated: "#262626",
    colorBgLayout: "#141414",
    colorBgSpotlight: "#262626",
    colorBorder: "#424242",
    colorBorderSecondary: "#303030",
    colorText: "rgba(255, 255, 255, 0.85)",
    colorTextSecondary: "rgba(255, 255, 255, 0.65)",
    colorTextTertiary: "rgba(255, 255, 255, 0.45)",
    colorPrimaryBgHover: "rgba(24, 144, 255, 0.25)",
  },
  components: {
    // 按钮组件
    Button: {
      controlHeight: 32,
      paddingInline: 12,
      fontSize: 15,
    },

    // 输入框组件
    Input: {
      controlHeight: 32,
      paddingInline: 12,
      fontSize: 13,
      colorBgContainer: "#1f1f1f",
      colorBorder: "#424242",
    },

    // 标签/Tag 组件
    Tag: {
      borderRadius: 4,
      fontSize: 12,
    },

    // 表格组件
    Table: {
      fontSize: 13,
      padding: 12,
      paddingSM: 8,
      paddingXS: 4,
      headerBg: "#1f1f1f",
      headerColor: "rgba(255, 255, 255, 0.85)",
      colorBgContainer: "#1f1f1f",
    },

    // 模态框组件
    Modal: {
      borderRadiusLG: 6,
      padding: 16,
      paddingLG: 24,
      contentBg: "#1f1f1f",
    },

    // 下拉菜单组件
    Dropdown: {
      borderRadius: 4,
    },

    // 菜单组件
    Menu: {
      itemBorderRadius: 4,
      paddingXS: 12,
      fontSize: 13,
      colorBgContainer: "#1f1f1f",
      colorBgElevated: "#262626",
    },

    // 排版组件
    Typography: {
      fontSize: 13,
      fontSizeHeading1: 32,
      fontSizeHeading2: 24,
      fontSizeHeading3: 20,
      fontSizeHeading4: 16,
      fontSizeHeading5: 15,
      colorText: "rgba(255, 255, 255, 0.85)",
      colorTextSecondary: "rgba(255, 255, 255, 0.65)",
    },

    // 卡片组件
    Card: {
      colorBgContainer: "#1f1f1f",
    },
  },
  algorithm: theme.darkAlgorithm,
};

/**
 * 创建带动态主色的主题
 */
export function createDynamicTheme(
  accentColor: string,
  isDark: boolean,
): ThemeConfig {
  const baseTheme = isDark ? darkTheme : lightTheme;

  return {
    ...baseTheme,
    token: {
      ...baseTheme.token,
      colorPrimary: accentColor,
      colorPrimaryBgHover: isDark
        ? "rgba(24, 144, 255, 0.25)"
        : "rgba(24, 144, 255, 0.15)",
    },
    algorithm: isDark ? theme.darkAlgorithm : undefined,
  };
}
