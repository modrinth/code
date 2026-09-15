"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";

interface Post {
  id: number;
  content: string;
  created_at: string;
  author_nickname?: string;
  author_role?: string;
}

export default function ForumTopicPage() {
  const params = useParams();
  const id = String(params.id || "");
  const { isAuth } = useAuth();
  const [topic, setTopic] = useState<{ title: string; category_slug?: string; category_title?: string } | null>(null);
  const [posts, setPosts] = useState<Post[]>([]);
  const [reply, setReply] = useState("");
  const [sending, setSending] = useState(false);
  const [loading, setLoading] = useState(true);

  async function load() {
    const res = await fetch(`/api/forum/topics/${id}`);
    if (res.ok) {
      const data = await res.json();
      setTopic(data.topic);
      setPosts(data.posts ?? []);
    }
    setLoading(false);
  }

  useEffect(() => {
    if (id) load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  async function sendReply(e: React.FormEvent) {
    e.preventDefault();
    if (!reply.trim()) return;
    setSending(true);
    try {
      const res = await fetch(`/api/forum/topics/${id}/posts`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ content: reply }),
      });
      if (res.ok) {
        setReply("");
        await load();
      }
    } finally {
      setSending(false);
    }
  }

  return (
    <>
      <Header />
      <main className="min-h-[calc(100vh-64px)] py-10 px-4">
        <div className="max-w-3xl mx-auto">
          <div className="flex items-center gap-2 text-sm text-gray-500 mb-6">
            <Link href="/forum" className="hover:text-[#FFAA00]">Форум</Link>
            {topic?.category_slug && (
              <>
                <span>/</span>
                <Link href={`/forum/${topic.category_slug}`} className="hover:text-[#FFAA00]">
                  {topic.category_title}
                </Link>
              </>
            )}
          </div>

          {loading && <p className="text-gray-400">Загрузка...</p>}
          {topic && (
            <h1 className="text-2xl font-bold text-[#FFFF55] text-shadow mb-8">{topic.title}</h1>
          )}

          <div className="space-y-4 mb-10">
            {posts.map((p) => (
              <article key={p.id} className="glass-effect rounded-xl p-5 border border-[#FFAA00]/10">
                <div className="flex justify-between text-xs text-gray-500 mb-3">
                  <span className="text-[#FFAA00] font-medium">{p.author_nickname || "Игрок"}</span>
                  <span>{new Date(p.created_at).toLocaleString("ru-RU")}</span>
                </div>
                <div className="text-gray-200 whitespace-pre-wrap text-sm leading-relaxed">{p.content}</div>
              </article>
            ))}
          </div>

          {isAuth ? (
            <form onSubmit={sendReply} className="glass-effect rounded-xl p-5 space-y-3">
              <h3 className="text-[#FFAA00] font-bold">Ответить</h3>
              <textarea
                rows={4}
                value={reply}
                onChange={(e) => setReply(e.target.value)}
                className="w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
                placeholder="Ваш ответ..."
              />
              <button
                type="submit"
                disabled={sending || !reply.trim()}
                className="btn-minecraft px-6 py-2 rounded-lg font-bold disabled:opacity-60"
              >
                {sending ? "Отправка..." : "Отправить"}
              </button>
            </form>
          ) : (
            <p className="text-gray-400 text-sm">
              <Link href="/login" className="text-[#FFAA00] hover:underline">Войдите</Link>, чтобы ответить
            </p>
          )}
        </div>
      </main>
      <Footer />
    </>
  );
}
