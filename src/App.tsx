import { useState, useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import { CssBaseline, Box } from "@mui/material";
import { lightTheme, darkTheme } from "./theme";
import { I18nProvider } from "./lib/i18n";
import Layout from "./components/Layout";

function App() {
  const [isDark, setIsDark] = useState(true);
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
      setIsDark(savedDarkMode);

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

      return () => observer.disconnect();
    }
  }, []);

  const theme = createTheme(isDark ? darkTheme : lightTheme);

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
