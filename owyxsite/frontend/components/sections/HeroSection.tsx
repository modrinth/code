"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/useAuth";
import BrandPalette from "@/components/brand/BrandPalette";

/**
 * Brand-first first viewport (DESIGN.md §5):
 * Owyx wordmark · one value line · primary CTA + secondary auth.
 */
export default function HeroSection() {
  const { user, loading } = useAuth();

  return (
    <section className="relative overflow-hidden">
      <BrandPalette />
      <div className="max-w-6xl mx-auto px-4 sm:px-6 pt-12 pb-16 sm:pt-16 sm:pb-20">
        <div className="max-w-2xl min-w-0">
          <h1 className="fade-up font-display text-[clamp(3rem,11vw,5.75rem)] font-bold tracking-[-0.05em] leading-[0.92] break-words min-w-0">
            <span className="text-accent">owyx</span>
          </h1>
          <span className="hero-brand-underline" aria-hidden="true" />

          <p className="fade-up-2 mt-5 text-base sm:text-lg text-muted max-w-xl leading-relaxed">
            Свой лаунчер Minecraft: зарегистрируйся на сайте, войди в лаунчере — играешь
            под своим логином. Microsoft-аккаунт опционален для лицензии и скина Mojang.
          </p>

          <div className="fade-up-3 mt-6 flex flex-wrap items-center gap-3">
            <Link href="/download" className="btn btn-primary btn-lg">
              Скачать лаунчер
            </Link>
            {loading ? (
              <span className="btn btn-ghost pointer-events-none opacity-60">Загрузка…</span>
            ) : user ? (
              <Link href="/profile" className="btn btn-ghost">
                В кабинет
              </Link>
            ) : (
              <Link href="/login" className="btn btn-ghost">
                Войти / Создать аккаунт
              </Link>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
