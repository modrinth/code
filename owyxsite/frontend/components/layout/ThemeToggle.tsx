"use client";

import { useLocale } from "@/hooks/useLocale";
import { useSiteAccent, type SiteAccent } from "@/lib/siteTheme";

export default function ThemeToggle() {
  const { dict } = useLocale();
  const { accent, cycle } = useSiteAccent();

  const labels: Record<SiteAccent, string> = {
    cyan: dict.header.themeCyan,
    synthwave: dict.header.themeSynthwave,
    onyx: dict.header.themeOnyx,
    magma: dict.header.themeMagma,
  };

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
      <span className="text-[10px] font-bold tracking-wide">{labels[accent]}</span>
    </button>
  );
}
