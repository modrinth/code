"use client";

import { useEffect, useId, useRef, useState } from "react";
import { useLocale } from "@/hooks/useLocale";
import { useSiteAccent, type SiteAccent } from "@/lib/siteTheme";

export default function ThemeToggle() {
  const { dict } = useLocale();
  const { accent, setAccent, accents } = useSiteAccent();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const listId = useId();

  const labels: Record<SiteAccent, string> = {
    cyan: dict.header.themeCyan,
    synthwave: dict.header.themeSynthwave,
    onyx: dict.header.themeOnyx,
    magma: dict.header.themeMagma,
  };

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={rootRef} className="relative">
      <button
        type="button"
        className="inline-flex items-center justify-center gap-1.5 rounded-[10px] border border-line bg-panel/80 h-11 min-w-11 px-2.5 text-muted hover:text-accent hover:border-accent transition-colors cursor-pointer backdrop-blur-md"
        aria-label={dict.header.themeAria}
        title={dict.header.themeTitle}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={listId}
        onClick={() => setOpen((v) => !v)}
      >
        <span
          className="h-2.5 w-2.5 rounded-full shrink-0"
          style={{ background: "var(--accent)" }}
          aria-hidden="true"
        />
        <span className="text-[10px] font-bold tracking-wide">{labels[accent]}</span>
        <span className="text-[9px] opacity-70" aria-hidden="true">
          ▾
        </span>
      </button>

      {open && (
        <ul
          id={listId}
          role="listbox"
          aria-label={dict.header.themeAria}
          className="absolute right-0 top-[calc(100%+6px)] z-[60] min-w-[10.5rem] overflow-hidden rounded-xl border border-line bg-panel/95 p-1 shadow-[0_16px_40px_-18px_rgba(0,0,0,0.75)] backdrop-blur-xl"
        >
          {accents.map((option) => {
            const selected = option === accent;
            return (
              <li key={option} role="option" aria-selected={selected}>
                <button
                  type="button"
                  className={`flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-left text-xs font-semibold transition-colors cursor-pointer ${
                    selected
                      ? "bg-accent/15 text-accent"
                      : "text-muted hover:bg-panel-2 hover:text-text"
                  }`}
                  onClick={() => {
                    setAccent(option);
                    setOpen(false);
                  }}
                >
                  <span
                    className="h-2.5 w-2.5 rounded-full shrink-0"
                    style={{
                      background:
                        option === "cyan"
                          ? "#00e5ff"
                          : option === "synthwave"
                            ? "#ec4899"
                            : option === "onyx"
                              ? "#f5f5f5"
                              : "#f87171",
                    }}
                    aria-hidden="true"
                  />
                  {labels[option]}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
