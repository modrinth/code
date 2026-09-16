import Link from "next/link";

const LEGAL = [
  { href: "/legal/terms", label: "Соглашение" },
  { href: "/legal/privacy", label: "Конфиденциальность" },
  { href: "/legal/eula", label: "EULA" },
  { href: "/legal/offer", label: "Оферта" },
];

export default function Footer() {
  return (
    <footer className="mt-auto border-t border-line/90">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 py-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p className="text-xs text-muted">
          <span className="font-display font-semibold tracking-tight text-accent">owyx</span>
          {" "}© {new Date().getFullYear()} · сделано ebluffy
        </p>
        <nav className="flex flex-wrap gap-x-5 gap-y-2 text-xs">
          {LEGAL.map((l) => (
            <Link key={l.href} href={l.href} className="text-muted hover:text-accent transition-colors">
              {l.label}
            </Link>
          ))}
          <a href="https://discord.gg/owyx" target="_blank" rel="noopener noreferrer" className="text-muted hover:text-accent transition-colors">
            Discord
          </a>
        </nav>
      </div>
      <div className="border-t border-line/80">
        <p className="max-w-6xl mx-auto px-4 sm:px-6 py-3.5 text-center text-[11px] leading-relaxed text-muted">
          Лаунчер использует код open-source клиента{" "}
          <a href="https://github.com/modrinth/code" target="_blank" rel="noopener noreferrer" className="link-accent">Modrinth (Theseus)</a>
          {" "}— форк под себя, не аффилирован с Modrinth.
        </p>
      </div>
    </footer>
  );
}
