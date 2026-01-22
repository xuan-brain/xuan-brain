<script lang="ts">
  import { locale, isLoading, t, waitLocale } from "$lib/i18n";
  import {
    availableLocales,
    localeList,
    changeLocale,
    getCurrentLocaleInfo,
    formatDate,
  } from "$lib/i18n";
  import { Palette } from "lucide-svelte";

  // Available Skeleton color themes
  const colorThemes = [
    { name: "Cerberus", value: "cerberus", emoji: "🤖" },
    { name: "Catppuccin", value: "catppuccin", emoji: "🐈" },
    { name: "Concord", value: "concord", emoji: "🔴" },
    { name: "Crimson", value: "crimson", emoji: "🦊" },
    { name: "Fennec", value: "fennec", emoji: "👔" },
    { name: "HamlinDigo", value: "hamlindigo", emoji: "💀" },
    { name: "Legacy", value: "legacy", emoji: "🍃" },
    { name: "Mint", value: "mint", emoji: "🌸" },
    { name: "Modern", value: "modern", emoji: "🐙" },
    { name: "Mona", value: "mona", emoji: "🥙" },
    { name: "Nosh", value: "nosh", emoji: "🎑" },
    { name: "Nouveau", value: "nouveau", emoji: "🌲" },
    { name: "Pine", value: "pine", emoji: "📒" },
    { name: "Reign", value: "reign", emoji: "🚀" },
    { name: "Rocket", value: "rocket", emoji: "🌷" },
    { name: "Rose", value: "rose", emoji: "🏜️" },
    { name: "Sahara", value: "sahara", emoji: "🏜️" },
    { name: "Seafoam", value: "seafoam", emoji: "🌑" },
    { name: "Terminus", value: "terminus", emoji: "📺" },
    { name: "Vintage", value: "vintage", emoji: "👾" },
    { name: "Vox", value: "vox", emoji: "🌨️" },
    { name: "Wintry", value: "wintry", emoji: "❄️" },
  ];

  // Theme state using Svelte 5 runes
  let selectedColorTheme = $state("cerberus");
  let isDarkMode = $state(false);
  let isColorDropdownOpen = $state(false);
  let isLanguageDropdownOpen = $state(false);
  let isAccentColorDropdownOpen = $state(false);
  let isLocaleLoading = $state(false);

  // Predefined accent colors
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

  let selectedAccentColor = $state(accentColors[0].value);

  // Reactive: Current locale from i18n
  const currentLocaleCode = $derived($locale as "en" | "zh");
  const currentLocaleInfo = $derived(availableLocales[currentLocaleCode]);
  const isLocaleRTL = $derived($locale === "ar" || $locale === "he");
  const isI18nLoading = $derived($isLoading);

  // Apply color theme to document
  function applyColorTheme(theme: string) {
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", theme);
    }
    if (typeof window !== "undefined") {
      localStorage.setItem("color-theme", theme);
    }
  }

  // Apply dark mode to document
  function applyDarkMode(dark: boolean) {
    if (typeof document !== "undefined") {
      const mode = dark ? "dark" : "light";
      document.documentElement.setAttribute("data-mode", mode);
    }
    if (typeof window !== "undefined") {
      localStorage.setItem("dark-mode", String(dark));
    }
  }

  // Initialize themes from localStorage on mount
  function initializeThemes() {
    if (typeof window !== "undefined") {
      const savedColorTheme = localStorage.getItem("color-theme");
      const savedDarkMode = localStorage.getItem("dark-mode") === "true";
      const savedAccentColor = localStorage.getItem("accent-color");

      selectedColorTheme = savedColorTheme || "cerberus";
      isDarkMode = savedDarkMode;
      selectedAccentColor = savedAccentColor || accentColors[0].value;

      applyColorTheme(selectedColorTheme);
      applyDarkMode(isDarkMode);
      applyAccentColor(selectedAccentColor);
    }
  }

  // Apply accent color to document
  function applyAccentColor(color: string) {
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
  }

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

  // Initialize immediately on client side
  if (typeof window !== "undefined") {
    initializeThemes();
  }

  // Apply color theme whenever it changes
  $effect(() => {
    applyColorTheme(selectedColorTheme);
  });

  // Apply dark mode whenever it changes
  $effect(() => {
    applyDarkMode(isDarkMode);
  });

  // Apply accent color whenever it changes
  $effect(() => {
    applyAccentColor(selectedAccentColor);
  });

  // Toggle color theme dropdown
  function toggleColorDropdown() {
    isColorDropdownOpen = !isColorDropdownOpen;
    isLanguageDropdownOpen = false;
  }

  // Toggle language dropdown
  function toggleLanguageDropdown() {
    isLanguageDropdownOpen = !isLanguageDropdownOpen;
    isColorDropdownOpen = false;
    isAccentColorDropdownOpen = false;
  }

  // Toggle accent color dropdown
  function toggleAccentColorDropdown() {
    isAccentColorDropdownOpen = !isAccentColorDropdownOpen;
    isColorDropdownOpen = false;
    isLanguageDropdownOpen = false;
  }

  // Select a color theme
  function selectColorTheme(theme: string) {
    selectedColorTheme = theme;
    isColorDropdownOpen = false;
  }

  // Select an accent color
  function selectAccentColor(color: string) {
    selectedAccentColor = color;
    isAccentColorDropdownOpen = false;
  }

  // Toggle dark mode
  function toggleDarkMode() {
    isDarkMode = !isDarkMode;
  }

  // Select a language
  async function selectLanguage(localeCode: string) {
    if (localeCode === currentLocaleCode) {
      isLanguageDropdownOpen = false;
      return;
    }

    isLocaleLoading = true;
    try {
      await changeLocale(localeCode as "en" | "zh");
    } catch (error) {
      console.error("Failed to change language:", error);
      // Re-open dropdown to allow retry
    } finally {
      isLocaleLoading = false;
      isLanguageDropdownOpen = false;
    }
  }

  // Close dropdowns when clicking outside
  function handleOutsideClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".theme-switcher")) {
      isColorDropdownOpen = false;
      isLanguageDropdownOpen = false;
      isAccentColorDropdownOpen = false;
    }
  }

  // Handle keyboard navigation for dropdowns
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      isColorDropdownOpen = false;
      isLanguageDropdownOpen = false;
      isAccentColorDropdownOpen = false;
    }
  }
</script>

<!-- Prevent flash by setting themes immediately before hydration -->
<svelte:head>
  <script>
    (function () {
      // Initialize themes
      const colorTheme = localStorage.getItem("color-theme") || "cerberus";
      const darkMode = localStorage.getItem("dark-mode") === "true";
      const accentColor = localStorage.getItem("accent-color") || "#3b82f6";
      document.documentElement.setAttribute("data-theme", colorTheme);
      document.documentElement.setAttribute(
        "data-mode",
        darkMode ? "dark" : "light",
      );
      document.documentElement.style.setProperty("--accent-color", accentColor);

      // Initialize language (svelte-i18n will pick this up)
      const savedLanguage = localStorage.getItem("xuan-brain-locale");
      if (savedLanguage) {
        window.__svelte_i18n_language = savedLanguage;
      }
    })();
  </script>
</svelte:head>

<svelte:window on:click={handleOutsideClick} on:keydown={handleKeydown} />

<!-- Compact horizontal layout for desktop app style -->
<div
  class="theme-switcher flex flex-row items-center gap-2"
  dir={isLocaleRTL ? "rtl" : "ltr"}
>
  <!-- Dark Mode Toggle - Circular Button -->
  <button
    class="w-8 h-8 rounded-full flex items-center justify-center transition-all duration-200 hover:scale-105 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 shrink-0"
    onclick={toggleDarkMode}
    aria-label={isDarkMode ? $t("theme.lightMode") : $t("theme.darkMode")}
    aria-pressed={isDarkMode}
    type="button"
    disabled={isI18nLoading}
    title={isDarkMode ? $t("theme.lightMode") : $t("theme.darkMode")}
  >
    <span class="text-base">{isDarkMode ? "🌙" : "☀️"}</span>
  </button>

  <!-- Language Selector - Compact Button -->
  <div class="relative">
    <button
      class="flex items-center gap-1 text-sm font-medium rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed h-8"
      style="padding: 5px 8px;"
      onclick={toggleLanguageDropdown}
      aria-expanded={isLanguageDropdownOpen}
      aria-haspopup="listbox"
      disabled={isI18nLoading}
      title={$t("language.selectLanguage")}
    >
      <span class="text-base">{currentLocaleInfo?.flag || "🌐"}</span>
      {#if isI18nLoading}
        <svg
          class="animate-spin h-3 w-3"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12c0 2.347.274 2.018 2.414 4.014.602l2.314-2.314a1 1 0 011.414 1.414l-2.314 2.314A7.994 7.994 0 0122 12z"
          ></path>
        </svg>
      {/if}
    </button>

    <!-- Language Dropdown Menu -->
    {#if isLanguageDropdownOpen}
      <div
        class="absolute bottom-full left-0 mb-2 w-36 max-h-60 overflow-y-auto rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50"
        role="listbox"
      >
        <div class="p-1 space-y-0.5">
          {#each localeList as localeInfo}
            <button
              class="w-full flex items-center gap-1.5 text-sm rounded transition-colors {currentLocaleCode ===
              localeInfo.code
                ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
              style="padding: 4px 6px;"
              onclick={() => selectLanguage(localeInfo.code)}
              role="option"
              aria-selected={currentLocaleCode === localeInfo.code}
              disabled={isI18nLoading}
            >
              <span class="text-base">{localeInfo.flag}</span>
              <span class="flex-1">{localeInfo.nativeName}</span>
              {#if currentLocaleCode === localeInfo.code}
                <svg
                  class="ml-auto h-3 w-3"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-1.5a.75.75 0 010-1.438l8.5-8.5a.75.75 0 011.06-.009z"
                    clip-rule="evenodd"
                  />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Color Theme Selector - Compact Button -->
  <div class="relative">
    <button
      class="flex items-center gap-1 text-sm font-medium rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed h-8"
      style="padding: 5px 8px;"
      onclick={toggleColorDropdown}
      aria-expanded={isColorDropdownOpen}
      aria-haspopup="listbox"
      disabled={isI18nLoading}
      title={$t("theme.selectTheme")}
    >
      <span class="text-base">
        {colorThemes.find((t) => t.value === selectedColorTheme)?.emoji || "🎨"}
      </span>
    </button>

    <!-- Color Theme Dropdown Menu -->
    {#if isColorDropdownOpen}
      <div
        class="absolute bottom-full left-0 mb-2 w-40 max-h-80 overflow-y-auto rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50"
        role="listbox"
      >
        <div class="p-1 space-y-0.5">
          {#each colorThemes as theme}
            <button
              class="w-full flex items-center gap-1.5 text-sm rounded transition-colors {selectedColorTheme ===
              theme.value
                ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
              style="padding: 4px 6px;"
              onclick={() => selectColorTheme(theme.value)}
              role="option"
              aria-selected={selectedColorTheme === theme.value}
              disabled={isI18nLoading}
            >
              <span class="text-base">{theme.emoji}</span>
              <span>{theme.name}</span>
              {#if selectedColorTheme === theme.value}
                <svg
                  class="ml-auto h-3 w-3"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-1.5a.75.75 0 010-1.438l8.5-8.5a.75.75 0 011.06-.009z"
                    clip-rule="evenodd"
                  />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Accent Color Selector - Compact Button -->
  <div class="relative">
    <button
      class="flex items-center gap-1 text-sm font-medium rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed h-8"
      style="padding: 5px 8px;"
      onclick={toggleAccentColorDropdown}
      aria-expanded={isAccentColorDropdownOpen}
      aria-haspopup="listbox"
      disabled={isI18nLoading}
      title="主色调"
    >
      <span class="text-base">🎨</span>
    </button>

    <!-- Accent Color Dropdown Menu -->
    {#if isAccentColorDropdownOpen}
      <div
        class="absolute bottom-full left-0 mb-2 w-32 rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50"
        role="listbox"
      >
        <div class="p-2 grid grid-cols-4 gap-1.5">
          {#each accentColors as color}
            <button
              class="w-6 h-6 rounded-full border-2 transition-transform hover:scale-110 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400 shrink-0 {selectedAccentColor ===
              color.value
                ? 'border-gray-900 dark:border-gray-100 ring-2 ring-offset-1'
                : 'border-gray-300 dark:border-gray-600'}"
              style="background-color: {color.value};"
              onclick={() => selectAccentColor(color.value)}
              role="option"
              aria-selected={selectedAccentColor === color.value}
              aria-label={color.name}
              disabled={isI18nLoading}
              title={color.name}
            >
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Ensure button has no default appearance */
  button {
    appearance: none;
    border: none;
    background: none;
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }

  /* Add subtle hover effect */
  .theme-switcher button:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .theme-switcher button:active:not(:disabled) {
    transform: scale(0.98);
  }

  /* Scrollbar styling for dropdowns */
  .theme-switcher div[role="listbox"]::-webkit-scrollbar {
    width: 6px;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-track {
    background: transparent;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Dark mode scrollbar */
  @media (prefers-color-scheme: dark) {
    .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb {
      background: #4b5563;
    }

    .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb:hover {
      background: #6b7280;
    }
  }

  /* Loading spinner animation */
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }
</style>
