import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

export const metadata = {
  title: "Скачать лаунчер — Owyx",
  description: "Скачай лаунчер Owyx, введи ник и играй. Аккаунт нужен только для скина и плюшек.",
};

const REPO_URL = "https://github.com/ebluffy/Owyx";
const RELEASES_URL = `${REPO_URL}/releases`;
const DOWNLOAD_URL =
  process.env.NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL ||
  `${REPO_URL}/releases/latest/download/Owyx.exe`;

const steps = [
  { n: "1", title: "Скачай и запусти", text: "Один файл Owyx.exe для Windows. Установка не нужна." },
  { n: "2", title: "Введи ник (или войди)", text: "Гостю хватит ника. Аккаунт Owyx — скин и плюшки." },
  { n: "3", title: "Нажми Play", text: "Лаунчер сам скачает сборку и запустит игру." },
];

export default function DownloadPage() {
  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <section className="max-w-3xl mx-auto px-4 sm:px-6 py-16">
          <div className="mb-10">
            <p className="text-sm text-accent font-medium tracking-wide">Windows 10 / 11</p>
            <h1 className="font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em] mt-2 mb-3">
              Скачать Owyx
            </h1>
            <p className="text-lg text-muted max-w-2xl">
              Свой лаунчер Minecraft. Скачал, ввёл ник — играешь. Дальше — сервера сообщества.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-3 mb-4">
            <a href={DOWNLOAD_URL} className="btn btn-primary btn-lg" id="download-launcher" rel="noopener noreferrer">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M7 10l5 5 5-5M12 15V3" />
              </svg>
              Скачать Owyx.exe
            </a>
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="btn btn-ghost btn-lg" id="repo-link">
              Репозиторий
            </a>
          </div>
          <p className="text-muted text-sm mb-14 max-w-lg">
            Если публичного релиза ещё нет, ссылка ведёт на{" "}
            <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer" className="link-accent">страницу релизов</a>.
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
            Хочешь свой скин и плюшки?{" "}
            <Link href="/register" className="link-accent">Создай аккаунт Owyx</Link> и войди тем же аккаунтом в лаунчере.
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}
