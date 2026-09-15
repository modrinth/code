import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import ServerConnectCard from "@/components/server/ServerConnectCard";

export const metadata = {
  title: "Серверы — Owyx",
  description: "Свои приватные сервера Owyx: IP, статус и как начать играть.",
};

const START = [
  { n: "01", title: "Скачай лаунчер", body: "Один файл Owyx.exe. Запустил — можно играть по нику." },
  { n: "02", title: "Аккаунт (по желанию)", body: "Зарегистрируйся на сайте и войди в лаунчере — скин и плюшки." },
  { n: "03", title: "Play", body: "Подключайся к своим серверам. Без заявок и ожидания." },
];

export default function ServersPage() {
  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 py-16">
          <header className="max-w-2xl min-w-0">
            <div className="badge badge-accent">Приватные сервера</div>
            <h1 className="mt-4 font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em]">
              Свои сервера Owyx
            </h1>
            <p className="mt-4 text-muted text-lg leading-relaxed">
              Не generic-хостинг, а свои сервера под сообщество. Скачал лаунчер — играешь.
            </p>
          </header>

          <ServerConnectCard />

          <section className="mt-16">
            <h2 className="font-display text-2xl font-bold tracking-tight">Как начать играть</h2>
            <ol className="mt-4 border-t border-line">
              {START.map((s) => (
                <li key={s.n} className="flex gap-5 py-5 border-b border-line">
                  <span className="font-mono text-sm text-accent pt-1 w-8 shrink-0">{s.n}</span>
                  <div className="min-w-0">
                    <h3 className="text-lg font-semibold">{s.title}</h3>
                    <p className="mt-1.5 text-sm text-muted leading-relaxed">{s.body}</p>
                  </div>
                </li>
              ))}
            </ol>
          </section>

          <section className="mt-12 border-t border-line pt-10 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-6">
            <div className="max-w-xl min-w-0">
              <h2 className="font-display text-xl font-bold tracking-tight">Лаунчер Owyx</h2>
              <p className="mt-2 text-muted">
                Offline по нику или вход аккаунтом Owyx. Microsoft — отдельно, когда понадобится.
              </p>
            </div>
            <div className="flex flex-wrap gap-3">
              <Link href="/download" className="btn btn-primary">
                Скачать лаунчер
              </Link>
              <span className="badge">Windows · v0.1</span>
            </div>
          </section>
        </div>
      </main>
      <Footer />
    </>
  );
}
