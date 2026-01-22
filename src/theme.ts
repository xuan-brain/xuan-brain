import type { ThemeOptions } from "@mui/material/styles";

// 紧凑型桌面客户端样式配置
const compactComponents = {
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none" as const,
          fontSize: "0.8125rem",
          padding: "4px 12px",
          minHeight: "28px",
        },
        sizeSmall: {
          fontSize: "0.75rem",
          padding: "2px 8px",
          minHeight: "24px",
        },
      },
      defaultProps: {
        size: "small" as const,
      },
    },
    MuiIconButton: {
      styleOverrides: {
        root: {
          padding: "6px",
        },
        sizeSmall: {
          padding: "4px",
        },
      },
      defaultProps: {
        size: "small" as const,
      },
    },
    MuiListItem: {
      styleOverrides: {
        root: {
          paddingTop: "4px",
          paddingBottom: "4px",
        },
      },
      defaultProps: {
        dense: true,
      },
    },
    MuiListItemButton: {
      styleOverrides: {
        root: {
          paddingTop: "4px",
          paddingBottom: "4px",
          minHeight: "32px",
        },
      },
    },
    MuiListItemIcon: {
      styleOverrides: {
        root: {
          minWidth: "32px",
        },
      },
    },
    MuiListItemText: {
      styleOverrides: {
        root: {
          margin: "0",
        },
        primary: {
          fontSize: "0.8125rem",
          lineHeight: 1.4,
        },
        secondary: {
          fontSize: "0.75rem",
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          "& .MuiInputBase-root": {
            fontSize: "0.8125rem",
          },
          "& .MuiInputLabel-root": {
            fontSize: "0.8125rem",
          },
        },
      },
      defaultProps: {
        size: "small" as const,
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          height: "24px",
          fontSize: "0.75rem",
        },
        label: {
          padding: "0 8px",
        },
      },
      defaultProps: {
        size: "small" as const,
      },
    },
    MuiMenuItem: {
      styleOverrides: {
        root: {
          fontSize: "0.8125rem",
          minHeight: "32px",
          paddingTop: "4px",
          paddingBottom: "4px",
        },
      },
    },
    MuiDialog: {
      styleOverrides: {
        paper: {
          borderRadius: "6px",
        },
      },
    },
    MuiDialogTitle: {
      styleOverrides: {
        root: {
          padding: "12px 16px",
          fontSize: "0.9375rem",
          fontWeight: 600,
        },
      },
    },
    MuiDialogContent: {
      styleOverrides: {
        root: {
          padding: "12px 16px",
        },
      },
    },
    MuiDialogActions: {
      styleOverrides: {
        root: {
          padding: "8px 16px",
        },
      },
    },
    MuiTypography: {
      styleOverrides: {
        body1: {
          fontSize: "0.8125rem",
        },
        body2: {
          fontSize: "0.75rem",
        },
        caption: {
          fontSize: "0.6875rem",
        },
        h6: {
          fontSize: "0.9375rem",
        },
      },
    },
    MuiTableCell: {
      styleOverrides: {
        root: {
          padding: "8px 12px",
          fontSize: "0.8125rem",
        },
        head: {
          fontWeight: 600,
          fontSize: "0.8125rem",
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: "6px",
        },
      },
    },
  },
  spacing: 8, // 默认间距单位
  shape: {
    borderRadius: 4,
  },
};

export const lightTheme: ThemeOptions = {
  palette: {
    mode: "light",
    primary: {
      main: "#1976d2",
      light: "#42a5f5",
      dark: "#1565c0",
      contrastText: "#fff",
    },
    secondary: {
      main: "#dc004e",
      light: "#ff5c8d",
      dark: "#9a0036",
      contrastText: "#fff",
    },
    background: {
      default: "#f5f5f5",
      paper: "#ffffff",
    },
  },
  typography: {
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
    fontSize: 13, // 基础字体大小
  },
  ...compactComponents,
};

export const darkTheme: ThemeOptions = {
  palette: {
    mode: "dark",
    primary: {
      main: "#90caf9",
      light: "#b3e5fc",
      dark: "#648dae",
      contrastText: "#000",
    },
    secondary: {
      main: "#f48fb1",
      light: "#ffbee3",
      dark: "#bc5b8a",
      contrastText: "#000",
    },
    background: {
      default: "#121212",
      paper: "#1e1e1e",
    },
  },
  typography: {
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
    fontSize: 13, // 基础字体大小
  },
  ...compactComponents,
};
