"use client";

import { Suspense, useState, useEffect, useCallback } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";
import CatalogAdmin from "@/components/admin/CatalogAdmin";

// Lightweight CMS-style control panel: manage accounts (nick / email / role).
// Deliberately NOT a moderation combine — no ban/delete/applications/trust here
// (open-access model; bans belong to the plugin, deletion breaks plugin links).

interface AdminUser {
  id: number;
  nickname?: string;
  email: string;
  role: string;
  status?: string;
  created_at?: string;
}

const ROLES = ["user", "helper", "moderator", "admin"] as const;

export default function AdminPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center">
          <p className="text-[#9aa0a8]">Загрузка...</p>
        </div>
      }
    >
      <AdminPageInner />
    </Suspense>
  );
}

function AdminPageInner() {
  const { user, loading: authLoading } = useAuth({ requireAuth: true });
  const router = useRouter();
  const searchParams = useSearchParams();
  const advanced = searchParams.get("advanced") === "1";

  const [users, setUsers] = useState<AdminUser[] | null>(null);
  const [search, setSearch] = useState("");
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [siteInfo, setSiteInfo] = useState<{ name?: string; ip?: string; online?: boolean } | null>(null);

  const isAdmin = user?.role === "admin";
  const dataLoading = users === null;

  const authHeaders = useCallback(() => ({
    Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
    "Content-Type": "application/json",
  }), []);

  const showMessage = useCallback((text: string, type: "success" | "error") => {
    setMsg({ text, type });
    setTimeout(() => setMsg(null), 4000);
  }, []);

  useEffect(() => {
    if (!isAdmin || authLoading) return;
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch("/api/admin/users", { headers: authHeaders() });
        if (!cancelled) {
          if (res.ok) {
            const data = await res.json();
            setUsers(data.users ?? data ?? []);
          } else {
            setUsers([]);
          }
        }
      } catch {
        if (!cancelled) setUsers([]);
      }
    })();
    (async () => {
      try {
        const res = await fetch("/api/settings/public");
        if (res.ok) {
          const d = await res.json();
          if (!cancelled) {
            setSiteInfo({ name: d.serverName ?? d.name, ip: d.serverIp ?? d.ip, online: d.online });
          }
        }
      } catch { /* ignore */ }
    })();
    return () => {
      cancelled = true;
    };
  }, [isAdmin, authLoading, authHeaders]);

  async function changeRole(id: number, role: string) {
    try {
      const res = await fetch(`/api/admin/users/${id}/role`, {
        method: "PUT",
        headers: authHeaders(),
        body: JSON.stringify({ role }),
      });
      if (res.ok) {
        setUsers((prev) => (prev ?? []).map((u) => (u.id === id ? { ...u, role } : u)));
        showMessage("Роль обновлена", "success");
      } else {
        const d = await res.json().catch(() => ({}));
        showMessage(d.error || "Не удалось изменить роль", "error");
      }
    } catch {
      showMessage("Не удалось связаться с сервером", "error");
    }
  }

  async function toggleBan(u: AdminUser) {
    const banned = u.status === "banned";
    const res = await fetch(`/api/admin/users/${u.id}/${banned ? "unban" : "ban"}`, {
      method: "POST", headers: authHeaders(),
      body: banned ? undefined : JSON.stringify({ reason: "Заблокирован из админ-панели" }),
    });
    if (res.ok) { setUsers((p) => (p ?? []).map((x) => x.id === u.id ? { ...x, status: banned ? "active" : "banned" } : x)); showMessage(banned ? "Разблокирован" : "Заблокирован", "success"); }
    else showMessage("Не удалось изменить статус", "error");
  }

  async function deleteUser(u: AdminUser) {
    const reason = window.prompt(`Удалить аккаунт ${u.nickname || u.email}? Причина (мин. 5 символов). Это может сломать связку с плагином.`);
    if (!reason || reason.trim().length < 5) { if (reason !== null) showMessage("Нужна причина (мин. 5 символов)", "error"); return; }
    const res = await fetch(`/api/admin/users/${u.id}/delete`, {
      method: "DELETE", headers: authHeaders(), body: JSON.stringify({ reason: reason.trim() }),
    });
    if (res.ok) { setUsers((p) => (p ?? []).filter((x) => x.id !== u.id)); showMessage("Аккаунт удалён", "success"); }
    else { const d = await res.json().catch(() => ({})); showMessage(d.error || "Не удалось удалить", "error"); }
  }

  const filtered = (users ?? []).filter(
    (u) => !search ||
      (u.nickname ?? "").toLowerCase().includes(search.toLowerCase()) ||
      u.email.toLowerCase().includes(search.toLowerCase())
  );

  if (authLoading) {
    return <div className="min-h-screen flex items-center justify-center"><p className="text-[#9aa0a8]">Загрузка...</p></div>;
  }

  if (!isAdmin) {
    return (
      <>
        <Header />
        <main id="main-content" className="relative z-10 flex-1 pt-24 pb-12">
          <div className="max-w-md mx-auto px-4">
            <div className="panel p-8 text-center">
              <h1 className="font-display text-2xl font-bold text-white mb-3">Доступ запрещён</h1>
              <p className="text-[#9aa0a8] mb-6">Нужны права администратора.</p>
              <button onClick={() => router.push("/profile")} className="btn btn-primary">В профиль</button>
            </div>
          </div>
        </main>
        <Footer />
      </>
    );
  }

  return (
    <>
      <Header />
      <main id="main-content" className="min-h-[calc(100vh-64px)] py-8 px-4">
        <div className="max-w-5xl mx-auto">
          <h1 className="font-display text-3xl font-bold text-white mb-2">Панель управления</h1>
          <p className="text-[#9aa0a8] mb-6">Каталог серверов и паков, аккаунты и роли. Заявок нет.</p>

          {msg && <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-5`}>{msg.text}</div>}

          {/* Site info (read-only) */}
          <div className="panel p-4 mb-6 flex flex-wrap items-center gap-x-8 gap-y-2 text-sm">
            <div><span className="text-[#9aa0a8]">Сервер:</span> <span className="text-white font-medium">{siteInfo?.name ?? "Owyx"}</span></div>
            <div><span className="text-[#9aa0a8]">IP:</span> <span className="text-white font-mono">{siteInfo?.ip ?? "play.owyx.site"}</span></div>
            <div><span className="text-[#9aa0a8]">Статус:</span> <span className={siteInfo?.online ? "text-[#5eead4]" : "text-[#ff5c6c]"}>{siteInfo?.online ? "онлайн" : "оффлайн"}</span></div>
            <div className="text-[#9aa0a8] ml-auto">Пользователей: <span className="text-white font-medium">{users?.length ?? 0}</span></div>
          </div>

          {/* Search */}
          <div className="field max-w-sm">
            <label className="field-label" htmlFor="user-search">Поиск по нику или email</label>
            <input id="user-search" className="input" value={search} onChange={(e) => setSearch(e.target.value)} placeholder="ник или email" />
          </div>

          {/* Accounts table */}
          <div className="panel overflow-hidden">
            {dataLoading ? (
              <p className="text-center text-[#9aa0a8] py-10">Загрузка аккаунтов...</p>
            ) : filtered.length === 0 ? (
              <p className="text-center text-[#9aa0a8] py-10">Ничего не найдено</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-left text-[#9aa0a8] border-b border-[#2a2b30]">
                      <th className="px-4 py-3 font-medium">Ник</th>
                      <th className="px-4 py-3 font-medium">Email</th>
                      <th className="px-4 py-3 font-medium">Роль</th>
                      {advanced && <th className="px-4 py-3 font-medium">Опасная зона</th>}
                    </tr>
                  </thead>
                  <tbody>
                    {filtered.map((u) => (
                      <tr key={u.id} className="border-b border-[#2a2b30] last:border-0 hover:bg-[#14141c]">
                        <td className="px-4 py-3 text-white font-medium">{u.nickname || "—"}</td>
                        <td className="px-4 py-3 text-[#9aa0a8]">{u.email}</td>
                        <td className="px-4 py-3">
                          <select
                            className="select !py-1.5 max-w-[10rem]"
                            value={ROLES.includes(u.role as typeof ROLES[number]) ? u.role : "user"}
                            onChange={(e) => changeRole(u.id, e.target.value)}
                            disabled={u.id === user?.id}
                            title={u.id === user?.id ? "Нельзя изменить свою роль" : "Сменить роль"}
                          >
                            {ROLES.map((r) => <option key={r} value={r}>{r}</option>)}
                          </select>
                        </td>
                        {advanced && (
                          <td className="px-4 py-3">
                            {u.id === user?.id ? (
                              <span className="text-xs text-[#9aa0a8]">—</span>
                            ) : (
                              <div className="flex gap-2">
                                <button onClick={() => toggleBan(u)} className="btn btn-secondary !py-1.5 !px-3 text-xs">
                                  {u.status === "banned" ? "Разбан" : "Бан"}
                                </button>
                                <button onClick={() => deleteUser(u)} className="btn btn-danger !py-1.5 !px-3 text-xs">Удалить</button>
                              </div>
                            )}
                          </td>
                        )}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          <div className="mt-10">
            <CatalogAdmin authHeaders={authHeaders} showMessage={showMessage} />
          </div>

          {/* News management */}
          <div className="mt-10">
            <NewsAdmin authHeaders={authHeaders} showMessage={showMessage} />
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

interface NewsRow {
  id: number;
  title: string;
  tag: string;
  summary: string;
  published: boolean;
  created_at?: string;
}

function NewsAdmin({
  authHeaders,
  showMessage,
}: {
  authHeaders: () => Record<string, string>;
  showMessage: (t: string, k: "success" | "error") => void;
}) {
  const [items, setItems] = useState<NewsRow[]>([]);
  const [form, setForm] = useState({ title: "", tag: "Новость", summary: "" });
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const res = await fetch("/api/admin/news", { headers: authHeaders() });
      if (res.ok) {
        const data = await res.json();
        setItems(data.news ?? []);
      }
    } catch { /* ignore */ }
  }, [authHeaders]);

  useEffect(() => { load(); }, [load]);

  async function create(e: React.FormEvent) {
    e.preventDefault();
    if (form.title.trim().length < 2) { showMessage("Заголовок слишком короткий", "error"); return; }
    setBusy(true);
    try {
      const res = await fetch("/api/admin/news", {
        method: "POST", headers: authHeaders(), body: JSON.stringify({ ...form, published: true }),
      });
      if (res.ok) { setForm({ title: "", tag: "Новость", summary: "" }); showMessage("Новость добавлена", "success"); await load(); }
      else { const d = await res.json().catch(() => ({})); showMessage(d.error || "Ошибка", "error"); }
    } catch { showMessage("Ошибка соединения", "error"); }
    finally { setBusy(false); }
  }

  async function togglePublish(n: NewsRow) {
    const res = await fetch(`/api/admin/news/${n.id}`, {
      method: "PUT", headers: authHeaders(), body: JSON.stringify({ published: !n.published }),
    });
    if (res.ok) await load(); else showMessage("Не удалось обновить", "error");
  }

  async function remove(id: number) {
    const res = await fetch(`/api/admin/news/${id}`, { method: "DELETE", headers: authHeaders() });
    if (res.ok) { showMessage("Удалено", "success"); await load(); } else showMessage("Не удалось удалить", "error");
  }

  return (
    <div>
      <h2 className="text-xl font-bold tracking-tight mb-1">Новости</h2>
      <p className="text-[#9aa0a8] text-sm mb-4">Появляются на главной. Снимите публикацию, чтобы скрыть.</p>

      <form onSubmit={create} className="panel p-4 mb-5 grid gap-3 sm:grid-cols-[1fr_auto]">
        <div className="space-y-3">
          <input className="input" placeholder="Заголовок" value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
          <div className="flex gap-3">
            <input className="input max-w-[10rem]" placeholder="Тег" value={form.tag} onChange={(e) => setForm((f) => ({ ...f, tag: e.target.value }))} />
            <input className="input flex-1" placeholder="Кратко" value={form.summary} onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))} />
          </div>
        </div>
        <button type="submit" disabled={busy} className="btn btn-primary self-start">{busy ? "..." : "Добавить"}</button>
      </form>

      <div className="space-y-2">
        {items.length === 0 ? (
          <p className="text-[#9aa0a8] text-sm">Пока нет новостей.</p>
        ) : items.map((n) => (
          <div key={n.id} className="panel p-3 flex items-center gap-3">
            <span className={`badge ${n.published ? "badge-accent" : ""}`}>{n.tag}</span>
            <div className="flex-1 min-w-0">
              <div className="text-white text-sm font-medium truncate">{n.title}</div>
              <div className="text-[#9aa0a8] text-xs truncate">{n.summary}</div>
            </div>
            <button onClick={() => togglePublish(n)} className="btn btn-secondary !py-1.5 !px-3 text-xs">
              {n.published ? "Скрыть" : "Опубликовать"}
            </button>
            <button onClick={() => remove(n.id)} className="btn btn-danger !py-1.5 !px-3 text-xs">Удалить</button>
          </div>
        ))}
      </div>
    </div>
  );
}
