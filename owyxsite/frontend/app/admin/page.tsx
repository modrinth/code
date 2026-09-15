"use client";

import { Suspense, useState, useEffect, useCallback } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";
import CatalogAdmin from "@/components/admin/CatalogAdmin";

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
          <p className="text-muted">Загрузка...</p>
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
  const [section, setSection] = useState<"users" | "catalog" | "news">("users");

  const isAdmin = user?.role === "admin";
  const dataLoading = users === null;

  const authHeaders = useCallback(
    () => ({
      Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
      "Content-Type": "application/json",
    }),
    []
  );

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
      } catch {
        /* ignore */
      }
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
      method: "POST",
      headers: authHeaders(),
      body: banned ? undefined : JSON.stringify({ reason: "Заблокирован из админ-панели" }),
    });
    if (res.ok) {
      setUsers((p) =>
        (p ?? []).map((x) => (x.id === u.id ? { ...x, status: banned ? "active" : "banned" } : x))
      );
      showMessage(banned ? "Разблокирован" : "Заблокирован", "success");
    } else showMessage("Не удалось изменить статус", "error");
  }

  async function deleteUser(u: AdminUser) {
    const reason = window.prompt(
      `Удалить аккаунт ${u.nickname || u.email}? Причина (мин. 5 символов). Это может сломать связку с плагином.`
    );
    if (!reason || reason.trim().length < 5) {
      if (reason !== null) showMessage("Нужна причина (мин. 5 символов)", "error");
      return;
    }
    const res = await fetch(`/api/admin/users/${u.id}/delete`, {
      method: "DELETE",
      headers: authHeaders(),
      body: JSON.stringify({ reason: reason.trim() }),
    });
    if (res.ok) {
      setUsers((p) => (p ?? []).filter((x) => x.id !== u.id));
      showMessage("Аккаунт удалён", "success");
    } else {
      const d = await res.json().catch(() => ({}));
      showMessage(d.error || "Не удалось удалить", "error");
    }
  }

  const filtered = (users ?? []).filter(
    (u) =>
      !search ||
      (u.nickname ?? "").toLowerCase().includes(search.toLowerCase()) ||
      u.email.toLowerCase().includes(search.toLowerCase())
  );

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-muted">Загрузка...</p>
      </div>
    );
  }

  if (!isAdmin) {
    return (
      <>
        <Header />
        <main id="main-content" className="relative z-10 flex-1 pt-24 pb-12">
          <div className="max-w-md mx-auto px-4">
            <div className="rounded-2xl border border-line bg-panel p-8 text-center">
              <h1 className="font-display text-2xl font-bold text-text mb-3">Доступ запрещён</h1>
              <p className="text-muted mb-6">Нужны права администратора.</p>
              <button onClick={() => router.push("/profile")} className="btn btn-primary">
                В профиль
              </button>
            </div>
          </div>
        </main>
        <Footer />
      </>
    );
  }

  const nav = [
    { id: "users" as const, label: "Аккаунты" },
    { id: "catalog" as const, label: "Каталог" },
    { id: "news" as const, label: "Новости" },
  ];

  return (
    <>
      <Header />
      <main id="main-content" className="relative min-h-[calc(100vh-64px)] overflow-hidden">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0"
          style={{
            background:
              "radial-gradient(ellipse 60% 45% at 10% 0%, rgba(0,229,255,0.10), transparent 50%), radial-gradient(ellipse 40% 35% at 100% 10%, rgba(255,92,108,0.06), transparent 45%)",
          }}
        />
        <div className="relative z-10 mx-auto max-w-5xl px-4 py-10 sm:px-6">
          <header className="mb-8">
            <p className="text-xs uppercase tracking-[0.14em] text-muted">Control plane</p>
            <h1 className="font-display text-3xl font-bold tracking-tight text-text mt-1">Панель управления</h1>
            <p className="mt-2 text-sm text-muted max-w-xl">
              Каталог серверов и паков, роли аккаунтов, новости. Без заявок и модерации чата.
            </p>
          </header>

          {msg && (
            <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-5`}>
              {msg.text}
            </div>
          )}

          <div className="mb-6 grid gap-3 sm:grid-cols-3">
            <StatCard label="Сервер" value={siteInfo?.name ?? "Owyx"} />
            <StatCard label="IP" value={siteInfo?.ip ?? "—"} mono />
            <StatCard
              label="Пользователи"
              value={String(users?.length ?? "…")}
              hint={siteInfo?.online ? "статус: онлайн" : "статус: оффлайн"}
              hintOk={Boolean(siteInfo?.online)}
            />
          </div>

          <div className="overflow-hidden rounded-2xl border border-line bg-panel/90 backdrop-blur-sm">
            <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4">
              <nav className="flex gap-1" aria-label="Разделы админки">
                {nav.map((n) => (
                  <button
                    key={n.id}
                    type="button"
                    onClick={() => setSection(n.id)}
                    className={`relative px-4 py-3.5 text-sm font-medium transition-colors ${
                      section === n.id ? "text-accent" : "text-muted hover:text-text"
                    }`}
                  >
                    {n.label}
                    {section === n.id && (
                      <span className="absolute inset-x-3 bottom-0 h-0.5 rounded-full bg-accent" />
                    )}
                  </button>
                ))}
              </nav>
            </div>

            <div className="p-5 sm:p-6">
              {section === "users" && (
                <div className="space-y-5">
                  <div className="field max-w-sm">
                    <label className="field-label" htmlFor="user-search">
                      Поиск по нику или email
                    </label>
                    <input
                      id="user-search"
                      className="input"
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                      placeholder="ник или email"
                    />
                  </div>

                  <div className="overflow-hidden rounded-xl border border-line">
                    {dataLoading ? (
                      <p className="text-center text-muted py-10">Загрузка аккаунтов...</p>
                    ) : filtered.length === 0 ? (
                      <p className="text-center text-muted py-10">Ничего не найдено</p>
                    ) : (
                      <div className="overflow-x-auto">
                        <table className="w-full text-sm">
                          <thead>
                            <tr className="text-left text-muted border-b border-line bg-panel-2/50">
                              <th className="px-4 py-3 font-medium">Ник</th>
                              <th className="px-4 py-3 font-medium">Email</th>
                              <th className="px-4 py-3 font-medium">Роль</th>
                              {advanced && <th className="px-4 py-3 font-medium">Опасная зона</th>}
                            </tr>
                          </thead>
                          <tbody>
                            {filtered.map((u) => (
                              <tr
                                key={u.id}
                                className="border-b border-line last:border-0 hover:bg-panel-2/40"
                              >
                                <td className="px-4 py-3 text-text font-medium">{u.nickname || "—"}</td>
                                <td className="px-4 py-3 text-muted">{u.email}</td>
                                <td className="px-4 py-3">
                                  <select
                                    className="select !py-1.5 max-w-[10rem]"
                                    value={ROLES.includes(u.role as (typeof ROLES)[number]) ? u.role : "user"}
                                    onChange={(e) => changeRole(u.id, e.target.value)}
                                    disabled={u.id === user?.id}
                                    title={u.id === user?.id ? "Нельзя изменить свою роль" : "Сменить роль"}
                                  >
                                    {ROLES.map((r) => (
                                      <option key={r} value={r}>
                                        {r}
                                      </option>
                                    ))}
                                  </select>
                                </td>
                                {advanced && (
                                  <td className="px-4 py-3">
                                    {u.id === user?.id ? (
                                      <span className="text-xs text-muted">—</span>
                                    ) : (
                                      <div className="flex gap-2">
                                        <button
                                          onClick={() => toggleBan(u)}
                                          className="btn btn-secondary !py-1.5 !px-3 text-xs"
                                        >
                                          {u.status === "banned" ? "Разбан" : "Бан"}
                                        </button>
                                        <button
                                          onClick={() => deleteUser(u)}
                                          className="btn btn-danger !py-1.5 !px-3 text-xs"
                                        >
                                          Удалить
                                        </button>
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
                  {!advanced && (
                    <p className="text-xs text-muted">
                      Бан/удаление: открой{" "}
                      <a className="text-accent hover:underline" href="/admin?advanced=1">
                        /admin?advanced=1
                      </a>
                      .
                    </p>
                  )}
                </div>
              )}

              {section === "catalog" && (
                <CatalogAdmin authHeaders={authHeaders} showMessage={showMessage} />
              )}

              {section === "news" && (
                <NewsAdmin authHeaders={authHeaders} showMessage={showMessage} />
              )}
            </div>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

function StatCard({
  label,
  value,
  hint,
  hintOk,
  mono,
}: {
  label: string;
  value: string;
  hint?: string;
  hintOk?: boolean;
  mono?: boolean;
}) {
  return (
    <div className="rounded-xl border border-line bg-panel/80 px-4 py-3">
      <p className="text-xs uppercase tracking-wide text-muted">{label}</p>
      <p className={`mt-1 text-lg font-semibold text-text truncate ${mono ? "font-mono text-base" : ""}`}>
        {value}
      </p>
      {hint && (
        <p className={`mt-0.5 text-xs ${hintOk ? "text-ok" : "text-muted"}`}>{hint}</p>
      )}
    </div>
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
    } catch {
      /* ignore */
    }
  }, [authHeaders]);

  useEffect(() => {
    load();
  }, [load]);

  async function create(e: React.FormEvent) {
    e.preventDefault();
    if (form.title.trim().length < 2) {
      showMessage("Заголовок слишком короткий", "error");
      return;
    }
    setBusy(true);
    try {
      const res = await fetch("/api/admin/news", {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({ ...form, published: true }),
      });
      if (res.ok) {
        setForm({ title: "", tag: "Новость", summary: "" });
        showMessage("Новость добавлена", "success");
        await load();
      } else {
        const d = await res.json().catch(() => ({}));
        showMessage(d.error || "Ошибка", "error");
      }
    } catch {
      showMessage("Ошибка соединения", "error");
    } finally {
      setBusy(false);
    }
  }

  async function togglePublish(n: NewsRow) {
    const res = await fetch(`/api/admin/news/${n.id}`, {
      method: "PUT",
      headers: authHeaders(),
      body: JSON.stringify({ published: !n.published }),
    });
    if (res.ok) await load();
    else showMessage("Не удалось обновить", "error");
  }

  async function remove(id: number) {
    const res = await fetch(`/api/admin/news/${id}`, { method: "DELETE", headers: authHeaders() });
    if (res.ok) {
      showMessage("Удалено", "success");
      await load();
    } else showMessage("Не удалось удалить", "error");
  }

  return (
    <div className="space-y-5">
      <div>
        <h2 className="font-display text-lg font-bold tracking-tight">Новости</h2>
        <p className="text-muted text-sm mt-1">Появляются на главной. Сними публикацию, чтобы скрыть.</p>
      </div>

      <form onSubmit={create} className="rounded-xl border border-line bg-panel-2/40 p-4 grid gap-3 sm:grid-cols-[1fr_auto]">
        <div className="space-y-3">
          <input
            className="input"
            placeholder="Заголовок"
            value={form.title}
            onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
          />
          <div className="flex gap-3">
            <input
              className="input max-w-[10rem]"
              placeholder="Тег"
              value={form.tag}
              onChange={(e) => setForm((f) => ({ ...f, tag: e.target.value }))}
            />
            <input
              className="input flex-1"
              placeholder="Кратко"
              value={form.summary}
              onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))}
            />
          </div>
        </div>
        <button type="submit" disabled={busy} className="btn btn-primary self-start">
          {busy ? "..." : "Добавить"}
        </button>
      </form>

      <div className="space-y-2">
        {items.length === 0 ? (
          <p className="text-muted text-sm">Пока нет новостей.</p>
        ) : (
          items.map((n) => (
            <div key={n.id} className="flex items-center gap-3 rounded-xl border border-line px-3 py-3">
              <span className={`badge ${n.published ? "badge-accent" : ""}`}>{n.tag}</span>
              <div className="flex-1 min-w-0">
                <div className="text-text text-sm font-medium truncate">{n.title}</div>
                <div className="text-muted text-xs truncate">{n.summary}</div>
              </div>
              <button onClick={() => togglePublish(n)} className="btn btn-secondary !py-1.5 !px-3 text-xs">
                {n.published ? "Скрыть" : "Опубликовать"}
              </button>
              <button onClick={() => remove(n.id)} className="btn btn-danger !py-1.5 !px-3 text-xs">
                Удалить
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
