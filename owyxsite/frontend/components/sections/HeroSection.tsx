"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";
import BrandPalette from "@/components/brand/BrandPalette";

/**
 * Brand-first first viewport (DESIGN.md §5):
 * Owyx™ wordmark · one value line · primary CTA + secondary auth.
 */
export default function HeroSection() {
  const { user, loading } = useAuth();
  const { dict } = useLocale();

  return (
    <section className="relative overflow-hidden">
      <BrandPalette />
      <div className="max-w-6xl mx-auto px-4 sm:px-6 pt-14 pb-16 sm:pt-20 sm:pb-24">
        <div className="max-w-2xl min-w-0">
          <h1 className="fade-up font-display text-[clamp(3rem,11vw,5.75rem)] font-bold tracking-[-0.05em] leading-[0.92] break-words min-w-0">
            <span className="text-accent">
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
