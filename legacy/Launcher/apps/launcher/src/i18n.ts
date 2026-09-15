import en from "./i18n/en.json";
import ru from "./i18n/ru.json";

export type Locale = "en" | "ru";

const catalogs: Record<Locale, Record<string, string>> = {
  en: en as Record<string, string>,
  ru: ru as Record<string, string>,
};

let locale: Locale = "ru";

export function normalizeLocale(value?: string | null): Locale {
  return value === "en" ? "en" : "ru";
}

export function getLocale(): Locale {
  return locale;
}

export function setLocale(next: Locale) {
  locale = next;
  document.documentElement.lang = next;
}

export function t(key: string, vars?: Record<string, string | number>): string {
  const table = catalogs[locale] ?? catalogs.ru;
  let text = table[key] ?? catalogs.en[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replaceAll(`{${k}}`, String(v));
    }
  }
  return text;
}
