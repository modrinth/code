"use client";

import { Suspense, useState, useEffect, useCallback } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import CabinetShell, { Toast } from "@/components/layout/CabinetShell";
import { useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";
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

type Section = "users" | "catalog" | "news";

export default function AdminPage() {
  return (
    <Suspense fallback={<AdminLoadingFallback />}>
      <AdminPageInner />
    </Suspense>
  );
}

function AdminLoadingFallback() {
  const { locale } = useLocale();
  return (
    <div className="min-h-screen flex items-center justify-center">
      <p className="text-muted">{locale === "en_US" ? "Loading…" : "Загрузка…"}</p>
    </div>
  );
}

function AdminPageInner() {
  const { user, loading: authLoading } = useAuth({ requireAuth: true });
  const { locale } = useLocale();
  const en = locale === "en_US";
  const t = (a: string, b: string) => (en ? a : b);
  const router = useRouter();
  const searchParams = useSearchParams();
  const advanced = searchParams.get("advanced") === "1";

  const [users, setUsers] = useState<AdminUser[] | null>(null);
  const [search, setSearch] = useState("");
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [section, setSection] = useState<Section>("users");

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
    setToast({ text, type });
    window.setTimeout(() => setToast(null), 4000);
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
          } else setUsers([]);
        }
      } catch {
        if (!cancelled) setUsers([]);
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
      `Удалить ${u.nickname || u.email}? Причина (мин. 5 символов). Может сломать связку с плагином.`
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
        <p className="text-muted">{t("Loading…", "Загрузка…")}</p>
      </div>
    );
  }

  if (!isAdmin) {
    return (
      <>
        <Header />
        <main id="main-content" className="relative min-h-[calc(100vh-64px)]">
          <div className="relative z-10 mx-auto max-w-md px-4 py-24 text-center">
            <p className="text-xs uppercase tracking-[0.16em] text-accent/80">{t("Admin", "Админ")}</p>
            <h1 className="font-display mt-2 text-2xl font-bold tracking-tight">
              {t("Access denied", "Доступ запрещён")}
            </h1>
            <p className="mt-2 text-sm text-muted leading-relaxed">
              {t("Administrator role required.", "Нужны права администратора.")}
            </p>
            <button type="button" onClick={() => router.push("/profile")} className="btn btn-primary mt-6">
              {t("Back to account", "В личный кабинет")}
            </button>
          </div>
        </main>
        <Footer />
      </>
    );
  }

  return (
    <>
      <Header />
      <CabinetShell
        eyebrow="Control plane"
        title={t("Control panel", "Панель управления")}
        subtitle={t(
          "Accounts, launcher catalog, and home news.",
          "Аккаунты, каталог лаунчера и новости на главной.",
        )}
        nav={[
          {
            id: "users",
            label: t("Accounts", "Аккаунты"),
            hint: `${users?.length ?? "…"} ${t("users", "чел.")}`,
          },
          { id: "catalog", label: t("Catalog", "Каталог"), hint: t("Servers & packs", "Серверы и паки") },
          { id: "news", label: t("News", "Новости"), hint: t("Site home", "Главная сайта") },
        ]}
        activeId={section}
        onNav={(id) => setSection(id as Section)}
      >
        {section === "users" && (
          <UsersPane
            filtered={filtered}
            search={search}
            setSearch={setSearch}
            dataLoading={dataLoading}
            advanced={advanced}
            selfId={user?.id}
            changeRole={changeRole}
            toggleBan={toggleBan}
            deleteUser={deleteUser}
          />
        )}
        {section === "catalog" && (
          <CatalogAdmin authHeaders={authHeaders} showMessage={showMessage} />
        )}
        {section === "news" && (
          <NewsAdmin authHeaders={authHeaders} showMessage={showMessage} />
        )}
      </CabinetShell>
      {toast && <Toast text={toast.text} type={toast.type} />}
      <Footer />
    </>
  );
}

function UsersPane({
  filtered,
  search,
  setSearch,
  dataLoading,
  advanced,
  selfId,
  changeRole,
  toggleBan,
  deleteUser,
}: {
  filtered: AdminUser[];
  search: string;
  setSearch: (v: string) => void;
  dataLoading: boolean;
  advanced: boolean;
  selfId?: number;
  changeRole: (id: number, role: string) => void;
  toggleBan: (u: AdminUser) => void;
  deleteUser: (u: AdminUser) => void;
}) {
  return (
    <div className="space-y-5">
      <div>
        <h2 className="font-display text-lg font-bold tracking-tight">Аккаунты</h2>
        <p className="mt-1 text-sm text-muted">Роли для сайта и лаунчера. Бан/удаление — только advanced.</p>
      </div>

      <div className="field max-w-md">
        <label className="field-label" htmlFor="user-search">
          Поиск
        </label>
        <input
          id="user-search"
          className="input"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="ник или email"
          autoComplete="off"
        />
      </div>

      <div className="overflow-hidden rounded-[14px] border border-line">
        {dataLoading ? (
          <p className="py-12 text-center text-muted text-sm">Загрузка аккаунтов…</p>
        ) : filtered.length === 0 ? (
          <div className="empty-surface m-4">
            <h3>Никого не нашли</h3>
            <p>Сбрось поиск или проверь написание ника.</p>
          </div>
        ) : (
          <div>
            <div className="hidden sm:grid grid-cols-[minmax(0,1.1fr)_minmax(0,1.4fr)_auto] gap-4 border-b border-line bg-panel-2/50 px-4 py-2.5 text-xs font-medium uppercase tracking-wide text-muted">
              <span>Ник</span>
              <span>Email</span>
              <span className="text-right">Роль</span>
            </div>
            {filtered.map((u) => (
              <div key={u.id} className="admin-user-row">
                <div className="min-w-0">
                  <p className="font-medium text-text truncate">{u.nickname || "—"}</p>
                  {u.status === "banned" && <span className="badge badge-danger mt-1">бан</span>}
                </div>
                <p className="text-sm text-muted truncate">{u.email}</p>
                <div className="flex flex-wrap items-center gap-2 sm:justify-end">
                  <select
                    className="select !py-2 !min-h-10 max-w-[9.5rem]"
                    value={ROLES.includes(u.role as (typeof ROLES)[number]) ? u.role : "user"}
                    onChange={(e) => changeRole(u.id, e.target.value)}
                    disabled={u.id === selfId}
                    title={u.id === selfId ? "Нельзя изменить свою роль" : "Сменить роль"}
                    aria-label={`Роль ${u.nickname || u.email}`}
                  >
                    {ROLES.map((r) => (
                      <option key={r} value={r}>
                        {r}
                      </option>
                    ))}
                  </select>
                  {advanced && u.id !== selfId && (
                    <>
                      <button
                        type="button"
                        onClick={() => toggleBan(u)}
                        className="btn btn-secondary btn-sm"
                      >
                        {u.status === "banned" ? "Разбан" : "Бан"}
                      </button>
                      <button type="button" onClick={() => deleteUser(u)} className="btn btn-danger btn-sm">
                        Удалить
                      </button>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {!advanced && (
        <p className="text-xs text-muted">
          Опасные действия:{" "}
          <a className="link-accent" href="/admin?advanced=1">
            /admin?advanced=1
          </a>
        </p>
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
    void load();
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
    <div className="space-y-6">
      <div>
        <h2 className="font-display text-lg font-bold tracking-tight">Новости</h2>
        <p className="mt-1 text-sm text-muted">Блок на главной. Сними публикацию, чтобы скрыть.</p>
      </div>

      <form
        onSubmit={create}
        className="section-callout space-y-3"
      >
        <div className="field">
          <label className="field-label" htmlFor="news-title">
            Заголовок
          </label>
          <input
            id="news-title"
            className="input"
            value={form.title}
            onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
            placeholder="Что нового"
          />
        </div>
        <div className="grid gap-3 sm:grid-cols-[8rem_1fr]">
          <div className="field">
            <label className="field-label" htmlFor="news-tag">
              Тег
            </label>
            <input
              id="news-tag"
              className="input"
              value={form.tag}
              onChange={(e) => setForm((f) => ({ ...f, tag: e.target.value }))}
            />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="news-summary">
              Кратко
            </label>
            <input
              id="news-summary"
              className="input"
              value={form.summary}
              onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))}
              placeholder="Одна фраза"
            />
          </div>
        </div>
        <div className="flex justify-end">
          <button type="submit" disabled={busy} className="btn btn-primary">
            {busy ? "…" : "Добавить"}
          </button>
        </div>
      </form>

      <div className="space-y-2">
        {items.length === 0 ? (
          <div className="empty-surface">
            <h3>Пока пусто</h3>
            <p>Добавь первую новость — она появится на главной.</p>
          </div>
        ) : (
          items.map((n) => (
            <div
              key={n.id}
              className="list-row"
            >
              <span className={`badge ${n.published ? "badge-accent" : ""}`}>{n.tag}</span>
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-text truncate">{n.title}</p>
                <p className="text-xs text-muted truncate">{n.summary}</p>
              </div>
              <button type="button" onClick={() => togglePublish(n)} className="btn btn-secondary btn-sm">
                {n.published ? "Скрыть" : "Опубликовать"}
              </button>
              <button type="button" onClick={() => remove(n.id)} className="btn btn-danger btn-sm">
                Удалить
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
