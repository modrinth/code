"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

interface Category {
  id: number;
  title: string;
  slug: string;
  description?: string;
  icon?: string;
  color?: string;
  topics_count?: number;
  posts_count?: number;
  is_locked?: boolean;
}

export default function ForumPage() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const res = await fetch("/api/forum/categories");
        if (!res.ok) throw new Error("Не удалось загрузить форум");
        const data = await res.json();
        setCategories(data.categories ?? []);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Ошибка загрузки");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  return (
    <>
      <Header />
      <main className="min-h-[calc(100vh-64px)] py-10 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-[#FFFF55] text-shadow">Форум</h1>
            <p className="text-gray-400 mt-2">Обсуждения, гайды и новости сообщества</p>
          </div>

          {loading && <p className="text-gray-400">Загрузка категорий...</p>}
          {error && (
            <div className="p-4 rounded-lg bg-red-900/30 border border-red-500/30 text-red-300">
              {error}
            </div>
          )}

          {!loading && !error && categories.length === 0 && (
            <div className="glass-effect rounded-xl p-10 text-center text-gray-400">
              Категории пока не созданы. Их можно добавить через Directus или SQL.
            </div>
          )}

          <div className="space-y-3">
            {categories.map((cat) => (
              <Link
                key={cat.id}
                href={`/forum/${cat.slug}`}
                className="block glass-effect rounded-xl p-5 border border-[#FFAA00]/15 hover:border-[#FFAA00]/50 transition-colors"
              >
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <h2 className="text-lg font-bold text-[#FFAA00]">{cat.title}</h2>
                    {cat.description && (
                      <p className="text-gray-400 text-sm mt-1">{cat.description}</p>
                    )}
                  </div>
                  <div className="text-right text-xs text-gray-500 shrink-0">
                    <div>{cat.topics_count ?? 0} тем</div>
                    <div>{cat.posts_count ?? 0} сообщений</div>
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
