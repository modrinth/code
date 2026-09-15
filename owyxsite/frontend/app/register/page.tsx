"use client";

import { useMemo, useState, useCallback } from "react";
import Link from "next/link";
import AuthShell from "@/components/layout/AuthShell";
import Turnstile from "@/components/ui/Turnstile";
import { useAuth } from "@/hooks/useAuth";

const SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || "";

function passwordChecks(pw: string) {
  return {
    length: pw.length >= 8,
    upper: /[A-ZА-ЯЁ]/.test(pw),
    digit: /\d/.test(pw),
    special: /[^A-Za-zА-Яа-яЁё0-9]/.test(pw),
  };
}

function passwordScore(pw: string): 0 | 1 | 2 | 3 | 4 {
  if (!pw) return 0;
  const c = passwordChecks(pw);
  const met = [c.length, c.upper, c.digit, c.special].filter(Boolean).length;
  if (met <= 1) return 1;
  if (met === 2) return 2;
  if (met === 3) return 3;
  return 4;
}

const STRENGTH_LABEL = ["", "Слабый", "Средний", "Хороший", "Надёжный"] as const;
const STRENGTH_COLOR = ["", "bg-danger", "bg-orange-400", "bg-accent", "bg-ok"] as const;

export default function RegisterPage() {
  useAuth({ redirectIfAuth: true });

  const [login, setLogin] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [done, setDone] = useState(false);
  const [loading, setLoading] = useState(false);

  const onToken = useCallback((t: string | null) => setTurnstileToken(t), []);
  const checks = useMemo(() => passwordChecks(password), [password]);
  const score = useMemo(() => passwordScore(password), [password]);
  const strongEnough = checks.length && checks.upper && checks.digit && checks.special;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (!/^[a-zA-Z0-9_]{3,32}$/.test(login)) {
      setError("Логин: 3–32 символа, только буквы, цифры и _.");
      return;
    }
    if (!strongEnough) {
      setError("Пароль: минимум 8 символов, 1 заглавная буква, 1 цифра и 1 спецсимвол.");
      return;
    }
    if (password !== confirm) {
      setError("Пароли не совпадают.");
      return;
    }
    if (SITE_KEY && !turnstileToken) {
      setError("Пройдите проверку, что вы не робот.");
      return;
    }
    setLoading(true);
    try {
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          login,
          email,
          password,
          turnstileToken: turnstileToken || undefined,
        }),
      });
      const result = await res.json();
      if (res.ok) {
        setDone(true);
      } else {
        setError(result.error || result.details?.[0]?.msg || "Не удалось зарегистрироваться.");
      }
    } catch {
      setError("Ошибка сети. Попробуйте позже.");
    } finally {
      setLoading(false);
    }
  }

  if (done) {
    return (
      <AuthShell
        title="Почти готово"
        subtitle="Аккаунт создан. Проверьте email — мы отправили ссылку для подтверждения."
        footer={<>Готово? <Link href="/login" className="link-accent">Войти</Link></>}
      >
        <div className="panel p-5 text-sm text-muted">
          Подтвердите email, затем скачайте лаунчер и войдите тем же аккаунтом.
        </div>
      </AuthShell>
    );
  }

  return (
    <AuthShell
      title="Создать аккаунт"
      subtitle="Один аккаунт Owyx: сайт, доступ к серверам и лаунчер."
      footer={<>Уже есть аккаунт? <Link href="/login" className="link-accent">Войти</Link></>}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="field-label" htmlFor="login">Логин</label>
          <input
            id="login"
            className="input"
            required
            minLength={3}
            maxLength={32}
            autoComplete="username"
            placeholder="steve"
            value={login}
            onChange={(e) => setLogin(e.target.value)}
          />
          <p className="field-hint mt-1.5">Этот же логин будет в лаунчере и в игре (если не входишь через Microsoft).</p>
        </div>
        <div>
          <label className="field-label" htmlFor="email">Email</label>
          <input
            id="email"
            type="email"
            autoComplete="email"
            required
            className="input"
            placeholder="you@example.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
        </div>
        <div>
          <label className="field-label" htmlFor="password">Пароль</label>
          <input
            id="password"
            type="password"
            autoComplete="new-password"
            required
            minLength={8}
            className="input"
            placeholder="Минимум 8 символов"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            aria-describedby="password-hints"
          />
          {password.length > 0 && (
            <div className="mt-2 space-y-2" id="password-hints">
              <div className="flex items-center gap-2">
                <div className="flex-1 h-1.5 rounded-full bg-panel-2 overflow-hidden flex gap-0.5">
                  {[1, 2, 3, 4].map((i) => (
                    <div
                      key={i}
                      className={`h-full flex-1 rounded-full transition-colors ${
                        score >= i ? STRENGTH_COLOR[score] : "bg-transparent"
                      }`}
                    />
                  ))}
                </div>
                <span className="font-mono text-[11px] text-muted shrink-0">
                  {STRENGTH_LABEL[score]}
                </span>
              </div>
              <ul className="text-xs text-muted space-y-1 m-0 pl-4 list-disc">
                <li className={checks.length ? "text-ok" : undefined}>не меньше 8 символов</li>
                <li className={checks.upper ? "text-ok" : undefined}>минимум 1 заглавная буква</li>
                <li className={checks.digit ? "text-ok" : undefined}>минимум 1 цифра</li>
                <li className={checks.special ? "text-ok" : undefined}>минимум 1 спецсимвол (!@#$…)</li>
              </ul>
            </div>
          )}
          {!password && (
            <p className="field-hint mt-1.5">
              Нужны: заглавная буква, цифра и спецсимвол (например Owyx!2026).
            </p>
          )}
        </div>
        <div>
          <label className="field-label" htmlFor="confirm">Повторите пароль</label>
          <input
            id="confirm"
            type="password"
            autoComplete="new-password"
            required
            className="input"
            placeholder="••••••••"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        </div>

        {SITE_KEY && <Turnstile siteKey={SITE_KEY} onToken={onToken} />}

        {error && <p className="text-sm text-danger">{error}</p>}

        <button type="submit" className="btn btn-primary w-full" disabled={loading || !strongEnough}>
          {loading ? "Создаём…" : "Создать аккаунт"}
        </button>
      </form>
    </AuthShell>
  );
}
