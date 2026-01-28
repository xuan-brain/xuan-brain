import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
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

// Translations map - use any to avoid type compatibility issues between locales
const translations: Record<LocaleCode, any> = {
  en,
  zh,
};

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

// I18n Context
interface I18nContextType {
  locale: LocaleCode;
  setLocale: (locale: LocaleCode) => void;
  t: (key: string) => string;
  isLoading: boolean;
  currentLocaleInfo: LocaleInfo;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

// I18n Provider Props
interface I18nProviderProps {
  children: ReactNode;
}

// I18n Provider Component
export function I18nProvider({ children }: I18nProviderProps) {
  const [locale, setLocaleState] = useState<LocaleCode>(getInitialLocale());
  const [isLoading, setIsLoading] = useState(false);

  // Update document direction and language
  useEffect(() => {
    const localeInfo = availableLocales[locale];
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("dir", localeInfo.direction);
      document.documentElement.setAttribute("lang", locale);
    }
  }, [locale]);

  // Translation function
  const t = (key: string): string => {
    const keys = key.split(".");
    let value: any = translations[locale];

    for (const k of keys) {
      if (value && typeof value === "object" && k in value) {
        value = value[k];
      } else {
        // Fallback to English if key not found
        let fallbackValue: any = translations[FALLBACK_LOCALE];
        for (const fk of keys) {
          if (
            fallbackValue &&
            typeof fallbackValue === "object" &&
            fk in fallbackValue
          ) {
            fallbackValue = fallbackValue[fk];
          } else {
            return key; // Return key if not found in fallback either
          }
        }
        return fallbackValue;
      }
    }

    return typeof value === "string" ? value : key;
  };

  // Change locale with persistence
  const setLocale = async (newLocale: LocaleCode) => {
    if (!availableLocales[newLocale]) {
      console.error(`Locale "${newLocale}" is not available`);
      return;
    }

    setIsLoading(true);
    try {
      setLocaleState(newLocale);

      // Persist to localStorage
      if (typeof window !== "undefined") {
        localStorage.setItem(STORAGE_KEY, newLocale);
      }
    } catch (error) {
      console.error("Failed to change locale:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const value: I18nContextType = {
    locale,
    setLocale,
    t,
    isLoading,
    currentLocaleInfo: availableLocales[locale],
  };

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

// Custom hook to use i18n
export function useI18n() {
  const context = useContext(I18nContext);
  if (context === undefined) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return context;
}

// Helper: Get formatted date/time
export function formatDate(
  date: Date,
  locale: LocaleCode,
  options?: Intl.DateTimeFormatOptions,
): string {
  return new Intl.DateTimeFormat(locale, options).format(date);
}

// Helper: Get formatted number
export function formatNumber(
  number: number,
  locale: LocaleCode,
  options?: Intl.NumberFormatOptions,
): string {
  return new Intl.NumberFormat(locale, options).format(number);
}
