"use client";

import { useState, useEffect, useSyncExternalStore } from "react";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";

const authHeader = () => ({ Authorization: `Bearer ${localStorage.getItem("auth_token")}` });

function formatDate(dateStr?: string): string {
  if (!dateStr) return "—";
  return new Date(dateStr).toLocaleDateString("ru-RU", { day: "2-digit", month: "long", year: "numeric" });
}

function roleBadge(role?: string) {
  if (role === "admin") return <span className="badge badge-danger">ADMIN</span>;
  if (role === "moderator") return <span className="badge badge-accent">MOD</span>;
  return <span className="badge">Игрок</span>;
}

type Tab = "overview" | "settings";

function useStableNow(intervalMs = 60_000) {
  return useSyncExternalStore(
    (onStoreChange) => {
      const id = window.setInterval(onStoreChange, intervalMs);
      return () => window.clearInterval(id);
    },
    () => Math.floor(Date.now() / intervalMs) * intervalMs,
    () => 0
  );
}

export default function ProfilePage() {
  const { user, loading, logout } = useAuth({ requireAuth: true });
  const [tab, setTab] = useState<Tab>("settings");

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-muted">Загрузка профиля...</p>
      </div>
    );
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: "overview", label: "Обзор" },
    { id: "settings", label: "Настройки" },
  ];

  const initial = (user?.nickname || user?.email || "?").slice(0, 1).toUpperCase();

  return (
    <>
      <Header />
      <main id="main-content" className="relative min-h-[calc(100vh-64px)] overflow-hidden">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 opacity-80"
          style={{
            background:
              "radial-gradient(ellipse 70% 50% at 15% 0%, rgba(0,229,255,0.12), transparent 55%), radial-gradient(ellipse 50% 40% at 90% 20%, rgba(139,92,246,0.08), transparent 50%)",
          }}
        />
        <div className="relative z-10 mx-auto max-w-3xl px-4 py-10 sm:px-6">
          <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
            <div className="flex items-center gap-4 min-w-0">
              <div className="grid h-14 w-14 shrink-0 place-items-center rounded-2xl border border-line bg-panel text-xl font-bold text-accent">
                {user?.avatar_url ? (
                  // eslint-disable-next-line @next/next/no-img-element
                  <img src={user.avatar_url} alt="" className="h-full w-full rounded-2xl object-cover" />
                ) : (
                  initial
                )}
              </div>
              <div className="min-w-0">
                <p className="text-xs uppercase tracking-[0.14em] text-muted">Личный кабинет</p>
                <h1 className="font-display truncate text-3xl font-bold tracking-tight text-text">
                  {user?.nickname || "Игрок"}
                </h1>
                <div className="mt-1.5 flex flex-wrap items-center gap-2">
                  {roleBadge(user?.role)}
                  <span className={`badge ${user?.status === "banned" ? "badge-danger" : "badge-ok"}`}>
                    {user?.status === "banned" ? "Заблокирован" : "Активен"}
                  </span>
                </div>
              </div>
            </div>
            <button onClick={logout} className="btn btn-danger" id="profile-logout-btn">
              Выйти
            </button>
          </div>

          <div className="overflow-hidden rounded-2xl border border-line bg-panel/90 backdrop-blur-sm">
            <div className="border-b border-line px-4">
              <nav className="flex gap-1" aria-label="Разделы кабинета">
                {tabs.map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    onClick={() => setTab(t.id)}
                    className={`relative px-4 py-3.5 text-sm font-medium transition-colors ${
                      tab === t.id ? "text-accent" : "text-muted hover:text-text"
                    }`}
                  >
                    {t.label}
                    {tab === t.id && (
                      <span className="absolute inset-x-3 bottom-0 h-0.5 rounded-full bg-accent" />
                    )}
                  </button>
                ))}
              </nav>
            </div>
            <div className="p-6 sm:p-8">
              {tab === "overview" && <OverviewTab user={user} />}
              {tab === "settings" && <SettingsTab user={user} />}
            </div>
          </div>

          <p className="mt-6 text-center text-xs text-muted">
            Скины и «Внешний вид» временно скрыты — выбор скина будет в лаунчере.
          </p>
        </div>
      </main>
      <Footer />
    </>
  );
}

function OverviewTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const now = useStableNow();
  if (!user) return null;
  const daysWithUs = user.created_at
    ? Math.max(0, Math.floor((now - new Date(user.created_at).getTime()) / 86400000))
    : 0;
  const rows = [
    { label: "Логин", value: user.nickname || "—" },
    { label: "Имя", value: user.first_name || "—" },
    { label: "Email", value: user.email || "—" },
    { label: "Discord", value: user.discord || "—" },
    { label: "Роль", value: user.role || "user" },
    { label: "Регистрация", value: formatDate(user.created_at) },
    { label: "Дней с нами", value: String(daysWithUs) },
  ];
  return (
    <div className="space-y-6">
      <div>
        <h2 className="font-display text-xl font-bold tracking-tight">Обзор аккаунта</h2>
        <p className="mt-1 text-sm text-muted">Тот же логин потом в лаунчере Owyx.</p>
      </div>
      <dl className="divide-y divide-line rounded-xl border border-line bg-panel-2/40">
        {rows.map((r) => (
          <div key={r.label} className="flex items-center justify-between gap-4 px-4 py-3">
            <dt className="text-sm text-muted">{r.label}</dt>
            <dd className="truncate text-sm font-medium text-text">{r.value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

function SettingsTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  return (
    <div className="space-y-10 max-w-xl">
      <div>
        <h2 className="font-display text-xl font-bold tracking-tight">Настройки</h2>
        <p className="mt-1 text-sm text-muted">Почта, ник, профиль и пароль.</p>
      </div>
      <EmailSection currentEmail={user?.email} />
      <NicknameSection currentNick={user?.nickname} />
      <PasswordAndFieldsSection user={user} />
      <NotificationsSection />
    </div>
  );
}

function EmailSection({ currentEmail }: { currentEmail?: string }) {
  const [overrideEmail, setOverrideEmail] = useState<string | null>(null);
  const email = overrideEmail ?? currentEmail ?? "";
  const [open, setOpen] = useState(false);

  return (
    <section>
      <h3 className="text-base font-semibold tracking-tight mb-1">Почта</h3>
      <p className="text-sm text-muted mb-3">Вход и восстановление доступа.</p>
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-panel-2/50 px-4 py-3">
        <span className="font-medium break-all text-text">{email || "—"}</span>
        <button type="button" className="btn btn-secondary" onClick={() => setOpen(true)}>
          Сменить почту
        </button>
      </div>
      {open && (
        <EmailChangeModal
          onClose={() => setOpen(false)}
          onDone={(newEmail) => {
            setOverrideEmail(newEmail);
            setOpen(false);
          }}
        />
      )}
    </section>
  );
}

function EmailChangeModal({ onClose, onDone }: { onClose: () => void; onDone: (email: string) => void }) {
  const [step, setStep] = useState<"email" | "code">("email");
  const [newEmail, setNewEmail] = useState("");
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  async function requestCode(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setMsg(null);
    try {
      const res = await fetch("/api/profile/email/request", {
        method: "POST",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ email: newEmail }),
      });
      const data = await res.json();
      if (res.ok) {
        setStep("code");
        const shown =
          process.env.NODE_ENV === "development" && data.devCode
            ? `Код (dev): ${data.devCode}`
            : "Код отправлен на новую почту.";
        setMsg({ text: shown, type: "success" });
      } else setMsg({ text: data.error || "Не удалось отправить код", type: "error" });
    } catch {
      setMsg({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  async function confirmCode(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setMsg(null);
    try {
      const res = await fetch("/api/profile/email/confirm", {
        method: "POST",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ code }),
      });
      const data = await res.json();
      if (res.ok) onDone(data.email || newEmail);
      else setMsg({ text: data.error || "Неверный код", type: "error" });
    } catch {
      setMsg({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/60" onClick={onClose}>
      <div className="panel w-full max-w-sm p-6" onClick={(e) => e.stopPropagation()}>
        <h4 className="text-lg font-bold mb-1">Смена почты</h4>
        <p className="text-sm text-muted mb-4">
          {step === "email" ? "Введите новую почту — на неё придёт код." : `Введите код, отправленный на ${newEmail}.`}
        </p>
        {msg && (
          <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>
        )}
        {step === "email" ? (
          <form onSubmit={requestCode} className="space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="new-email">
                Новая почта
              </label>
              <input
                id="new-email"
                type="email"
                required
                className="input"
                value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)}
                placeholder="new@owyx.site"
              />
            </div>
            <div className="flex gap-3">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "Отправка..." : "Отправить код"}
              </button>
              <button type="button" onClick={onClose} className="btn btn-ghost">
                Отмена
              </button>
            </div>
          </form>
        ) : (
          <form onSubmit={confirmCode} className="space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="email-code">
                Код из письма
              </label>
              <input
                id="email-code"
                inputMode="numeric"
                maxLength={6}
                required
                className="input tracking-[0.4em]"
                value={code}
                onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                placeholder="123456"
              />
            </div>
            <div className="flex gap-3">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "Проверка..." : "Подтвердить"}
              </button>
              <button type="button" onClick={() => setStep("email")} className="btn btn-ghost">
                Назад
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

function NicknameSection({ currentNick }: { currentNick?: string }) {
  const [nick, setNick] = useState(currentNick || "");
  const [changedAt, setChangedAt] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await fetch("/api/profile", { headers: authHeader() });
        if (res.ok) {
          const data = await res.json();
          if (data.minecraft_nick) setNick(data.minecraft_nick);
          setChangedAt(data.nickname_changed_at ?? null);
        }
      } catch {
        /* ignore */
      }
    })();
  }, []);

  const now = useStableNow();
  const nextAllowed = changedAt ? new Date(changedAt).getTime() + 30 * 86400000 : 0;
  const onCooldown = nextAllowed > now;
  const daysLeft = onCooldown ? Math.ceil((nextAllowed - now) / 86400000) : 0;

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setMsg(null);
    try {
      const res = await fetch("/api/profile/nickname", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ nickname: nick }),
      });
      const data = await res.json();
      if (res.ok) {
        setMsg({ text: "Ник изменён. Следующая смена — через 30 дней.", type: "success" });
        setChangedAt(data.nickname_changed_at ?? new Date().toISOString());
      } else setMsg({ text: data.error || "Не удалось изменить ник", type: "error" });
    } catch {
      setMsg({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="border-t border-line pt-8">
      <h3 className="text-base font-semibold tracking-tight mb-1">Никнейм</h3>
      <p className="text-sm text-muted mb-3">Менять можно не чаще одного раза в 30 дней.</p>
      {msg && (
        <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>
      )}
      <form onSubmit={save} className="space-y-4">
        <div className="field">
          <label className="field-label" htmlFor="acc-nick">
            Ник
          </label>
          <input
            id="acc-nick"
            className="input"
            value={nick}
            onChange={(e) => setNick(e.target.value)}
            maxLength={32}
            placeholder="YourNickname"
            disabled={onCooldown}
          />
          {onCooldown && <span className="field-hint">Следующая смена доступна через {daysLeft} дн.</span>}
        </div>
        <button type="submit" className="btn btn-primary" disabled={busy || onCooldown}>
          {busy ? "Сохранение..." : "Сменить ник"}
        </button>
      </form>
    </section>
  );
}

function PasswordAndFieldsSection({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const [form, setForm] = useState({
    first_name: user?.first_name || "",
    discord_username: user?.discord || "",
    current_password: "",
    new_password: "",
  });
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await fetch("/api/profile", { headers: authHeader() });
        if (res.ok) {
          const data = await res.json();
          setForm((f) => ({
            ...f,
            first_name: data.first_name || "",
            discord_username: data.discord || "",
          }));
        }
      } catch {
        /* ignore */
      }
    })();
  }, []);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setMsg(null);
    try {
      const body: Record<string, unknown> = {
        first_name: form.first_name,
        discord_username: form.discord_username,
      };
      if (form.new_password) {
        body.current_password = form.current_password;
        body.new_password = form.new_password;
      }
      const res = await fetch("/api/profile", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setMsg({ text: data.message || "Сохранено", type: "success" });
        setForm((f) => ({ ...f, current_password: "", new_password: "" }));
      } else setMsg({ text: data.error || "Ошибка сохранения", type: "error" });
    } catch {
      setMsg({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="border-t border-line pt-8">
      <h3 className="text-base font-semibold tracking-tight mb-4">Профиль и безопасность</h3>
      {msg && (
        <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>
      )}
      <form onSubmit={save} className="space-y-4">
        <div className="field">
          <label className="field-label" htmlFor="acc-name">
            Имя
          </label>
          <input
            id="acc-name"
            className="input"
            value={form.first_name}
            onChange={(e) => setForm((f) => ({ ...f, first_name: e.target.value }))}
          />
        </div>
        <div className="field">
          <label className="field-label" htmlFor="acc-discord">
            Discord
          </label>
          <input
            id="acc-discord"
            className="input"
            value={form.discord_username}
            onChange={(e) => setForm((f) => ({ ...f, discord_username: e.target.value }))}
          />
        </div>
        <div className="rounded-xl border border-line bg-panel-2/40 p-4 space-y-3">
          <p className="text-sm font-medium text-text">Смена пароля (необязательно)</p>
          <div className="field">
            <label className="field-label" htmlFor="acc-cur-pass">
              Текущий пароль
            </label>
            <input
              id="acc-cur-pass"
              type="password"
              className="input"
              autoComplete="current-password"
              value={form.current_password}
              onChange={(e) => setForm((f) => ({ ...f, current_password: e.target.value }))}
            />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="acc-new-pass">
              Новый пароль
            </label>
            <input
              id="acc-new-pass"
              type="password"
              className="input"
              autoComplete="new-password"
              value={form.new_password}
              onChange={(e) => setForm((f) => ({ ...f, new_password: e.target.value }))}
            />
          </div>
        </div>
        <button type="submit" className="btn btn-primary" disabled={busy}>
          {busy ? "Сохранение..." : "Сохранить"}
        </button>
      </form>
    </section>
  );
}

function NotificationsSection() {
  return (
    <section className="border-t border-line pt-8">
      <h3 className="text-base font-semibold tracking-tight mb-1">Уведомления</h3>
      <p className="text-sm text-muted">Скоро — письма о важных событиях аккаунта.</p>
    </section>
  );
}
