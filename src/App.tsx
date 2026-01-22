import { useState } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ThemeProvider, CssBaseline, createTheme } from "@mui/material/styles";
import { Box } from "@mui/material";
import { lightTheme, darkTheme } from "./theme";
import Layout from "./components/Layout";
import LibraryPage from "./pages/LibraryPage";

function App() {
  const [isDark, setIsDark] = useState(true);

  const theme = createTheme(isDark ? darkTheme : lightTheme);

  const toggleTheme = () => {
    setIsDark(!isDark);
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <BrowserRouter>
        <Routes>
          <Route
            path="/"
            element={<Layout isDark={isDark} onToggleTheme={toggleTheme} />}
          >
            <Route index element={<LibraryPage />} />
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
  );
}

export default App;
