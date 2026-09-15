"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

interface Topic {
  id: number;
  title: string;
  slug: string;
  is_pinned?: boolean;
  is_locked?: boolean;
  views_count?: number;
  posts_count?: number;
  created_at: string;
  author_nickname?: string;
}

export default function ForumCategoryPage() {
  const params = useParams();
  const slug = String(params.slug || "");
  const [topics, setTopics] = useState<Topic[]>([]);
  const [categoryTitle, setCategoryTitle] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!slug) return;
    async function load() {
      try {
        const res = await fetch(`/api/forum/categories/${encodeURIComponent(slug)}/topics`);
        if (res.ok) {
          const data = await res.json();
          setTopics(data.topics ?? []);
          setCategoryTitle(data.category?.title || slug);
        }
      } catch { /* ignore */ }
      setLoading(false);
    }
    load();
  }, [slug]);

  return (
    <>
      <Header />
      <main className="min-h-[calc(100vh-64px)] py-10 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-center gap-2 text-sm text-gray-500 mb-6">
            <Link href="/forum" className="hover:text-[#FFAA00]">Форум</Link>
            <span>/</span>
            <span className="text-[#FFAA00]">{categoryTitle || slug}</span>
          </div>
          <h1 className="text-3xl font-bold text-[#FFFF55] text-shadow mb-8">{categoryTitle}</h1>

          {loading && <p className="text-gray-400">Загрузка...</p>}
          {!loading && topics.length === 0 && (
            <p className="text-gray-500">В этой категории пока нет тем</p>
          )}

          <div className="space-y-2">
            {topics.map((t) => (
              <Link
                key={t.id}
                href={`/forum/topic/${t.id}`}
                className="block glass-effect rounded-lg px-4 py-3 border border-[#FFAA00]/10 hover:border-[#FFAA00]/40 transition-colors"
              >
                <div className="flex justify-between gap-3">
                  <div>
                    <span className="text-white font-medium">
                      {t.is_pinned ? "📌 " : ""}
                      {t.title}
                    </span>
                    <p className="text-xs text-gray-500 mt-1">
                      {t.author_nickname || "Аноним"} · {new Date(t.created_at).toLocaleDateString("ru-RU")}
                    </p>
                  </div>
                  <div className="text-xs text-gray-500 text-right">
                    <div>{t.posts_count ?? 0} ответов</div>
                    <div>{t.views_count ?? 0} просмотров</div>
                  </div>
                </div>
              </Link>
            ))}
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
