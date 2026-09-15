import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

/** Honest "coming soon" section in the new skin for parked community pages. */
export default function ComingSoon({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1 grid place-items-center px-4 py-16">
        <div className="empty-surface max-w-md w-full">
          <span className="badge badge-accent">Скоро</span>
          <h1 className="mt-4 font-display text-3xl font-bold tracking-[-0.03em]">{title}</h1>
          <p className="mt-3 text-muted leading-relaxed">{description}</p>
          <div className="mt-8 flex justify-center gap-3 flex-wrap">
            <Link href="/" className="btn btn-ghost">На главную</Link>
            <Link href="/servers" className="btn btn-primary">Свои сервера</Link>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
