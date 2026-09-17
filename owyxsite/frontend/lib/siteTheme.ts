"use client";

import { useCallback, useEffect, useState } from "react";

export type SiteAccent = "cyan" | "synthwave" | "onyx" | "magma";

const STORAGE_KEY = "owyx.siteTheme";
const ACCENTS: SiteAccent[] = ["cyan", "synthwave", "onyx", "magma"];

function isAccent(v: string | null): v is SiteAccent {
  return !!v && (ACCENTS as string[]).includes(v);
}

export function readSiteAccent(): SiteAccent {
  if (typeof window === "undefined") return "cyan";
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (isAccent(stored)) return stored;
  } catch {
    /* ignore */
  }
  return "cyan";
}

export function applySiteAccent(accent: SiteAccent) {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-accent", accent);
  try {
    localStorage.setItem(STORAGE_KEY, accent);
  } catch {
    /* ignore */
  }
}

export function useSiteAccent() {
  const [accent, setAccentState] = useState<SiteAccent>("cyan");

  useEffect(() => {
    const current = readSiteAccent();
    setAccentState(current);
    applySiteAccent(current);
  }, []);

  const setAccent = useCallback((next: SiteAccent) => {
    setAccentState(next);
    applySiteAccent(next);
  }, []);

  const cycle = useCallback(() => {
    setAccentState((prev) => {
      const i = ACCENTS.indexOf(prev);
      const next = ACCENTS[(i + 1) % ACCENTS.length];
      applySiteAccent(next);
      return next;
    });
  }, []);

  return { accent, setAccent, cycle, accents: ACCENTS };
}
