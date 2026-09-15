"use client";

import { useState, useEffect, useSyncExternalStore } from "react";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";
import SkinViewer from "@/components/profile/SkinViewer";

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

type Tab = "info" | "appearance" | "account";

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

function useSkin() {
  const { user, refreshAuth } = useAuth();
  const [override, setOverride] = useState<{
    skin_url: string | null;
    skin_model: "classic" | "slim";
  } | null>(null);
  const skin = override ?? {
    skin_url: user?.skin_url ?? null,
    skin_model: user?.skin_model === "slim" ? "slim" : "classic",
  };
  async function reload() {
    await refreshAuth();
    setOverride(null);
  }
  return { skin, reload };
}

export default function ProfilePage() {
  const { user, loading, logout } = useAuth({ requireAuth: true });
  const [tab, setTab] = useState<Tab>("info");

  if (loading) {
    return <div className="min-h-screen flex items-center justify-center"><p className="text-muted">Загрузка профиля...</p></div>;
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: "info", label: "Профиль" },
    { id: "appearance", label: "Внешний вид" },
    { id: "account", label: "Аккаунт" },
  ];

  return (
    <>
      <Header />
      <main id="main-content" className="min-h-[calc(100vh-64px)] py-8 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h1 className="font-display text-3xl font-bold tracking-tight">Личный кабинет</h1>
              <div className="flex items-center gap-2 mt-2">
                <span className="text-accent font-medium">{user?.nickname}</span>
                {roleBadge(user?.role)}
              </div>
            </div>
            <button onClick={logout} className="btn btn-danger" id="profile-logout-btn">Выйти</button>
          </div>

          <div className="panel overflow-hidden">
            <div className="border-b border-line px-4 overflow-x-auto">
              <nav className="flex gap-6">
                {tabs.map((t) => (
                  <button
                    key={t.id}
                    onClick={() => setTab(t.id)}
                    className={`py-4 border-b-2 text-sm font-medium whitespace-nowrap transition-colors ${
                      tab === t.id ? "border-accent text-accent" : "border-transparent text-muted hover:text-accent"
                    }`}
                  >
                    {t.label}
                  </button>
                ))}
              </nav>
            </div>
            <div className="p-6">
              {tab === "info" && <InfoTab user={user} />}
              {tab === "appearance" && <AppearanceTab nickname={user?.nickname} />}
              {tab === "account" && <AccountTab user={user} />}
            </div>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

// ==== Профиль — скин игрока (3D/2D) + краткие данные ====
function InfoTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const { skin } = useSkin();
  const now = useStableNow();
  if (!user) return null;
  const daysWithUs = user.created_at
    ? Math.max(0, Math.floor((now - new Date(user.created_at).getTime()) / 86400000))
    : 0;
  const rows = [
    { label: "Minecraft ник", value: user.nickname || "—" },
    { label: "Имя", value: user.first_name || "—" },
    { label: "Email", value: user.email || "—" },
    { label: "Discord", value: user.discord || "—" },
    { label: "Роль", value: user.role || "user" },
    { label: "Регистрация", value: formatDate(user.created_at) },
    { label: "Дней с нами", value: String(daysWithUs) },
  ];
  return (
    <div className="grid gap-8 md:grid-cols-[auto_1fr] items-start">
      <div className="flex flex-col items-center gap-3">
        <SkinViewer skinUrl={skin.skin_url} model={skin.skin_model} initialMode="2d" />
        <span className={`badge ${user.status === "banned" ? "badge-danger" : "badge-ok"}`}>
          {user.status === "banned" ? "Заблокирован" : "Активен"}
        </span>
      </div>
      <div className="divide-y divide-line">
        {rows.map((r) => (
          <div key={r.label} className="flex justify-between items-center py-2.5">
            <span className="text-sm text-muted">{r.label}</span>
            <span className="text-sm text-text font-medium">{r.value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ==== Внешний вид — скин (3D/2D), модель, upload, cape placeholder ====
function AppearanceTab({ nickname }: { nickname?: string }) {
  const { skin, reload } = useSkin();
  const [modelChoice, setModelChoice] = useState<"classic" | "slim" | null>(null);
  const model = modelChoice ?? skin.skin_model;
  const [file, setFile] = useState<File | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  async function upload(e: React.FormEvent) {
    e.preventDefault();
    setMsg(null);
    if (!file) { setMsg({ text: "Сначала выбери PNG-файл скина (64×64).", type: "error" }); return; }
    setBusy(true);
    try {
      const fd = new FormData();
      fd.append("skin", file);
      fd.append("model", model);
      const res = await fetch("/api/profile/skin", { method: "PUT", headers: authHeader(), body: fd });
      const data = await res.json();
      if (res.ok) { setMsg({ text: "Скин обновлён. Лаунчер подхватит его при входе.", type: "success" }); setFile(null); await reload(); }
      else setMsg({ text: data.error || "Не удалось загрузить скин", type: "error" });
    } catch { setMsg({ text: "Не удалось связаться с сервером. Попробуй позже.", type: "error" }); }
    finally { setBusy(false); }
  }

  async function removeSkin() {
    setBusy(true); setMsg(null);
    try {
      const res = await fetch("/api/profile/skin", { method: "DELETE", headers: authHeader() });
      if (res.ok) { setMsg({ text: "Скин удалён — вернётся стандартный.", type: "success" }); await reload(); }
      else setMsg({ text: "Не удалось удалить скин", type: "error" });
    } catch { setMsg({ text: "Не удалось связаться с сервером.", type: "error" }); }
    finally { setBusy(false); }
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold tracking-tight mb-1">Внешний вид</h3>
        <p className="text-sm text-muted">Загрузи свой скин Minecraft (PNG 64×64). Лаунчер подхватит его при входе аккаунтом Owyx.</p>
      </div>

      {msg && <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"}`}>{msg.text}</div>}

      <div className="flex flex-col sm:flex-row items-start gap-8">
        <div className="flex flex-col items-center gap-2">
          <SkinViewer skinUrl={skin.skin_url} model={model} initialMode="3d" />
          <span className="text-xs text-muted">{model === "slim" ? "Alex (slim)" : "Steve (classic)"}{nickname ? ` · ${nickname}` : ""}</span>
        </div>

        <form onSubmit={upload} className="flex-1 min-w-[260px] space-y-5">
          <div className="field">
            <label className="field-label" htmlFor="skin-file">PNG скина (64×64 или 64×32)</label>
            <input id="skin-file" type="file" accept="image/png"
              onChange={(e) => setFile(e.target.files?.[0] ?? null)}
              className="block w-full text-sm text-muted file:mr-3 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-accent file:text-[#041018] file:font-semibold hover:file:opacity-90 cursor-pointer" />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="skin-model">Модель тела</label>
            <select id="skin-model" className="select" value={model} onChange={(e) => setModelChoice(e.target.value as "classic" | "slim")}>
              <option value="classic">Steve (classic, 4px руки)</option>
              <option value="slim">Alex (slim, 3px руки)</option>
            </select>
          </div>
          <div className="flex gap-3 pt-1">
            <button type="submit" disabled={busy} className="btn btn-primary">{busy ? "Загрузка..." : "Загрузить скин"}</button>
            {skin.skin_url && <button type="button" onClick={removeSkin} disabled={busy} className="btn btn-secondary">Сбросить</button>}
          </div>
        </form>
      </div>

      <div className="panel p-4 flex items-center justify-between">
        <span className="text-sm text-muted">Плащ (cape)</span>
        <span className="badge">скоро</span>
      </div>
    </div>
  );
}

// ==== Аккаунт — email(смена), ник, пароль, уведомления ====
function AccountTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  return (
    <div className="space-y-10 max-w-xl">
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
      <h3 className="text-xl font-bold tracking-tight mb-1">Почта</h3>
      <p className="text-sm text-muted mb-4">Используется для входа и восстановления доступа.</p>
      <div className="panel p-4 flex flex-wrap items-center justify-between gap-3">
        <span className="text-text font-medium break-all">{email || "—"}</span>
        <button type="button" className="btn btn-secondary" onClick={() => setOpen(true)}>Сменить почту</button>
      </div>
      {open && (
        <EmailChangeModal
          onClose={() => setOpen(false)}
          onDone={(newEmail) => { setOverrideEmail(newEmail); setOpen(false); }}
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
    setBusy(true); setMsg(null);
    try {
      const res = await fetch("/api/profile/email/request", {
        method: "POST", headers: { ...authHeader(), "Content-Type": "application/json" },
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
    } catch { setMsg({ text: "Не удалось связаться с сервером.", type: "error" }); }
    finally { setBusy(false); }
  }

  async function confirmCode(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true); setMsg(null);
    try {
      const res = await fetch("/api/profile/email/confirm", {
        method: "POST", headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ code }),
      });
      const data = await res.json();
      if (res.ok) onDone(data.email || newEmail);
      else setMsg({ text: data.error || "Неверный код", type: "error" });
    } catch { setMsg({ text: "Не удалось связаться с сервером.", type: "error" }); }
    finally { setBusy(false); }
  }

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/60" onClick={onClose}>
      <div className="panel w-full max-w-sm p-6" onClick={(e) => e.stopPropagation()}>
        <h4 className="text-lg font-bold mb-1">Смена почты</h4>
        <p className="text-sm text-muted mb-4">
          {step === "email" ? "Введите новую почту — на неё придёт код." : `Введите код, отправленный на ${newEmail}.`}
        </p>
        {msg && <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>}
        {step === "email" ? (
          <form onSubmit={requestCode} className="space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="new-email">Новая почта</label>
              <input id="new-email" type="email" required className="input" value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)} placeholder="new@owyx.site" />
            </div>
            <div className="flex gap-3">
              <button type="submit" disabled={busy} className="btn btn-primary">{busy ? "Отправка..." : "Отправить код"}</button>
              <button type="button" onClick={onClose} className="btn btn-ghost">Отмена</button>
            </div>
          </form>
        ) : (
          <form onSubmit={confirmCode} className="space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="email-code">Код из письма</label>
              <input id="email-code" inputMode="numeric" maxLength={6} required className="input tracking-[0.4em]"
                value={code} onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))} placeholder="123456" />
            </div>
            <div className="flex gap-3">
              <button type="submit" disabled={busy} className="btn btn-primary">{busy ? "Проверка..." : "Подтвердить"}</button>
              <button type="button" onClick={() => setStep("email")} className="btn btn-ghost">Назад</button>
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
      } catch { /* ignore */ }
    })();
  }, []);

  const now = useStableNow();
  const nextAllowed = changedAt ? new Date(changedAt).getTime() + 30 * 86400000 : 0;
  const onCooldown = nextAllowed > now;
  const daysLeft = onCooldown ? Math.ceil((nextAllowed - now) / 86400000) : 0;

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true); setMsg(null);
    try {
      const res = await fetch("/api/profile/nickname", {
        method: "PUT", headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ nickname: nick }),
      });
      const data = await res.json();
      if (res.ok) { setMsg({ text: "Ник изменён. Следующая смена — через 30 дней.", type: "success" }); setChangedAt(data.nickname_changed_at ?? new Date().toISOString()); }
      else setMsg({ text: data.error || "Не удалось изменить ник", type: "error" });
    } catch { setMsg({ text: "Не удалось связаться с сервером.", type: "error" }); }
    finally { setBusy(false); }
  }

  return (
    <section className="border-t border-line pt-8">
      <h3 className="text-xl font-bold tracking-tight mb-1">Никнейм</h3>
      <p className="text-sm text-muted mb-4">Ник можно менять не чаще одного раза в 30 дней.</p>
      {msg && <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>}
      <form onSubmit={save} className="space-y-4">
        <div className="field">
          <label className="field-label" htmlFor="acc-nick">Ник</label>
          <input id="acc-nick" className="input" value={nick} onChange={(e) => setNick(e.target.value)} maxLength={32} placeholder="YourNickname" disabled={onCooldown} />
          {onCooldown && <span className="field-hint">Следующая смена доступна через {daysLeft} дн.</span>}
        </div>
        <button type="submit" className="btn btn-primary" disabled={busy || onCooldown}>{busy ? "Сохранение..." : "Сменить ник"}</button>
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
          setForm((f) => ({ ...f, first_name: data.first_name || "", discord_username: data.discord || "" }));
        }
      } catch { /* ignore */ }
    })();
  }, []);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true); setMsg(null);
    try {
      const body: Record<string, unknown> = { first_name: form.first_name, discord_username: form.discord_username };
      if (form.new_password) { body.current_password = form.current_password; body.new_password = form.new_password; }
      const res = await fetch("/api/profile", {
        method: "PUT", headers: { ...authHeader(), "Content-Type": "application/json" }, body: JSON.stringify(body),
      });
      const data = await res.json();
      if (res.ok && data.success) { setMsg({ text: data.message || "Сохранено", type: "success" }); setForm((f) => ({ ...f, current_password: "", new_password: "" })); }
      else setMsg({ text: data.error || "Ошибка сохранения", type: "error" });
    } catch { setMsg({ text: "Не удалось связаться с сервером.", type: "error" }); }
    finally { setBusy(false); }
  }

  return (
    <section className="border-t border-line pt-8">
      <h3 className="text-xl font-bold tracking-tight mb-4">Профиль и безопасность</h3>
      {msg && <div className={`form-msg ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"} mb-4`}>{msg.text}</div>}
      <form onSubmit={save} className="space-y-4">
        <div className="field">
          <label className="field-label" htmlFor="acc-name">Имя</label>
          <input id="acc-name" className="input" value={form.first_name} onChange={(e) => setForm((f) => ({ ...f, first_name: e.target.value }))} />
        </div>
        <div className="field">
          <label className="field-label" htmlFor="acc-discord">Discord</label>
          <input id="acc-discord" className="input" value={form.discord_username} onChange={(e) => setForm((f) => ({ ...f, discord_username: e.target.value }))} />
        </div>
        <div className="border-t border-line pt-5 mt-2 space-y-4">
          <p className="field-label">Смена пароля <span className="text-muted font-normal">(необязательно)</span></p>
          <div className="field">
            <label className="field-label" htmlFor="acc-cur-pass">Текущий пароль</label>
            <input id="acc-cur-pass" type="password" className="input" autoComplete="current-password"
              value={form.current_password} onChange={(e) => setForm((f) => ({ ...f, current_password: e.target.value }))} />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="acc-new-pass">Новый пароль</label>
            <input id="acc-new-pass" type="password" minLength={8} className="input" autoComplete="new-password"
              value={form.new_password} onChange={(e) => setForm((f) => ({ ...f, new_password: e.target.value }))} />
          </div>
        </div>
        <button type="submit" className="btn btn-primary" disabled={busy}>{busy ? "Сохранение..." : "Сохранить"}</button>
      </form>
    </section>
  );
}

function NotificationsSection() {
  return (
    <section className="border-t border-line pt-8">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-bold tracking-tight">Уведомления</h3>
          <p className="text-sm text-muted mt-1">Email-уведомления о входах и новостях сервера.</p>
        </div>
        <span className="badge">скоро</span>
      </div>
    </section>
  );
}
