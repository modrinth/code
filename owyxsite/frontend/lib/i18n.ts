import enUS from "@/locales/en_US.json";
import ruRU from "@/locales/ru_RU.json";

export const LOCALES = ["ru_RU", "en_US"] as const;
export type Locale = (typeof LOCALES)[number];
export const DEFAULT_LOCALE: Locale = "ru_RU";
export const LOCALE_STORAGE_KEY = "owyx.locale";

export type Dictionary = typeof ruRU;

const DICTS: Record<Locale, Dictionary> = {
  ru_RU: ruRU as Dictionary,
  en_US: enUS as Dictionary,
};

export function isLocale(value: string | null | undefined): value is Locale {
  return value === "ru_RU" || value === "en_US";
}

export function getDictionary(locale: Locale): Dictionary {
  return DICTS[locale] ?? DICTS[DEFAULT_LOCALE];
}

export function localeToHtmlLang(locale: Locale): string {
  return locale === "en_US" ? "en" : "ru";
}

export function localeShortLabel(locale: Locale): string {
  return locale === "en_US" ? "EN" : "RU";
}
