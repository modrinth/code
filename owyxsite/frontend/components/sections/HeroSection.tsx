"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";
import ApiServicesStatus from "@/components/brand/ApiServicesStatus";

/**
 * Brand-first first viewport (DESIGN.md §5):
 * Owyx™ · lead · CTA · optional API/services status (launcher-first — not a game-server IP).
 */
export default function HeroSection() {
  const { user, loading } = useAuth();
  const { dict } = useLocale();

  return (
    <section className="relative overflow-hidden min-h-[min(78vh,40rem)] flex items-center">
      <div className="hero-aurora" aria-hidden="true" />
      <div className="hero-grid" aria-hidden="true" />

      <div className="relative z-[1] w-full max-w-6xl mx-auto px-4 sm:px-6 pt-14 pb-16 sm:pt-20 sm:pb-24">
        <div className="max-w-2xl min-w-0">
          <p className="fade-up font-mono text-[11px] uppercase tracking-[0.22em] text-accent/70 mb-4">
            {dict.home.betaEyebrow}
          </p>
          <h1 className="fade-up font-display text-[clamp(3rem,11vw,5.75rem)] font-bold tracking-[-0.05em] leading-[0.92] break-words min-w-0">
            <span className="text-accent hero-title-glow">
              {dict.home.heroTitle}
              <sup className="ml-1 text-[0.35em] font-semibold tracking-normal text-accent/80 align-super">
                ™
              </sup>
            </span>
          </h1>
          <span className="hero-brand-underline" aria-hidden="true" />

          <p className="fade-up-2 mt-6 text-base sm:text-lg text-muted max-w-xl leading-relaxed">
            {dict.home.heroLead}
          </p>

          <div className="fade-up-3 mt-5">
            <ApiServicesStatus />
          </div>

          <div className="fade-up-3 mt-7 flex flex-wrap items-center gap-3">
            <Link href="/download" className="btn btn-primary btn-lg">
              {dict.home.downloadLauncher}
            </Link>
            {loading ? (
              <span className="btn btn-ghost pointer-events-none opacity-60">{dict.home.loading}</span>
            ) : user ? (
              <Link href="/profile" className="btn btn-ghost">
                {dict.home.toCabinet}
              </Link>
            ) : (
              <Link href="/login" className="btn btn-ghost">
                {dict.home.signInCreate}
              </Link>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
