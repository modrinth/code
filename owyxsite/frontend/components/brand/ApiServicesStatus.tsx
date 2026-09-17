"use client";

import { useEffect, useState } from "react";
import { useLocale } from "@/hooks/useLocale";

type HealthPayload = {
  status?: string;
  services?: {
    database?: string;
    server?: string;
  };
};

type Row = { id: string; label: string; ok: boolean | null };

/**
 * Compact control-plane status (API / DB) — launcher-first, not a game-server IP.
 * Realtime omitted: backend reports socketio as a static "enabled" flag, not live health.
 */
export default function ApiServicesStatus() {
  const { locale } = useLocale();
  const [rows, setRows] = useState<Row[]>([
    { id: "api", label: "API", ok: null },
    { id: "db", label: locale === "ru" ? "База" : "Database", ok: null },
  ]);

  useEffect(() => {
    let cancelled = false;
    const ctrl = new AbortController();
    const timer = window.setTimeout(() => ctrl.abort(), 2500);

    (async () => {
      try {
        const healthRes = await fetch("/health", { signal: ctrl.signal, cache: "no-store" });
        if (!healthRes.ok) throw new Error("unhealthy");
        const data = (await healthRes.json()) as HealthPayload;
        if (cancelled) return;

        const dbOk = data.services?.database === "connected";
        const apiOk = data.status === "healthy" || data.services?.server === "running";

        setRows([
          { id: "api", label: "API", ok: Boolean(apiOk) },
          {
            id: "db",
            label: locale === "ru" ? "База" : "Database",
            ok: Boolean(dbOk),
          },
        ]);
      } catch {
        if (cancelled) return;
        setRows((prev) => prev.map((r) => ({ ...r, ok: false })));
      } finally {
        window.clearTimeout(timer);
      }
    })();

    return () => {
      cancelled = true;
      ctrl.abort();
    };
  }, [locale]);

  const title = locale === "ru" ? "Сервисы Owyx" : "Owyx services";

  return (
    <div
      className="inline-flex flex-wrap items-center gap-2 rounded-[10px] border border-line bg-panel/70 px-3 py-2 backdrop-blur-sm"
      aria-label={title}
    >
      <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-muted">{title}</span>
      {rows.map((row) => (
        <span
          key={row.id}
          className="inline-flex items-center gap-1.5 rounded-md border border-line/80 bg-background/40 px-2 py-1 text-xs text-text"
        >
          <span
            className={
              row.ok === null
                ? "h-1.5 w-1.5 rounded-full bg-muted animate-pulse"
                : row.ok
                  ? "h-1.5 w-1.5 rounded-full bg-ok shadow-[0_0_8px_rgba(94,234,212,0.55)]"
                  : "h-1.5 w-1.5 rounded-full bg-danger"
            }
            aria-hidden
          />
          <span className="font-medium">{row.label}</span>
          <span className="text-muted tabular-nums">
            {row.ok === null
              ? "…"
              : row.ok
                ? locale === "ru"
                  ? "ок"
                  : "ok"
                : locale === "ru"
                  ? "нет"
                  : "down"}
          </span>
        </span>
      ))}
    </div>
  );
}
