import { useEffect } from "react";
import { Button, Dropdown, Spin, Space } from "antd";
import {
  MoonOutlined,
  SunOutlined,
  GlobalOutlined,
  BgColorsOutlined,
  CheckOutlined,
} from "@ant-design/icons";
import type { MenuProps } from "antd";
import { useI18n, localeList } from "../../lib/i18n";
import { useAppStore } from "../../stores/useAppStore";

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
  const { isDark: isDarkMode, accentColor, setTheme, setAccentColor } = useAppStore();

  // Initialize theme from store on mount
  useEffect(() => {
    if (typeof window !== "undefined") {
      // Apply dark mode to document
      const mode = isDarkMode ? "dark" : "light";
      document.documentElement.setAttribute("data-mode", mode);

      // Apply accent color to document
      document.documentElement.style.setProperty("--accent-color", accentColor);
      document.documentElement.style.setProperty(
        "--accent-color-hover",
        adjustBrightness(accentColor, -10),
      );
    }
  }, [isDarkMode, accentColor]);

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
    setTheme(newMode);
  };

  // Language menu items
  const languageMenuItems: MenuProps["items"] = localeList.map(
    (localeInfo) => ({
      key: localeInfo.code,
      label: (
        <span>
          <span style={{ marginRight: 8 }}>{localeInfo.flag}</span>
          {localeInfo.nativeName}
        </span>
      ),
      onClick: () => setLocale(localeInfo.code),
      icon: locale === localeInfo.code ? <CheckOutlined /> : null,
    }),
  );

  // Accent color menu items
  const accentMenuItems: MenuProps["items"] = [
    {
      key: "colors",
      label: (
        <div>
          <div style={{ marginBottom: 8 }}>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(4, 1fr)",
                gap: 4,
              }}
            >
              {accentColors.map((color) => (
                <div
                  key={color.value}
                  onClick={() => {
                    setAccentColor(color.value);
                  }}
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 4,
                    backgroundColor: color.value,
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    border:
                      accentColor === color.value
                        ? "2px solid currentColor"
                        : "2px solid transparent",
                    transition: "all 0.2s",
                  }}
                >
                  {accentColor === color.value && (
                    <CheckOutlined style={{ color: "white" }} />
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      ),
    },
  ];

  return (
    <Space size="small">
      {/* Dark Mode Toggle */}
      <Button
        type="text"
        size="small"
        icon={isDarkMode ? <SunOutlined /> : <MoonOutlined />}
        onClick={toggleDarkMode}
        title={isDarkMode ? t("theme.lightMode") : t("theme.darkMode")}
        style={{ height: 24, padding: "0 4px" }}
      />

      {/* Language Selector */}
      <Dropdown menu={{ items: languageMenuItems }} trigger={["click"]}>
        <Button
          type="text"
          size="small"
          icon={
            isI18nLoading ? (
              <Spin size="small" />
            ) : (
              <GlobalOutlined />
            )
          }
          title={t("language.selectLanguage")}
          disabled={isI18nLoading}
          style={{ height: 24, padding: "0 4px" }}
        />
      </Dropdown>

      {/* Accent Color Selector */}
      <Dropdown menu={{ items: accentMenuItems }} trigger={["click"]}>
        <Button
          type="text"
          size="small"
          icon={<BgColorsOutlined />}
          title={t("theme.accentColor")}
          style={{ height: 24, padding: "0 4px" }}
        />
      </Dropdown>
    </Space>
  );
}
