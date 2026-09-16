"use client";

import { useLocale } from "@/hooks/useLocale";
import { localeShortLabel, type Locale } from "@/lib/i18n";

export default function LanguageToggle() {
  const { locale, setLocale, dict } = useLocale();
  const next: Locale = locale === "ru_RU" ? "en_US" : "ru_RU";

  return (
    <button
      type="button"
      className="inline-flex items-center justify-center gap-1 rounded-[10px] border border-line bg-panel h-11 min-w-11 px-2 text-muted hover:text-accent hover:border-accent transition-colors cursor-pointer"
      aria-label={dict.header.languageAria}
      title={dict.header.languageTitle}
      onClick={() => setLocale(next)}
    >
      <svg className="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
        <circle cx="12" cy="12" r="9" strokeWidth="1.75" />
        <path
          strokeWidth="1.75"
          d="M3 12h18M12 3c2.5 2.8 3.8 5.8 3.8 9s-1.3 6.2-3.8 9c-2.5-2.8-3.8-5.8-3.8-9s1.3-6.2 3.8-9z"
        />
      </svg>
      <span className="text-[10px] font-bold tracking-wide">{localeShortLabel(locale)}</span>
    </button>
  );
}
