"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

type NewsItem = {
  id?: number;
  title: string;
  tag: string;
  summary: string;
  created_at?: string;
};

/** Shown only when the API is unreachable (network / timeout / 5xx). */
const FALLBACK: NewsItem[] = [
  { title: "Вход по аккаунту Owyx", tag: "Лаунчер", summary: "В лаунчере можно войти аккаунтом сайта — ник, роль и скин подтягиваются автоматически." },
  { title: "Скины и плащи", tag: "Косметика", summary: "Загружай свой скин в профиле — лаунчер подхватит его при входе. Плащи на подходе." },
];

const tagClass: Record<string, string> = {
  Лаунчер: "badge-accent",
  Косметика: "badge-ok",
};

function fmtDate(iso?: string) {
  if (!iso) return "";
  try {
    return new Date(iso).toLocaleDateString("ru-RU", { day: "numeric", month: "short", year: "numeric" });
  } catch {
    return "";
  }
}

export default function NewsSection() {
  const [news, setNews] = useState<NewsItem[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    const ctrl = new AbortController();
    const timer = window.setTimeout(() => ctrl.abort(), 2500);
    (async () => {
      try {
        const res = await fetch("/api/news?limit=6", { signal: ctrl.signal });
        if (!res.ok) throw new Error(`news ${res.status}`);
        const data = await res.json();
        if (cancelled) return;
        // Empty successful payload → honest empty surface (not FALLBACK).
        setNews(Array.isArray(data.news) ? data.news : []);
      } catch {
        if (!cancelled) setNews(FALLBACK);
      } finally {
        window.clearTimeout(timer);
      }
    })();
    return () => {
      cancelled = true;
      ctrl.abort();
      window.clearTimeout(timer);
    };
  }, []);

  return (
    <section className="max-w-6xl mx-auto px-4 sm:px-6 py-12 sm:py-14">
      <div className="flex items-end justify-between gap-4 mb-5 sm:mb-6">
        <h2 className="font-display text-2xl font-bold tracking-tight">Новости и обновления</h2>
      </div>

      {news === null ? (
        <p className="text-muted text-sm py-8">Загрузка…</p>
      ) : news.length === 0 ? (
        <div className="empty-surface">
          <h3>Пока тихо</h3>
          <p className="mt-2">Новости появятся здесь. А пока — скачай лаунчер и зайди на сервер.</p>
          <div className="mt-6 flex justify-center">
            <Link href="/download" className="btn btn-primary">Скачать лаунчер</Link>
          </div>
        </div>
      ) : (
        <div className="divide-y divide-line border-y border-line">
          {news.map((n, i) => (
            <article key={n.id ?? i} className="py-5 flex flex-col gap-2 sm:flex-row sm:items-start sm:gap-6">
              <div className="sm:w-36 shrink-0 flex items-center gap-2 pt-0.5">
                <span className={`badge ${tagClass[n.tag] ?? ""}`}>{n.tag}</span>
                {n.created_at && <span className="text-xs text-muted sm:hidden">{fmtDate(n.created_at)}</span>}
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-baseline justify-between gap-3">
                  <h3 className="font-display text-base font-semibold tracking-tight text-text leading-snug">{n.title}</h3>
                  {n.created_at && (
                    <span className="hidden sm:inline text-xs text-muted whitespace-nowrap">{fmtDate(n.created_at)}</span>
                  )}
                </div>
                <p className="mt-1.5 text-sm text-muted leading-relaxed">{n.summary}</p>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
