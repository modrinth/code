"use client";

import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useLocale } from "@/hooks/useLocale";

const REPO_URL = "https://github.com/ebluffy/Owyx";
const RELEASES_URL = `${REPO_URL}/releases/latest`;

/** Prefer env overrides; otherwise point at latest GitHub release assets. */
const WINDOWS_URL =
  process.env.NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL_WINDOWS ||
  process.env.NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL ||
  `${REPO_URL}/releases/latest/download/Owyx_0.5.2_x64-setup.exe`;

const LINUX_URL =
  process.env.NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL_LINUX ||
  `${REPO_URL}/releases/latest/download/Owyx_0.5.2_amd64.AppImage`;

function WindowsIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M3 5.5 10.5 4.4v7.1H3V5.5Zm0 13 7.5 1.1v-7.2H3v6.1ZM11.5 4.25 21 3v8.5h-9.5V4.25ZM11.5 20.9 21 22v-9.6h-9.5v8.5Z" />
    </svg>
  );
}

function LinuxIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12.5 2.2c-.9 0-1.7.7-1.9 1.7-.1.4-.3.8-.6 1.1-.8.8-1.3 1.9-1.3 3.1v.4c-.9.5-1.5 1.5-1.5 2.6 0 .6.2 1.2.5 1.6-.5.7-.8 1.6-.8 2.5 0 1.4.7 2.7 1.8 3.5-.2.5-.3 1-.3 1.6 0 1.9 1.4 3.5 3.3 3.8.3.7 1 1.2 1.8 1.2.6 0 1.1-.2 1.5-.6.4.4.9.6 1.5.6.8 0 1.5-.5 1.8-1.2 1.9-.3 3.3-1.9 3.3-3.8 0-.6-.1-1.1-.3-1.6 1.1-.8 1.8-2.1 1.8-3.5 0-.9-.3-1.8-.8-2.5.3-.4.5-1 .5-1.6 0-1.1-.6-2.1-1.5-2.6v-.4c0-1.2-.5-2.3-1.3-3.1-.3-.3-.5-.7-.6-1.1-.2-1-1-1.7-1.9-1.7h-1Zm0 1.6h1c.3 0 .6.3.7.6.1.5.4 1 .8 1.4.5.5.8 1.2.8 1.9v1.1l.7.3c.5.2.8.7.8 1.3 0 .4-.2.8-.5 1l-.5.4.3.5c.3.4.4.9.4 1.4 0 1.1-.7 2.1-1.7 2.5l-.6.2.1.6c.1.4.1.7.1 1.1 0 1.2-.9 2.2-2.1 2.3h-.3l-.2.5c-.1.3-.4.5-.7.5s-.6-.2-.7-.5l-.2-.5h-.3c-1.2-.1-2.1-1.1-2.1-2.3 0-.4 0-.7.1-1.1l.1-.6-.6-.2c-1-.4-1.7-1.4-1.7-2.5 0-.5.1-1 .4-1.4l.3-.5-.5-.4c-.3-.2-.5-.6-.5-1 0-.6.3-1.1.8-1.3l.7-.3V8.8c0-.7.3-1.4.8-1.9.4-.4.7-.9.8-1.4.1-.3.4-.6.7-.6Z" />
    </svg>
  );
}

export default function DownloadPage() {
  const { dict, locale } = useLocale();
  const d = dict.download;
  const steps = [
    { n: "1", title: d.step1Title, text: d.step1Text },
    { n: "2", title: d.step2Title, text: d.step2Text },
    { n: "3", title: d.step3Title, text: d.step3Text },
  ];

  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <section className="max-w-3xl mx-auto px-4 sm:px-6 py-16">
          <div className="mb-10">
            <p className="text-sm text-accent font-medium tracking-wide">{d.eyebrow}</p>
            <h1 className="font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em] mt-2 mb-3">
              {d.title}
            </h1>
            <p className="text-lg text-muted max-w-2xl">{d.lead}</p>
          </div>

          <div className="flex flex-wrap items-center gap-3 mb-4">
            <span className="text-sm font-medium text-text mr-1">{d.downloadLabel}</span>
            <a
              href={WINDOWS_URL}
              className="btn btn-primary"
              id="download-launcher-windows"
              rel="noopener noreferrer"
            >
              <WindowsIcon className="w-4 h-4" />
              Windows
            </a>
            <a
              href={LINUX_URL}
              className="btn btn-secondary"
              id="download-launcher-linux"
              rel="noopener noreferrer"
            >
              <LinuxIcon className="w-4 h-4" />
              Linux
            </a>
            <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer" className="btn btn-ghost">
              {d.allReleases}
            </a>
          </div>
          <p className="text-muted text-sm mb-14 max-w-lg">
            {locale === "en_US" ? "Current files:" : "Актуальные файлы:"}{" "}
            <code className="text-xs">Owyx_*_x64-setup.exe</code> /{" "}
            <code className="text-xs">Owyx_*_amd64.AppImage</code> —{" "}
            <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer" className="link-accent">
              GitHub Releases
            </a>
            .
          </p>

          <ol className="space-y-0 border-t border-line">
            {steps.map((s) => (
              <li key={s.n} className="flex gap-5 py-6 border-b border-line">
                <span className="font-mono text-accent text-sm pt-1 w-6 shrink-0">{s.n}</span>
                <div className="min-w-0">
                  <h2 className="text-text font-semibold">{s.title}</h2>
                  <p className="text-muted text-sm leading-relaxed mt-1">{s.text}</p>
                </div>
              </li>
            ))}
          </ol>

          <div className="mt-10 text-muted text-sm">
            {locale === "en_US" ? (
              <>
                Want a skin and extras?{" "}
                <Link href="/register" className="link-accent">
                  Create an Owyx™ account
                </Link>{" "}
                and sign in with the same account in the launcher.
              </>
            ) : (
              <>
                Хочешь свой скин и плюшки?{" "}
                <Link href="/register" className="link-accent">
                  Создай аккаунт Owyx™
                </Link>{" "}
                и войди тем же аккаунтом в лаунчере.
              </>
            )}
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}
