"use client";

import Link from "next/link";
import { useLocale } from "@/hooks/useLocale";

export default function Footer() {
  const { dict } = useLocale();
  const LEGAL = [
    { href: "/legal/terms", label: dict.footer.terms },
    { href: "/legal/privacy", label: dict.footer.privacy },
    { href: "/legal/eula", label: dict.footer.eula },
  ];

  return (
    <footer className="mt-auto border-t border-line/90">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 py-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p className="text-xs text-muted">
          <span className="font-display font-semibold tracking-tight text-accent">
            owyx
            <sup className="ml-0.5 text-[0.55em] font-semibold text-accent/80">™</sup>
          </span>
          {" "}© {new Date().getFullYear()} · {dict.footer.madeBy}
        </p>
        <nav className="flex flex-wrap gap-x-5 gap-y-2 text-xs">
          {LEGAL.map((l) => (
            <Link key={l.href} href={l.href} className="text-muted hover:text-accent transition-colors">
              {l.label}
            </Link>
          ))}
          <a href="https://discord.gg/owyx" target="_blank" rel="noopener noreferrer" className="text-muted hover:text-accent transition-colors">
            {dict.footer.discord}
          </a>
        </nav>
      </div>
      <div className="border-t border-line/80">
        <p className="max-w-6xl mx-auto px-4 sm:px-6 py-3.5 text-center text-[11px] leading-relaxed text-muted">
          {dict.footer.attribution}
        </p>
      </div>
    </footer>
  );
}
