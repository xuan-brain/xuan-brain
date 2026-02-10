import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import zh from "./locales/zh.json";

export type LocaleCode = "en" | "zh";

export const availableLocales: Record<LocaleCode, { name: string; nativeName: string; flag: string }> = {
  en: { name: "English", nativeName: "English", flag: "🇺🇸" },
  zh: { name: "Chinese", nativeName: "中文", flag: "🇨🇳" },
};

const STORAGE_KEY = "xuan-brain-locale";

function getInitialLocale(): LocaleCode {
  if (typeof window === "undefined") return "en";
  const saved = localStorage.getItem(STORAGE_KEY) as LocaleCode | null;
  if (saved && availableLocales[saved]) return saved;
  const browserLang = navigator.language.split("-")[0] as LocaleCode;
  if (browserLang && availableLocales[browserLang]) return browserLang;
  return "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: "en",
  messages: { en, zh },
});

export function useI18n() {
  return i18n.global;
}

export function setLocale(locale: LocaleCode) {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
}
