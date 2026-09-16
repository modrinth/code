import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import HeroSection from "@/components/sections/HeroSection";
import NewsSection from "@/components/sections/NewsSection";

// Minimalist home: hero → news → one closing CTA. No dashboard clutter.
export default function HomePage() {
  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <HeroSection />

        <NewsSection />

        <section className="max-w-6xl mx-auto px-4 sm:px-6 pb-20">
          <div className="section-callout flex flex-col sm:flex-row sm:items-center sm:justify-between gap-6">
            <div className="min-w-0">
              <h2 className="font-display text-xl font-bold tracking-tight">Готов играть?</h2>
              <p className="mt-2 text-sm text-muted leading-relaxed max-w-lg">
                Скачай лаунчер и введи ник, или создай аккаунт Owyx — для скина и плюшек.
              </p>
            </div>
            <div className="flex flex-wrap gap-3 shrink-0">
              <Link href="/download" className="btn btn-primary">Скачать лаунчер</Link>
              <Link href="/register" className="btn btn-ghost">Создать аккаунт</Link>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}
