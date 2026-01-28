import { useState, useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import { CssBaseline, Box } from "@mui/material";
import { lightTheme, darkTheme } from "./theme";
import { I18nProvider } from "./lib/i18n";
import Layout from "./components/layout/Layout";

function App() {
  const [isDark, setIsDark] = useState(true);
  const [accentColor, setAccentColor] = useState("#3b82f6");
  const [selectedDocument, setSelectedDocument] = useState<{
    id: number;
    title: string;
    authors: string[];
    year: number;
    abstract?: string;
    keywords?: string[];
    fileType?: string;
    fileSize?: string;
    addedDate?: string;
    tags?: { id: number; name: string; color: string }[];
  } | null>(null);

  // Initialize theme from localStorage and listen for changes
  useEffect(() => {
    if (typeof window !== "undefined") {
      const savedDarkMode = localStorage.getItem("dark-mode") === "true";
      const savedAccentColor =
        localStorage.getItem("accent-color") || "#3b82f6";
      setIsDark(savedDarkMode);
      setAccentColor(savedAccentColor);

      // Watch for changes to data-mode attribute
      const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
          if (
            mutation.type === "attributes" &&
            mutation.attributeName === "data-mode"
          ) {
            const mode = document.documentElement.getAttribute("data-mode");
            setIsDark(mode === "dark");
          }
        });
      });

      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["data-mode"],
      });

      // Watch for accent color changes
      const accentObserver = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
          if (
            mutation.type === "attributes" &&
            mutation.attributeName === "style"
          ) {
            const accentColor =
              document.documentElement.style.getPropertyValue("--accent-color");
            if (accentColor) {
              setAccentColor(accentColor);
            }
          }
        });
      });

      accentObserver.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["style"],
      });

      return () => {
        observer.disconnect();
        accentObserver.disconnect();
      };
    }
  }, []);

  // Create theme with dynamic accent color
  const theme = createTheme({
    ...(isDark ? darkTheme : lightTheme),
    palette: {
      ...(isDark ? darkTheme.palette : lightTheme.palette),
      primary: {
        main: accentColor,
        light: adjustBrightness(accentColor, 20),
        dark: adjustBrightness(accentColor, -20),
        contrastText: isDark ? "#000" : "#fff",
      },
    },
  });

  // Helper function to adjust color brightness
  function adjustBrightness(color: string, percent: number): string {
    const num = parseInt(color.replace("#", ""), 16);
    const amt = Math.round(2.55 * percent);
    const R = (num >> 16) + amt;
    const G = ((num >> 8) & 0x00ff) + amt;
    const B = (num & 0x0000ff) + amt;
    return (
      "#" +
      (
        0x1000000 +
        (R < 255 ? (R < 1 ? 0 : R) : 255) * 0x10000 +
        (G < 255 ? (G < 1 ? 0 : G) : 255) * 0x100 +
        (B < 255 ? (B < 1 ? 0 : B) : 255)
      )
        .toString(16)
        .slice(1)
    );
  }

  return (
    <I18nProvider>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <BrowserRouter>
          <Routes>
            <Route
              path="/"
              element={
                <Layout
                  selectedDocument={selectedDocument}
                  onDocumentSelect={setSelectedDocument}
                />
              }
            >
              <Route
                path="reader/:id"
                element={<Box sx={{ p: 2 }}>PDF 阅读器（待实现）</Box>}
              />
              <Route
                path="settings"
                element={<Box sx={{ p: 2 }}>设置页面（待实现）</Box>}
              />
            </Route>
          </Routes>
        </BrowserRouter>
      </ThemeProvider>
    </I18nProvider>
  );
}

export default App;
