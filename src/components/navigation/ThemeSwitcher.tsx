import { useState, useEffect } from "react";
import {
  IconButton,
  Menu,
  MenuItem,
  ListItemText,
  Box,
  CircularProgress,
} from "@mui/material";
import {
  Brightness4,
  Brightness7,
  Language,
  Palette,
  Check,
} from "@mui/icons-material";
import { useI18n, localeList, type LocaleCode } from "../../lib/i18n";

// Accent colors
const accentColors = [
  { name: "Blue", value: "#3b82f6", class: "bg-blue-500" },
  { name: "Purple", value: "#a855f7", class: "bg-purple-500" },
  { name: "Pink", value: "#ec4899", class: "bg-pink-500" },
  { name: "Red", value: "#ef4444", class: "bg-red-500" },
  { name: "Orange", value: "#f97316", class: "bg-orange-500" },
  { name: "Green", value: "#22c55e", class: "bg-green-500" },
  { name: "Teal", value: "#14b8a6", class: "bg-teal-500" },
  { name: "Cyan", value: "#06b6d4", class: "bg-cyan-500" },
];

export default function ThemeSwitcher() {
  const { locale, setLocale, t, isLoading: isI18nLoading } = useI18n();
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [selectedAccentColor, setSelectedAccentColor] = useState(
    accentColors[0].value,
  );

  // Menu states
  const [languageAnchor, setLanguageAnchor] = useState<null | HTMLElement>(
    null,
  );
  const [accentAnchor, setAccentAnchor] = useState<null | HTMLElement>(null);

  // Initialize theme from localStorage
  useEffect(() => {
    if (typeof window !== "undefined") {
      const savedDarkMode = localStorage.getItem("dark-mode") === "true";
      const savedAccentColor =
        localStorage.getItem("accent-color") || accentColors[0].value;

      setIsDarkMode(savedDarkMode);
      setSelectedAccentColor(savedAccentColor);

      applyDarkMode(savedDarkMode);
      applyAccentColor(savedAccentColor);
    }
  }, []);

  // Apply dark mode to document
  const applyDarkMode = (dark: boolean) => {
    if (typeof document !== "undefined") {
      const mode = dark ? "dark" : "light";
      document.documentElement.setAttribute("data-mode", mode);
    }
    if (typeof window !== "undefined") {
      localStorage.setItem("dark-mode", String(dark));
    }
  };

  // Apply accent color to document
  const applyAccentColor = (color: string) => {
    if (typeof document !== "undefined") {
      document.documentElement.style.setProperty("--accent-color", color);
      document.documentElement.style.setProperty(
        "--accent-color-hover",
        adjustBrightness(color, -10),
      );
    }
    if (typeof window !== "undefined") {
      localStorage.setItem("accent-color", color);
    }
  };

  // Helper function to adjust color brightness
  const adjustBrightness = (color: string, percent: number): string => {
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
  };

  // Toggle dark mode
  const toggleDarkMode = () => {
    const newMode = !isDarkMode;
    setIsDarkMode(newMode);
    applyDarkMode(newMode);
  };

  // Handle language menu
  const handleLanguageClick = (event: React.MouseEvent<HTMLElement>) => {
    setLanguageAnchor(event.currentTarget);
  };

  const handleLanguageClose = () => {
    setLanguageAnchor(null);
  };

  const handleLanguageSelect = async (localeCode: LocaleCode) => {
    await setLocale(localeCode);
    handleLanguageClose();
  };

  // Handle accent color menu
  const handleAccentClick = (event: React.MouseEvent<HTMLElement>) => {
    setAccentAnchor(event.currentTarget);
  };

  const handleAccentClose = () => {
    setAccentAnchor(null);
  };

  const handleAccentSelect = (color: string) => {
    setSelectedAccentColor(color);
    applyAccentColor(color);
    handleAccentClose();
  };

  return (
    <Box
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 0.5,
        height: "100%",
      }}
    >
      {/* Dark Mode Toggle */}
      <IconButton
        size="small"
        onClick={toggleDarkMode}
        title={isDarkMode ? t("theme.lightMode") : t("theme.darkMode")}
        sx={{
          color: "text.secondary",
          "&:hover": { color: "text.primary" },
        }}
      >
        {isDarkMode ? (
          <Brightness7 fontSize="small" />
        ) : (
          <Brightness4 fontSize="small" />
        )}
      </IconButton>

      {/* Language Selector */}
      <IconButton
        size="small"
        onClick={handleLanguageClick}
        title={t("language.selectLanguage")}
        sx={{
          color: "text.secondary",
          "&:hover": { color: "text.primary" },
        }}
        disabled={isI18nLoading}
      >
        {isI18nLoading ? (
          <CircularProgress size={16} />
        ) : (
          <Language fontSize="small" />
        )}
      </IconButton>
      <Menu
        anchorEl={languageAnchor}
        open={Boolean(languageAnchor)}
        onClose={handleLanguageClose}
        anchorOrigin={{
          vertical: "top",
          horizontal: "center",
        }}
        transformOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
        slotProps={{
          paper: {
            sx: {
              minWidth: 180,
              bgcolor: "background.paper",
            },
          },
        }}
      >
        {localeList.map((localeInfo) => (
          <MenuItem
            key={localeInfo.code}
            onClick={() => handleLanguageSelect(localeInfo.code)}
            selected={locale === localeInfo.code}
          >
            <ListItemText>
              <span style={{ marginRight: 8 }}>{localeInfo.flag}</span>
              {localeInfo.nativeName}
            </ListItemText>
            {locale === localeInfo.code && (
              <Check fontSize="small" sx={{ ml: 1 }} />
            )}
          </MenuItem>
        ))}
      </Menu>

      {/* Accent Color Selector */}
      <IconButton
        size="small"
        onClick={handleAccentClick}
        title={t("theme.accentColor")}
        sx={{
          color: "text.secondary",
          "&:hover": { color: "text.primary" },
        }}
      >
        <Palette fontSize="small" />
      </IconButton>
      <Menu
        anchorEl={accentAnchor}
        open={Boolean(accentAnchor)}
        onClose={handleAccentClose}
        anchorOrigin={{
          vertical: "top",
          horizontal: "center",
        }}
        transformOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
        slotProps={{
          paper: {
            sx: {
              minWidth: 200,
              bgcolor: "background.paper",
            },
          },
        }}
      >
        <Box sx={{ p: 2 }}>
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: "repeat(4, 1fr)",
              gap: 1,
            }}
          >
            {accentColors.map((color) => (
              <Box
                key={color.value}
                onClick={() => handleAccentSelect(color.value)}
                sx={{
                  width: 36,
                  height: 36,
                  borderRadius: 1,
                  bgcolor: color.value,
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  border:
                    selectedAccentColor === color.value
                      ? "2px solid"
                      : "2px solid transparent",
                  borderColor:
                    selectedAccentColor === color.value
                      ? "text.primary"
                      : "transparent",
                  "&:hover": {
                    opacity: 0.8,
                    transform: "scale(1.1)",
                  },
                  transition: "all 0.2s",
                }}
              >
                {selectedAccentColor === color.value && (
                  <Check fontSize="small" sx={{ color: "white" }} />
                )}
              </Box>
            ))}
          </Box>
        </Box>
      </Menu>
    </Box>
  );
}
