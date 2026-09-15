"use client";

import { useEffect, useState } from "react";

/** Brand v2 token strip — matches brand/v2/preview.html §06. Collapsible. */
const TOKENS = [
  { label: "Cyan solid", value: "#00e5ff" },
  { label: "Cyan gradient", value: "#bff8ff → #00e5ff → #0b7a96" },
  { label: "Ink solid", value: "#0e0e14" },
  { label: "Slate solid", value: "#64748b" },
  { label: "Icon highlight", value: "#9ff7ff" },
  { label: "Icon core", value: "#bff8ff" },
] as const;

const STORAGE_KEY = "owyx.brandPaletteOpen";

export default function BrandPalette() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    try {
      setOpen(localStorage.getItem(STORAGE_KEY) === "1");
    } catch {
      /* ignore */
    }
  }, []);

  function toggle() {
    setOpen((v) => {
      const next = !v;
      try {
        localStorage.setItem(STORAGE_KEY, next ? "1" : "0");
      } catch {
        /* ignore */
      }
      return next;
    });
  }

  return (
    <div className="fixed bottom-4 right-4 z-40 hidden lg:flex flex-col items-end gap-2">
      {open && (
        <aside
          className="max-w-[min(22rem,42vw)] flex flex-col gap-2 rounded-xl border border-line/80 bg-panel/90 backdrop-blur-sm p-3 shadow-[0_12px_40px_-20px_rgba(0,229,255,0.35)]"
          aria-label="Токены бренда v2"
        >
          <div className="flex items-center justify-between gap-3">
            <p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted m-0">
              Токены v2
            </p>
            <button
              type="button"
              onClick={toggle}
              className="font-mono text-[10px] text-muted hover:text-accent cursor-pointer border-0 bg-transparent p-0"
            >
              Скрыть
            </button>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {TOKENS.map((t) => (
              <span
                key={t.label}
                className="font-mono text-[10px] leading-snug px-2 py-1 rounded-md border border-line bg-panel-2 text-muted"
              >
                <strong className="text-text font-medium">{t.label}</strong>
                <span className="text-muted"> · </span>
                {t.value}
              </span>
            ))}
          </div>
        </aside>
      )}
      {!open && (
        <button
          type="button"
          onClick={toggle}
          className="font-mono text-[10px] uppercase tracking-[0.1em] rounded-lg border border-line bg-panel/80 backdrop-blur-sm px-3 py-2 text-muted hover:text-accent hover:border-accent cursor-pointer transition-colors"
          title="Показать палитру бренда"
        >
          Палитра
        </button>
      )}
    </div>
  );
}
