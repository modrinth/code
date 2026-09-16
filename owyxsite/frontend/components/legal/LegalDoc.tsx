"use client";

import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useLocale } from "@/hooks/useLocale";
import type { LegalSlug } from "@/lib/legal";

export default function LegalDoc({ slug }: { slug: LegalSlug }) {
  const { dict, locale } = useLocale();
  const doc = dict.legal[slug];

  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <article className="max-w-3xl mx-auto px-4 sm:px-6 py-16">
          <p className="text-xs text-muted mb-2">
            <Link href="/" className="link-accent">
              {dict.legal.breadcrumbHome}
            </Link>{" "}
            · {dict.legal.breadcrumbDocs}
          </p>
          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight mb-2">{doc.title}</h1>
          <p className="text-xs text-muted mb-6">
            {locale === "en_US" ? "Last modified" : "Обновлено"}: {doc.lastModified}
          </p>
          <div className="panel p-4 mb-8 text-sm text-muted space-y-2">
            <p>{doc.intro}</p>
            <p className="text-xs opacity-80">{dict.legal.notLegalAdvice}</p>
          </div>
          <div className="space-y-8">
            {doc.sections.map((s) => (
              <section key={s.id} id={s.id}>
                <h2 className="text-lg font-semibold text-text mb-2">{s.title}</h2>
                <div className="space-y-3">
                  {s.paragraphs.map((p, i) => (
                    <p key={`${s.id}-${i}`} className="text-sm text-muted leading-relaxed">
                      {p}
                    </p>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </article>
      </main>
      <Footer />
    </>
  );
}
