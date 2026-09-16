"use client";

import { useState, useEffect, useSyncExternalStore } from "react";
import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import CabinetShell, {
  SettingsSection,
  SettingsRow,
  Toast,
} from "@/components/layout/CabinetShell";
import AvatarCropModal from "@/components/profile/AvatarCropModal";
import { useAuth } from "@/hooks/useAuth";
import { resolveSiteAvatarUrl } from "@/lib/avatar";

const authHeader = () => ({ Authorization: `Bearer ${localStorage.getItem("auth_token")}` });

function formatDate(dateStr?: string): string {
  if (!dateStr) return "—";
  return new Date(dateStr).toLocaleDateString("ru-RU", {
    day: "2-digit",
    month: "long",
    year: "numeric",
  });
}

function roleLabel(role?: string) {
  if (role === "admin") return { text: "Админ", cls: "badge-danger" };
  if (role === "moderator") return { text: "Модератор", cls: "badge-accent" };
  if (role === "helper") return { text: "Хелпер", cls: "badge-accent" };
  return { text: "Игрок", cls: "" };
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
  const [tab, setTab] = useState<Tab>("overview");

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-muted">Загрузка…</p>
      </div>
    );
  }

  const role = roleLabel(user?.role);
  const avatarSrc = resolveSiteAvatarUrl(user?.avatar_url);

  return (
    <>
      <Header />
      <CabinetShell
        eyebrow="Аккаунт Owyx"
        title={user?.nickname || "Игрок"}
        subtitle="Тот же логин — на сайте и в лаунчере."
        actions={
          <>
            <Link href="/download" className="btn btn-primary">
              Скачать лаунчер
            </Link>
            <button type="button" onClick={logout} className="btn btn-ghost" id="profile-logout-btn">
              Выйти
            </button>
          </>
        }
        nav={[
          { id: "overview", label: "Обзор", hint: "Кто вы в Owyx" },
          { id: "settings", label: "Настройки", hint: "Почта, ник, пароль" },
        ]}
        activeId={tab}
        onNav={(id) => setTab(id as Tab)}
        footerNote="Скины на сайте скрыты — выбор скина будет в лаунчере."
      >
        {tab === "overview" && <OverviewPane user={user} role={role} avatarSrc={avatarSrc} />}
        {tab === "settings" && <SettingsPane user={user} />}
      </CabinetShell>
      <Footer />
    </>
  );
}

function OverviewPane({
  user,
  role,
  avatarSrc,
}: {
  user: ReturnType<typeof useAuth>["user"];
  role: { text: string; cls: string };
  avatarSrc: string;
}) {
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
    { label: "Регистрация", value: formatDate(user.created_at) },
    { label: "Дней с нами", value: String(daysWithUs) },
  ];

  return (
    <div className="space-y-7">
      <div className="flex flex-wrap items-center gap-4">
        <div className="grid h-16 w-16 shrink-0 place-items-center overflow-hidden rounded-2xl border border-line bg-panel-2">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={avatarSrc} alt="" className="h-full w-full object-cover" />
        </div>
        <div className="min-w-0">
          <p className="font-display text-xl font-bold tracking-tight truncate">{user.nickname}</p>
          <div className="mt-2 flex flex-wrap gap-2">
            <span className={`badge ${role.cls}`}>{role.text}</span>
            <span className={`badge ${user.status === "banned" ? "badge-danger" : "badge-ok"}`}>
              {user.status === "banned" ? "Заблокирован" : "Активен"}
            </span>
          </div>
        </div>
      </div>

      <div className="cabinet-kv">
        {rows.map((r) => (
          <div key={r.label} className="cabinet-kv-row">
            <span className="text-sm text-muted shrink-0">{r.label}</span>
            <span className="text-sm font-medium text-text truncate text-right">{r.value}</span>
          </div>
        ))}
      </div>

      <div className="section-callout">
        <p className="text-sm font-medium text-text">Дальше</p>
        <p className="mt-1 text-sm text-muted leading-relaxed">
          Скачай лаунчер и войди тем же логином — сборки и Play подтянутся с сайта.
        </p>
        <Link href="/download" className="btn btn-primary mt-4">
          Перейти к загрузке
        </Link>
      </div>
    </div>
  );
}

function SettingsPane({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  return (
    <div className="grid grid-cols-1 gap-5 lg:grid-cols-2 lg:gap-6">
      <div className="flex flex-col gap-5">
        <AvatarSection currentUrl={user?.avatar_url} />
        <ProfileInfoSection user={user} />
      </div>
      <div className="flex flex-col gap-5">
        <EmailSection currentEmail={user?.email} />
        <NicknameSection currentNick={user?.nickname} />
      </div>
      <div className="lg:col-span-2">
        <PasswordSection />
      </div>
    </div>
  );
}

function AvatarSection({ currentUrl }: { currentUrl?: string | null }) {
  const [preview, setPreview] = useState(resolveSiteAvatarUrl(currentUrl));
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [cropFile, setCropFile] = useState<File | null>(null);

  async function uploadWithCrop(file: File, cropData: import("@/components/profile/AvatarCropModal").AvatarCropData) {
    setBusy(true);
    setToast(null);
    try {
      const fd = new FormData();
      fd.append("avatar", file);
      fd.append("cropData", JSON.stringify(cropData));
      const res = await fetch("/api/profile/avatar", {
        method: "POST",
        headers: authHeader(),
        body: fd,
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok) {
        setPreview(resolveSiteAvatarUrl(data.avatar_url));
        setToast({ text: "Аватар обновлён", type: "success" });
      } else setToast({ text: data.error || "Не удалось загрузить", type: "error" });
    } catch {
      setToast({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
      setCropFile(null);
    }
  }

  async function removeAvatar() {
    setBusy(true);
    setToast(null);
    try {
      const res = await fetch("/api/profile/avatar", {
        method: "DELETE",
        headers: authHeader(),
      });
      if (res.ok) {
        setPreview(resolveSiteAvatarUrl(null));
        setToast({ text: "Аватар сброшен", type: "success" });
      } else {
        const data = await res.json().catch(() => ({}));
        setToast({ text: data.error || "Не удалось удалить", type: "error" });
      }
    } catch {
      setToast({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title="Аватар" description="Показывается на сайте и в лаунчере после следующего входа.">
      <SettingsRow label="Фото" hint="PNG/JPEG/WebP, до ~2 МБ. Можно обрезать перед загрузкой.">
        <div className="flex w-full flex-col gap-3 sm:items-end">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src={preview}
            alt=""
            className="h-16 w-16 rounded-2xl border border-line object-cover bg-panel-2"
          />
          <div className="flex flex-wrap gap-2 justify-end">
            <label className="btn btn-primary btn-sm cursor-pointer">
              {busy ? "…" : "Загрузить"}
              <input
                type="file"
                accept="image/png,image/jpeg,image/webp,image/gif"
                className="hidden"
                disabled={busy}
                onChange={(e) => {
                  const f = e.target.files?.[0] ?? null;
                  if (f) setCropFile(f);
                  e.target.value = "";
                }}
              />
            </label>
            <button type="button" className="btn btn-ghost btn-sm" disabled={busy} onClick={() => void removeAvatar()}>
              Сбросить
            </button>
          </div>
        </div>
      </SettingsRow>
      {toast && <Toast text={toast.text} type={toast.type} />}
      {cropFile && (
        <AvatarCropModal
          file={cropFile}
          onCancel={() => setCropFile(null)}
          onConfirm={(crop) => void uploadWithCrop(cropFile, crop)}
        />
      )}
    </SettingsSection>
  );
}

function EmailSection({ currentEmail }: { currentEmail?: string }) {
  const [overrideEmail, setOverrideEmail] = useState<string | null>(null);
  const email = overrideEmail ?? currentEmail ?? "";
  const [open, setOpen] = useState(false);

  return (
    <SettingsSection title="Почта" description="Вход и восстановление доступа.">
      <SettingsRow label="Текущий адрес" hint="На него приходят подтверждения и сброс пароля.">
        <div className="flex w-full flex-col gap-2 sm:items-end">
          <span className="text-sm font-medium text-text break-all text-right">{email || "—"}</span>
          <button type="button" className="btn btn-secondary btn-sm self-end" onClick={() => setOpen(true)}>
            Сменить почту
          </button>
        </div>
      </SettingsRow>
      {open && (
        <EmailChangeModal
          onClose={() => setOpen(false)}
          onDone={(newEmail) => {
            setOverrideEmail(newEmail);
            setOpen(false);
          }}
        />
      )}
    </SettingsSection>
  );
}

function EmailChangeModal({
  onClose,
  onDone,
}: {
  onClose: () => void;
  onDone: (email: string) => void;
}) {
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
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/65"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="w-full max-w-sm rounded-2xl border border-line bg-panel p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="email-change-title"
      >
        <h3 id="email-change-title" className="font-display text-lg font-bold tracking-tight">
          Смена почты
        </h3>
        <p className="mt-1 text-sm text-muted">
          {step === "email"
            ? "Введите новый адрес — придёт код подтверждения."
            : `Код отправлен на ${newEmail}.`}
        </p>
        {msg && (
          <div className={`form-msg mt-4 ${msg.type === "success" ? "form-msg-ok" : "form-msg-err"}`}>
            {msg.text}
          </div>
        )}
        {step === "email" ? (
          <form onSubmit={requestCode} className="mt-5 space-y-4">
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
                placeholder="you@example.com"
                autoComplete="email"
              />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "Отправка…" : "Отправить код"}
              </button>
              <button type="button" onClick={onClose} className="btn btn-ghost">
                Отмена
              </button>
            </div>
          </form>
        ) : (
          <form onSubmit={confirmCode} className="mt-5 space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="email-code">
                Код из письма
              </label>
              <input
                id="email-code"
                inputMode="numeric"
                maxLength={6}
                required
                className="input tracking-[0.35em]"
                value={code}
                onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                placeholder="123456"
                autoComplete="one-time-code"
              />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "Проверка…" : "Подтвердить"}
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
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);

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
    setToast(null);
    try {
      const res = await fetch("/api/profile/nickname", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ nickname: nick }),
      });
      const data = await res.json();
      if (res.ok) {
        setToast({ text: "Ник обновлён. Следующая смена — через 30 дней.", type: "success" });
        setChangedAt(data.nickname_changed_at ?? new Date().toISOString());
      } else setToast({ text: data.error || "Не удалось изменить ник", type: "error" });
    } catch {
      setToast({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title="Никнейм" description="Менять можно не чаще одного раза в 30 дней.">
      <form onSubmit={save} className="space-y-1">
        <SettingsRow
          label="Ник"
          hint={onCooldown ? `Следующая смена через ${daysLeft} дн.` : "Отображается на сайте и в лаунчере."}
        >
          <div className="flex w-full flex-col gap-2 sm:items-end">
            <input
              id="acc-nick"
              className="input sm:max-w-xs"
              value={nick}
              onChange={(e) => setNick(e.target.value)}
              maxLength={32}
              placeholder="YourNickname"
              disabled={onCooldown}
              autoComplete="username"
            />
            <button type="submit" className="btn btn-primary btn-sm self-end" disabled={busy || onCooldown}>
              {busy ? "Сохранение…" : "Сменить ник"}
            </button>
          </div>
        </SettingsRow>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}

function ProfileInfoSection({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const [form, setForm] = useState({
    first_name: user?.first_name || "",
    discord_username: user?.discord || "",
  });
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await fetch("/api/profile", { headers: authHeader() });
        if (res.ok) {
          const data = await res.json();
          setForm({
            first_name: data.first_name || "",
            discord_username: data.discord || "",
          });
        }
      } catch {
        /* ignore */
      }
    })();
  }, []);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setToast(null);
    try {
      const res = await fetch("/api/profile", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({
          first_name: form.first_name,
          discord_username: form.discord_username,
        }),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setToast({ text: data.message || "Профиль сохранён", type: "success" });
      } else setToast({ text: data.error || "Ошибка сохранения", type: "error" });
    } catch {
      setToast({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title="Профиль" description="Имя и Discord — видно в ЛК; на геймплей не влияет.">
      <form onSubmit={save} className="space-y-1">
        <SettingsRow label="Имя">
          <input
            id="acc-name"
            className="input sm:max-w-xs"
            value={form.first_name}
            onChange={(e) => setForm((f) => ({ ...f, first_name: e.target.value }))}
            autoComplete="given-name"
          />
        </SettingsRow>
        <SettingsRow label="Discord" hint="Ник в Discord, без #тега.">
          <input
            id="acc-discord"
            className="input sm:max-w-xs"
            value={form.discord_username}
            onChange={(e) => setForm((f) => ({ ...f, discord_username: e.target.value }))}
          />
        </SettingsRow>
        <div className="pt-4 flex justify-end">
          <button type="submit" className="btn btn-primary btn-sm" disabled={busy}>
            {busy ? "Сохранение…" : "Сохранить профиль"}
          </button>
        </div>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}

function PasswordSection() {
  const [form, setForm] = useState({
    current_password: "",
    new_password: "",
  });
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    if (!form.new_password) {
      setToast({ text: "Введите новый пароль", type: "error" });
      return;
    }
    setBusy(true);
    setToast(null);
    try {
      const res = await fetch("/api/profile", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({
          current_password: form.current_password,
          new_password: form.new_password,
        }),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setToast({ text: data.message || "Пароль обновлён", type: "success" });
        setForm({ current_password: "", new_password: "" });
      } else setToast({ text: data.error || "Ошибка смены пароля", type: "error" });
    } catch {
      setToast({ text: "Не удалось связаться с сервером.", type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title="Безопасность" description="Смена пароля для сайта и лаунчера.">
      <form onSubmit={save} className="space-y-1">
        <SettingsRow label="Текущий пароль">
          <input
            id="acc-cur-pass"
            type="password"
            className="input sm:max-w-xs"
            autoComplete="current-password"
            value={form.current_password}
            onChange={(e) => setForm((f) => ({ ...f, current_password: e.target.value }))}
            required
          />
        </SettingsRow>
        <SettingsRow label="Новый пароль" hint="Минимум 8 символов.">
          <input
            id="acc-new-pass"
            type="password"
            className="input sm:max-w-xs"
            autoComplete="new-password"
            value={form.new_password}
            onChange={(e) => setForm((f) => ({ ...f, new_password: e.target.value }))}
            required
            minLength={8}
          />
        </SettingsRow>
        <div className="pt-4 flex justify-end">
          <button type="submit" className="btn btn-primary btn-sm" disabled={busy}>
            {busy ? "Сохранение…" : "Сменить пароль"}
          </button>
        </div>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}
