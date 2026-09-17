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
import { useLocale } from "@/hooks/useLocale";
import type { Locale } from "@/lib/i18n";
import { resolveSiteAvatarUrl } from "@/lib/avatar";

const authHeader = () => ({ Authorization: `Bearer ${localStorage.getItem("auth_token")}` });

function dateLocale(locale: Locale): string {
  return locale === "en_US" ? "en-US" : "ru-RU";
}

function formatDate(dateStr: string | undefined, locale: Locale, dash: string): string {
  if (!dateStr) return dash;
  return new Date(dateStr).toLocaleDateString(dateLocale(locale), {
    day: "2-digit",
    month: "long",
    year: "numeric",
  });
}

function roleLabel(role: string | undefined, p: ReturnType<typeof useLocale>["dict"]["profile"]) {
  if (role === "admin") return { text: p.roleAdmin, cls: "badge-danger" };
  if (role === "moderator") return { text: p.roleModerator, cls: "badge-accent" };
  if (role === "helper") return { text: p.roleHelper, cls: "badge-accent" };
  return { text: p.rolePlayer, cls: "" };
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
  const { dict, locale } = useLocale();
  const p = dict.profile;
  const { user, loading, logout } = useAuth({ requireAuth: true });
  const [tab, setTab] = useState<Tab>("overview");

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-muted">{dict.common.loading}</p>
      </div>
    );
  }

  const role = roleLabel(user?.role, p);
  const avatarSrc = resolveSiteAvatarUrl(user?.avatar_url);

  return (
    <>
      <Header />
      <CabinetShell
        eyebrow={p.eyebrow}
        title={user?.display_nickname || user?.nickname || p.playerFallback}
        subtitle={p.subtitle}
        actions={
          <>
            <Link href="/download" className="btn btn-primary">
              {p.downloadLauncher}
            </Link>
            <button type="button" onClick={logout} className="btn btn-ghost" id="profile-logout-btn">
              {p.logout}
            </button>
          </>
        }
        nav={[
          { id: "overview", label: p.navOverview, hint: p.navOverviewHint },
          { id: "settings", label: p.navSettings, hint: p.navSettingsHint },
        ]}
        activeId={tab}
        onNav={(id) => setTab(id as Tab)}
        footerNote={p.footerNote}
      >
        {tab === "overview" && (
          <OverviewPane user={user} role={role} avatarSrc={avatarSrc} locale={locale} />
        )}
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
  locale,
}: {
  user: ReturnType<typeof useAuth>["user"];
  role: { text: string; cls: string };
  avatarSrc: string;
  locale: Locale;
}) {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;
  const now = useStableNow();
  if (!user) return null;
  const daysWithUs = user.created_at
    ? Math.max(0, Math.floor((now - new Date(user.created_at).getTime()) / 86400000))
    : 0;

  const rows = [
    { label: p.labelDisplayNick, value: user.display_nickname || user.nickname || c.dash },
    { label: p.labelLogin, value: user.nickname || c.dash },
    { label: p.labelName, value: user.first_name || c.dash },
    { label: p.labelEmail, value: user.email || c.dash },
    { label: p.labelDiscord, value: user.discord || c.dash },
    { label: p.labelRegistered, value: formatDate(user.created_at, locale, c.dash) },
    { label: p.labelDays, value: String(daysWithUs) },
  ];

  return (
    <div className="space-y-7">
      <div className="flex flex-wrap items-center gap-4">
        <div className="grid h-16 w-16 shrink-0 place-items-center overflow-hidden rounded-2xl border border-line bg-panel-2">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={avatarSrc} alt="" className="h-full w-full object-cover" />
        </div>
        <div className="min-w-0">
          <p className="font-display text-xl font-bold tracking-tight truncate">
            {user.display_nickname || user.nickname}
          </p>
          {user.nickname &&
            user.display_nickname &&
            user.nickname !== user.display_nickname && (
              <p className="mt-1 text-xs text-muted truncate">
                {p.labelLogin}: {user.nickname}
              </p>
            )}
          <div className="mt-2 flex flex-wrap gap-2">
            <span className={`badge ${role.cls}`}>{role.text}</span>
            <span className={`badge ${user.status === "banned" ? "badge-danger" : "badge-ok"}`}>
              {user.status === "banned" ? p.statusBanned : p.statusActive}
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
        <p className="text-sm font-medium text-text">{p.nextTitle}</p>
        <p className="mt-1 text-sm text-muted leading-relaxed">{p.nextBody}</p>
        <Link href="/download" className="btn btn-primary mt-4">
          {p.goDownload}
        </Link>
      </div>
    </div>
  );
}

function SettingsPane({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const { dict } = useLocale();
  const p = dict.profile;

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-2 lg:items-start lg:gap-8">
      <div className="flex flex-col gap-5 min-w-0">
        <h2 className="font-display text-base font-bold tracking-tight text-text m-0">
          {p.groupProfile}
        </h2>
        <AvatarSection currentUrl={user?.avatar_url} />
        <DisplayNicknameSection currentNick={user?.display_nickname || user?.nickname} />
        <ProfileInfoSection user={user} />
      </div>
      <div className="flex flex-col gap-5 min-w-0">
        <h2 className="font-display text-base font-bold tracking-tight text-text m-0">
          {p.groupSecurity}
        </h2>
        <NicknameSection currentNick={user?.nickname} />
        <EmailSection currentEmail={user?.email} />
        <PasswordSection />
      </div>
    </div>
  );
}

function AvatarSection({ currentUrl }: { currentUrl?: string | null }) {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

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
        setToast({ text: p.avatarUpdated, type: "success" });
      } else setToast({ text: data.error || p.uploadFailed, type: "error" });
    } catch {
      setToast({ text: c.serverError, type: "error" });
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
        setToast({ text: p.avatarCleared, type: "success" });
      } else {
        const data = await res.json().catch(() => ({}));
        setToast({ text: data.error || p.deleteFailed, type: "error" });
      }
    } catch {
      setToast({ text: c.serverError, type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title={p.avatarTitle} description={p.avatarDesc}>
      <SettingsRow label={p.avatarPhoto} hint={p.avatarHint}>
        <div className="flex w-full flex-col gap-3 sm:items-end">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src={preview}
            alt=""
            className="h-16 w-16 rounded-2xl border border-line object-cover bg-panel-2"
          />
          <div className="flex flex-wrap gap-2 justify-end">
            <label className="btn btn-primary btn-sm cursor-pointer">
              {busy ? c.saving : p.upload}
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
              {p.reset}
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
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

  const [overrideEmail, setOverrideEmail] = useState<string | null>(null);
  const email = overrideEmail ?? currentEmail ?? "";
  const [open, setOpen] = useState(false);

  return (
    <SettingsSection title={p.emailTitle} description={p.emailDesc}>
      <SettingsRow label={p.emailCurrent} hint={p.emailHint}>
        <div className="flex w-full flex-col gap-2 sm:items-end">
          <span className="text-sm font-medium text-text break-all text-right">{email || c.dash}</span>
          <button type="button" className="btn btn-secondary btn-sm self-end" onClick={() => setOpen(true)}>
            {p.changeEmail}
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
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

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
            ? `Dev: ${data.devCode}`
            : p.codeSent;
        setMsg({ text: shown, type: "success" });
      } else setMsg({ text: data.error || p.codeSendFailed, type: "error" });
    } catch {
      setMsg({ text: c.serverError, type: "error" });
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
      else setMsg({ text: data.error || p.badCode, type: "error" });
    } catch {
      setMsg({ text: c.serverError, type: "error" });
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
          {p.emailChangeTitle}
        </h3>
        <p className="mt-1 text-sm text-muted">
          {step === "email" ? p.emailChangeStep1 : p.emailChangeStep2.replace("{email}", newEmail)}
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
                {p.emailNew}
              </label>
              <input
                id="new-email"
                type="email"
                required
                className="input"
                value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)}
                placeholder={p.emailPlaceholder}
                autoComplete="email"
              />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? p.sending : p.sendCode}
              </button>
              <button type="button" onClick={onClose} className="btn btn-ghost">
                {c.cancel}
              </button>
            </div>
          </form>
        ) : (
          <form onSubmit={confirmCode} className="mt-5 space-y-4">
            <div className="field">
              <label className="field-label" htmlFor="email-code">
                {p.codeFromMail}
              </label>
              <input
                id="email-code"
                inputMode="numeric"
                maxLength={6}
                required
                className="input tracking-[0.35em]"
                value={code}
                onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                placeholder={p.codePlaceholder}
                autoComplete="one-time-code"
              />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? c.checking : c.confirm}
              </button>
              <button type="button" onClick={() => setStep("email")} className="btn btn-ghost">
                {c.back}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

function DisplayNicknameSection({ currentNick }: { currentNick?: string }) {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

  const [nick, setNick] = useState(currentNick || "");
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);

  useEffect(() => {
    setNick(currentNick || "");
  }, [currentNick]);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setToast(null);
    try {
      const res = await fetch("/api/profile/display-nickname", {
        method: "PUT",
        headers: { ...authHeader(), "Content-Type": "application/json" },
        body: JSON.stringify({ displayNickname: nick }),
      });
      const data = await res.json();
      if (res.ok) {
        setNick(data.display_nickname || nick);
        setToast({ text: p.displayNickUpdated, type: "success" });
      } else setToast({ text: data.error || p.displayNickFailed, type: "error" });
    } catch {
      setToast({ text: c.serverError, type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title={p.displayNickTitle} description={p.displayNickDesc}>
      <form onSubmit={save} className="space-y-1">
        <SettingsRow label={p.displayNickLabel} hint={p.displayNickHint}>
          <div className="flex w-full flex-col gap-2 sm:items-end">
            <input
              id="acc-display-nick"
              className="input sm:max-w-xs"
              value={nick}
              onChange={(e) => setNick(e.target.value)}
              maxLength={16}
              placeholder={p.displayNickPlaceholder}
              autoComplete="off"
              spellCheck={false}
            />
            <button type="submit" className="btn btn-primary btn-sm self-end" disabled={busy}>
              {busy ? c.saving : p.changeDisplayNick}
            </button>
          </div>
        </SettingsRow>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}

function NicknameSection({ currentNick }: { currentNick?: string }) {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

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
        setToast({ text: p.nickUpdated, type: "success" });
        setChangedAt(data.nickname_changed_at ?? new Date().toISOString());
      } else setToast({ text: data.error || p.nickFailed, type: "error" });
    } catch {
      setToast({ text: c.serverError, type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title={p.nickTitle} description={p.nickDesc}>
      <form onSubmit={save} className="space-y-1">
        <SettingsRow
          label={p.nickLabel}
          hint={
            onCooldown ? p.nickCooldown.replace("{days}", String(daysLeft)) : p.nickHint
          }
        >
          <div className="flex w-full flex-col gap-2 sm:items-end">
            <input
              id="acc-nick"
              className="input sm:max-w-xs"
              value={nick}
              onChange={(e) => setNick(e.target.value)}
              maxLength={16}
              placeholder={p.nickPlaceholder}
              disabled={onCooldown}
              autoComplete="username"
            />
            <button type="submit" className="btn btn-primary btn-sm self-end" disabled={busy || onCooldown}>
              {busy ? c.saving : p.changeNick}
            </button>
          </div>
        </SettingsRow>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}

function ProfileInfoSection({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

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
        setToast({ text: data.message || p.profileSaved, type: "success" });
      } else setToast({ text: data.error || p.profileSaveFailed, type: "error" });
    } catch {
      setToast({ text: c.serverError, type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title={p.infoTitle} description={p.infoDesc}>
      <form onSubmit={save} className="space-y-1">
        <SettingsRow label={p.labelName}>
          <input
            id="acc-name"
            className="input sm:max-w-xs"
            value={form.first_name}
            onChange={(e) => setForm((f) => ({ ...f, first_name: e.target.value }))}
            autoComplete="given-name"
          />
        </SettingsRow>
        <SettingsRow label={p.labelDiscord} hint={p.discordHint}>
          <input
            id="acc-discord"
            className="input sm:max-w-xs"
            value={form.discord_username}
            onChange={(e) => setForm((f) => ({ ...f, discord_username: e.target.value }))}
          />
        </SettingsRow>
        <div className="pt-4 flex justify-end">
          <button type="submit" className="btn btn-primary btn-sm" disabled={busy}>
            {busy ? c.saving : p.saveProfile}
          </button>
        </div>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}

function PasswordSection() {
  const { dict } = useLocale();
  const p = dict.profile;
  const c = dict.common;

  const [form, setForm] = useState({
    current_password: "",
    new_password: "",
  });
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ text: string; type: "success" | "error" } | null>(null);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    if (!form.new_password) {
      setToast({ text: p.enterNewPassword, type: "error" });
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
        setToast({ text: data.message || p.passwordUpdated, type: "success" });
        setForm({ current_password: "", new_password: "" });
      } else setToast({ text: data.error || p.passwordFailed, type: "error" });
    } catch {
      setToast({ text: c.serverError, type: "error" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsSection title={p.securityTitle} description={p.securityDesc}>
      <form onSubmit={save} className="space-y-1">
        <SettingsRow label={p.currentPassword}>
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
        <SettingsRow label={p.newPassword} hint={p.newPasswordHint}>
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
            {busy ? c.saving : p.changePassword}
          </button>
        </div>
      </form>
      {toast && <Toast text={toast.text} type={toast.type} />}
    </SettingsSection>
  );
}
