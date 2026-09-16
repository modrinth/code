import { notFound } from "next/navigation";
import LegalDoc from "@/components/legal/LegalDoc";
import { getDictionary, DEFAULT_LOCALE } from "@/lib/i18n";
import { isLegalSlug } from "@/lib/legal";

export function generateStaticParams() {
  return [{ slug: "terms" }, { slug: "privacy" }, { slug: "eula" }];
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  if (!isLegalSlug(slug)) return { title: "Owyx" };
  const doc = getDictionary(DEFAULT_LOCALE).legal[slug];
  return { title: `${doc.title} — Owyx` };
}

export default async function LegalPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  if (slug === "offer" || !isLegalSlug(slug)) notFound();
  return <LegalDoc slug={slug} />;
}
