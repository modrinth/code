"use client";

import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import ServerConnectCard from "@/components/server/ServerConnectCard";
import { useLocale } from "@/hooks/useLocale";

export default function ServersPage() {
  const { dict } = useLocale();
  const s = dict.servers;

  const steps = [
    { n: "01", title: s.step1Title, body: s.step1Body },
    { n: "02", title: s.step2Title, body: s.step2Body },
    { n: "03", title: s.step3Title, body: s.step3Body },
  ];

  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 py-16">
          <header className="max-w-2xl min-w-0">
            <div className="badge badge-accent">{s.badge}</div>
            <h1 className="mt-4 font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em]">{s.title}</h1>
            <p className="mt-4 text-muted text-lg leading-relaxed">{s.lead}</p>
          </header>

          <ServerConnectCard />

          <section className="mt-16">
            <h2 className="font-display text-2xl font-bold tracking-tight">{s.howTitle}</h2>
            <ol className="mt-4 border-t border-line">
              {steps.map((step) => (
                <li key={step.n} className="flex gap-5 py-5 border-b border-line">
                  <span className="font-mono text-sm text-accent pt-1 w-8 shrink-0">{step.n}</span>
                  <div className="min-w-0">
                    <h3 className="text-lg font-semibold">{step.title}</h3>
                    <p className="mt-1.5 text-sm text-muted leading-relaxed">{step.body}</p>
                  </div>
                </li>
              ))}
            </ol>
          </section>

          <section className="mt-12 border-t border-line pt-10 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-6">
            <div className="max-w-xl min-w-0">
              <h2 className="font-display text-xl font-bold tracking-tight">{s.ctaTitle}</h2>
              <p className="mt-2 text-muted">{s.ctaBody}</p>
            </div>
            <Link href="/download" className="btn btn-primary">
              {dict.home.downloadLauncher}
            </Link>
          </section>
        </div>
      </main>
      <Footer />
    </>
  );
}
