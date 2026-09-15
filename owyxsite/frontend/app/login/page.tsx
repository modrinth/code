"use client";

import { useState, useCallback } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";
import Turnstile from "@/components/ui/Turnstile";
import { notifyAuthChanged, useAuth } from "@/hooks/useAuth";

const SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || "";

export default function LoginPage() {
  const router = useRouter();
  useAuth({ redirectIfAuth: true });

  const [login, setLogin] = useState("");
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(false);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [turnstileReset, setTurnstileReset] = useState(0);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const onToken = useCallback((t: string | null) => setTurnstileToken(t), []);

  function refreshTurnstile() {
    setTurnstileToken(null);
    setTurnstileReset((n) => n + 1);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (SITE_KEY && !turnstileToken) {
      setError("Пройдите проверку, что вы не робот.");
      return;
    }
    setLoading(true);
    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ login, password, remember, turnstileToken: turnstileToken || undefined }),
      });
      const result = await res.json();
      if (res.ok && result.token) {
        localStorage.setItem("auth_token", result.token);
        localStorage.setItem("remember_me", String(remember));
        const ttl = remember ? 30 * 24 * 60 * 60 * 1000 : 24 * 60 * 60 * 1000;
        localStorage.setItem("token_expires", new Date(Date.now() + ttl).toISOString());
        notifyAuthChanged();
        router.replace("/profile");
      } else {
        setError(result.error || "Не удалось войти. Проверьте данные.");
        refreshTurnstile();
      }
    } catch {
      setError("Ошибка сети. Попробуйте позже.");
      refreshTurnstile();
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthShell
      title="Вход"
      subtitle="Войдите в аккаунт Owyx — тот же логин потом в лаунчере."
      footer={
        <>
          Нет аккаунта?{" "}
          <Link href="/register" className="link-accent">Создать</Link>
          <span className="mx-2 text-line">·</span>
          <Link href="/forgot-password" className="link-accent">Забыли пароль?</Link>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="field-label" htmlFor="login">Логин или email</label>
          <input
            id="login"
            type="text"
            autoComplete="username"
            required
            className="input"
            placeholder="steve или you@example.com"
            value={login}
            onChange={(e) => setLogin(e.target.value)}
          />
        </div>
        <div>
          <label className="field-label" htmlFor="password">Пароль</label>
          <input
            id="password"
            type="password"
            autoComplete="current-password"
            required
            className="input"
            placeholder="••••••••"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-muted select-none">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.target.checked)}
            className="accent-[var(--color-accent)] h-4 w-4"
          />
          Запомнить меня
        </label>

        {SITE_KEY && (
          <Turnstile siteKey={SITE_KEY} onToken={onToken} resetKey={turnstileReset} />
        )}

        {error && <p className="text-sm text-danger">{error}</p>}

        <button type="submit" className="btn btn-primary w-full" disabled={loading}>
          {loading ? "Вход…" : "Войти"}
        </button>
      </form>
    </AuthShell>
  );
}
