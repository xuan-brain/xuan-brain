import { useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { ConfigProvider } from "antd";
import { createDynamicTheme } from "./theme";
import { I18nProvider } from "./lib/i18n";
import MainLayout from "./components/layout/Layout";
import { useAppStore } from "./stores/useAppStore";

// Pages
import PapersPage from "./pages/PapersPage";
import ClipsPage from "./pages/ClipsPage";
import WritingPage from "./pages/WritingPage";
import SubscriptionPage from "./pages/SubscriptionPage";
import SettingsPage from "./pages/SettingsPage";

// 引入 Ant Design 默认样式
import "antd/dist/reset.css";

function App() {
  const { isDark, accentColor, setTheme, setAccentColor } = useAppStore();

  // Initialize theme from system/css variables if needed, though zustand persist handles localStorage
  useEffect(() => {
    // Test logging to verify tauri-plugin-tracing
    console.log("App component mounted - Tracing system check");
    console.info("Info level log check");

    if (typeof window !== "undefined") {
      // Watch for changes to data-mode attribute
      const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
          if (
            mutation.type === "attributes" &&
            mutation.attributeName === "data-mode"
          ) {
            const mode = document.documentElement.getAttribute("data-mode");
            setTheme(mode === "dark");
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
            const newAccentColor =
              document.documentElement.style.getPropertyValue("--accent-color");
            if (newAccentColor) {
              setAccentColor(newAccentColor);
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
  }, [setTheme, setAccentColor]);

  // Create theme with dynamic accent color
  const antdTheme = createDynamicTheme(accentColor, isDark);

  return (
    <I18nProvider>
      <ConfigProvider theme={antdTheme}>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<MainLayout />}>
              <Route index element={<Navigate to="/papers" replace />} />
              <Route path="papers" element={<PapersPage />} />
              <Route path="clips" element={<ClipsPage />} />
              <Route path="writing" element={<WritingPage />} />
              <Route path="subscriptions" element={<SubscriptionPage />} />
              <Route path="settings" element={<SettingsPage />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </ConfigProvider>
    </I18nProvider>
  );
}

export default App;
