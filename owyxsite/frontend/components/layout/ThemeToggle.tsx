"use client";

import { useLocale } from "@/hooks/useLocale";
import { useSiteAccent, type SiteAccent } from "@/lib/siteTheme";

const LABELS: Record<SiteAccent, string> = {
  cyan: "Cyan",
  synthwave: "Wave",
  onyx: "Onyx",
  magma: "Magma",
};

export default function ThemeToggle() {
  const { dict } = useLocale();
  const { accent, cycle } = useSiteAccent();

  return (
    <button
      type="button"
      className="inline-flex items-center justify-center gap-1 rounded-[10px] border border-line bg-panel/80 h-11 min-w-11 px-2 text-muted hover:text-accent hover:border-accent transition-colors cursor-pointer backdrop-blur-md"
      aria-label={dict.header.themeAria}
      title={dict.header.themeTitle}
      onClick={cycle}
    >
      <span
        className="h-2.5 w-2.5 rounded-full shrink-0"
        style={{ background: "var(--accent)" }}
        aria-hidden="true"
      />
      <span className="text-[10px] font-bold tracking-wide">{LABELS[accent]}</span>
    </button>
  );
}
