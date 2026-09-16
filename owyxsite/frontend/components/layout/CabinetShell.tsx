"use client";

import type { ReactNode } from "react";
import { useLocale } from "@/hooks/useLocale";

export type CabinetNavItem = {
  id: string;
  label: string;
  hint?: string;
};

export default function CabinetShell({
  eyebrow,
  title,
  subtitle,
  actions,
  nav,
  activeId,
  onNav,
  children,
  footerNote,
}: {
  eyebrow: string;
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  nav: CabinetNavItem[];
  activeId: string;
  onNav: (id: string) => void;
  children: ReactNode;
  footerNote?: ReactNode;
}) {
  const { dict } = useLocale();

  return (
    <main id="main-content" className="relative min-h-[calc(100vh-64px)]">
      <div className="relative z-10 mx-auto max-w-6xl px-4 py-8 sm:px-6 sm:py-10">
        <header className="mb-7 flex flex-wrap items-end justify-between gap-4 fade-up sm:mb-8">
          <div className="min-w-0">
            <p className="text-xs uppercase tracking-[0.16em] text-accent/80">{eyebrow}</p>
            <h1 className="font-display mt-1.5 text-3xl font-bold tracking-tight text-text sm:text-4xl overflow-wrap-anywhere min-w-0">
              {title}
            </h1>
            {subtitle && <p className="mt-2 max-w-xl text-sm leading-relaxed text-muted">{subtitle}</p>}
          </div>
          {actions && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
        </header>

        <div className="grid gap-5 lg:grid-cols-[13.5rem_minmax(0,1fr)] lg:gap-6 fade-up-2">
          <nav
            aria-label={dict.profile.sectionsAria}
            className="flex gap-2 overflow-x-auto pb-1 lg:flex-col lg:gap-1.5 lg:overflow-visible lg:pb-0 lg:sticky lg:top-24 lg:self-start"
          >
            {nav.map((item) => {
              const active = item.id === activeId;
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onNav(item.id)}
                  aria-current={active ? "page" : undefined}
                  className={`cabinet-rail-item ${active ? "is-active" : ""}`}
                >
                  <span className="truncate">{item.label}</span>
                  {item.hint && (
                    <span className={`hidden text-[11px] lg:block ${active ? "text-accent/70" : "text-muted"}`}>
                      {item.hint}
                    </span>
                  )}
                </button>
              );
            })}
          </nav>

          <div className="panel min-w-0 bg-panel/90 backdrop-blur-sm">
            <div className="p-5 sm:p-7">{children}</div>
          </div>
        </div>

        {footerNote && <p className="mt-6 text-center text-xs text-muted fade-up-3">{footerNote}</p>}
      </div>
    </main>
  );
}

export function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="section-callout space-y-4">
      <div className="border-b border-line pb-3">
        <h2 className="font-display text-base font-bold tracking-tight text-text sm:text-lg">{title}</h2>
        {description && <p className="mt-1 text-sm text-muted leading-relaxed">{description}</p>}
      </div>
      <div className="space-y-0">{children}</div>
    </section>
  );
}

export function SettingsRow({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-3 border-b border-line py-4 last:border-0 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0 sm:max-w-[42%]">
        <p className="text-sm font-medium text-text">{label}</p>
        {hint && <p className="mt-0.5 text-xs text-muted leading-relaxed">{hint}</p>}
      </div>
      <div className="min-w-0 w-full sm:max-w-sm sm:flex-1 sm:flex sm:justify-end">{children}</div>
    </div>
  );
}

export function Toast({
  text,
  type,
}: {
  text: string;
  type: "success" | "error";
}) {
  return (
    <div
      role="status"
      className={`fixed bottom-5 left-1/2 z-[70] max-w-[min(24rem,calc(100vw-2rem))] -translate-x-1/2 rounded-[10px] border px-4 py-3 text-sm shadow-lg fade-up ${
        type === "success" ? "form-msg-ok" : "form-msg-err"
      }`}
    >
      {text}
    </div>
  );
}
