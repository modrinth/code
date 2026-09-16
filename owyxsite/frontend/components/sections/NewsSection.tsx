"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useLocale } from "@/hooks/useLocale";

type NewsItem = {
  id?: number;
  title: string;
  tag: string;
  summary: string;
  created_at?: string;
};

export default function NewsSection() {
  const { locale, dict } = useLocale();
  const [news, setNews] = useState<NewsItem[] | null>(null);

  const FALLBACK: NewsItem[] = [
    {
      title: locale === "en_US" ? "Sign in with your Owyx™ account" : "Вход по аккаунту Owyx™",
      tag: dict.home.newsFallbackTagLauncher,
      summary:
        locale === "en_US"
          ? "You can sign into the launcher with your site account — nickname, role, and skin sync automatically."
          : "В лаунчере можно войти аккаунтом сайта — ник, роль и скин подтягиваются автоматически.",
    },
    {
      title: locale === "en_US" ? "Skins and capes" : "Скины и плащи",
      tag: dict.home.newsFallbackTagCosmetics,
      summary:
        locale === "en_US"
          ? "Upload your skin in the profile — the launcher picks it up when you sign in. Capes are on the way."
          : "Загружай свой скин в профиле — лаунчер подхватит его при входе. Плащи на подходе.",
    },
  ];

  const tagClass: Record<string, string> = {
    [dict.home.newsFallbackTagLauncher]: "badge-accent",
    [dict.home.newsFallbackTagCosmetics]: "badge-ok",
    Лаунчер: "badge-accent",
    Косметика: "badge-ok",
    Launcher: "badge-accent",
    Cosmetics: "badge-ok",
  };

  function fmtDate(iso?: string) {
    if (!iso) return "";
    try {
      return new Date(iso).toLocaleDateString(locale === "en_US" ? "en-US" : "ru-RU", {
        day: "numeric",
        month: "short",
        year: "numeric",
      });
    } catch {
      return "";
    }
  }

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
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- refetch when locale changes for fallback copy
  }, [locale]);

  return (
    <section className="max-w-6xl mx-auto px-4 sm:px-6 py-12 sm:py-14">
      <div className="flex items-end justify-between gap-4 mb-5 sm:mb-6">
        <h2 className="font-display text-2xl font-bold tracking-tight">{dict.home.newsTitle}</h2>
      </div>

      {news === null ? (
        <p className="text-muted text-sm py-8">{dict.home.loading}</p>
      ) : news.length === 0 ? (
        <div className="empty-surface">
          <h3>{dict.home.newsEmptyTitle}</h3>
          <p className="mt-2">{dict.home.newsEmptyBody}</p>
          <div className="mt-6 flex justify-center">
            <Link href="/download" className="btn btn-primary">
              {dict.home.downloadLauncher}
            </Link>
          </div>
        </div>
      ) : (
        <div className="divide-y divide-line border-y border-line">
          {news.map((n, i) => (
            <article key={n.id ?? i} className="py-5 flex flex-col gap-2 sm:flex-row sm:items-start sm:gap-6">
              <div className="sm:w-36 shrink-0 flex items-center gap-2 pt-0.5">
                <span className={`badge ${tagClass[n.tag] ?? ""}`}>{n.tag}</span>
                {n.created_at && (
                  <span className="text-xs text-muted sm:hidden">{fmtDate(n.created_at)}</span>
                )}
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-baseline justify-between gap-3">
                  <h3 className="font-display text-base font-semibold tracking-tight text-text leading-snug">
                    {n.title}
                  </h3>
                  {n.created_at && (
                    <span className="hidden sm:inline text-xs text-muted whitespace-nowrap">
                      {fmtDate(n.created_at)}
                    </span>
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
