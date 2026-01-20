import { init, register, waitLocale, locale, isLoading, _ } from "svelte-i18n";
import { derived } from "svelte/store";
import en from "./en";
import zh from "./zh";

// Type definitions
export type TranslationKey = keyof typeof en;
export type LocaleCode = "en" | "zh";

// Locale metadata interface
interface LocaleInfo {
  code: LocaleCode;
  name: string;
  nativeName: string;
  flag: string;
  direction: "ltr" | "rtl";
}

// Available locales with metadata
export const availableLocales: Record<LocaleCode, LocaleInfo> = {
  en: {
    code: "en",
    name: "English",
    nativeName: "English",
    flag: "🇺🇸",
    direction: "ltr",
  },
  zh: {
    code: "zh",
    name: "Chinese",
    nativeName: "中文",
    flag: "🇨🇳",
    direction: "ltr",
  },
} as const;

// Export locale info array for UI
export const localeList = Object.values(availableLocales);

// Default configuration
const STORAGE_KEY = "xuan-brain-locale";
const FALLBACK_LOCALE: LocaleCode = "en";

// Determine initial locale
function getInitialLocale(): LocaleCode {
  if (typeof window === "undefined") return FALLBACK_LOCALE;

  // 1. Check localStorage first
  const savedLocale = localStorage.getItem(STORAGE_KEY) as LocaleCode | null;
  if (savedLocale && availableLocales[savedLocale]) {
    return savedLocale;
  }

  // 2. Check browser language
  const browserLang = navigator.language.split("-")[0] as LocaleCode;
  if (browserLang && availableLocales[browserLang]) {
    return browserLang;
  }

  // 3. Fall back to default
  return FALLBACK_LOCALE;
}

// Initialize svelte-i18n
init({
  fallbackLocale: FALLBACK_LOCALE,
  initialLocale: getInitialLocale(),
});

// Register locale loaders after init
register("en", () => Promise.resolve(en));
register("zh", () => Promise.resolve(zh));

// Export svelte-i18n stores and functions
export { locale, isLoading, waitLocale, _, _ as t } from "svelte-i18n";

// Helper function to set locale
export function setLocale(newLocale: LocaleCode): void {
  locale.set(newLocale);
}

// Helper: Get locale info by code
export function getLocaleInfo(code: LocaleCode): LocaleInfo | undefined {
  return availableLocales[code];
}

// Helper: Get current locale info
export function getCurrentLocaleInfo(): LocaleInfo {
  const currentCode = $locale as LocaleCode;
  return availableLocales[currentCode];
}

// Helper: Check if locale is RTL
export function isRTL(code?: LocaleCode): boolean {
  const localeCode = code || ($locale as LocaleCode);
  return availableLocales[localeCode]?.direction === "rtl";
}

// Helper: Change locale with persistence and loading state
export async function changeLocale(newLocale: LocaleCode): Promise<void> {
  if (!availableLocales[newLocale]) {
    console.error(`Locale "${newLocale}" is not available`);
    return;
  }

  try {
    // Set the locale using svelte-i18n's store
    locale.set(newLocale);
    await waitLocale(newLocale);

    // Persist to localStorage
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, newLocale);
    }

    // Update document direction
    updateDocumentDirection(newLocale);
  } catch (error) {
    console.error("Failed to change locale:", error);
    throw error;
  }
}

// Helper: Update HTML document direction for RTL support
function updateDocumentDirection(localeCode: LocaleCode): void {
  if (typeof document !== "undefined") {
    const localeInfo = availableLocales[localeCode];
    if (localeInfo) {
      document.documentElement.setAttribute("dir", localeInfo.direction);
      document.documentElement.setAttribute("lang", localeCode);
    }
  }
}

// Helper: Get formatted date/time
export function formatDate(
  date: Date,
  options?: Intl.DateTimeFormatOptions,
): string {
  const currentLocale = $locale;
  return new Intl.DateTimeFormat(currentLocale, options).format(date);
}

// Helper: Get formatted number
export function formatNumber(
  number: number,
  options?: Intl.NumberFormatOptions,
): string {
  const currentLocale = $locale;
  return new Intl.NumberFormat(currentLocale, options).format(number);
}

// Helper: Translate with fallback
export function safeTranslate(key: TranslationKey, fallback?: string): string {
  try {
    const translation = t(key);
    // If translation equals key, it means it's missing
    if (translation === key && fallback) {
      return fallback;
    }
    return translation;
  } catch {
    return fallback || key;
  }
}

// Helper: Batch translate multiple keys
export function translateBatch<K extends TranslationKey>(
  keys: K[],
): Record<K, string> {
  return keys.reduce(
    (acc, key) => {
      acc[key] = t(key);
      return acc;
    },
    {} as Record<K, string>,
  );
}

// Reactive derived store for current locale info
export const currentLocaleInfo = derived(locale, ($locale) => {
  const localeCode = $locale as LocaleCode;
  return availableLocales[localeCode];
});

// Reactive derived store for RTL status
export const isCurrentRTL = derived(locale, ($locale) => {
  const localeCode = $locale as LocaleCode;
  return availableLocales[localeCode]?.direction === "rtl";
});

// Export type for components
export type I18nStores = {
  locale: typeof locale;
  isLoading: typeof isLoading;
};

// Export type for translation function
export type TranslateFunction = typeof t;
