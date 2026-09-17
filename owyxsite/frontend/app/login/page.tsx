"use client";

import { useState, useCallback } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";
import Turnstile from "@/components/ui/Turnstile";
import { notifyAuthChanged, useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";

const SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || "";

export default function LoginPage() {
  const router = useRouter();
  const { dict } = useLocale();
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
      setError(dict.auth.captchaRequired);
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
        setError(result.error || dict.auth.loginFailed);
        refreshTurnstile();
      }
    } catch {
      setError(dict.common.networkError);
      refreshTurnstile();
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthShell
      title={dict.auth.loginTitle}
      subtitle={dict.auth.loginSubtitle}
      footer={
        <>
          {dict.auth.noAccount}{" "}
          <Link href="/register" className="link-accent">
            {dict.auth.createOne}
          </Link>
          <span className="mx-2 text-line">·</span>
          <Link href="/forgot-password" className="link-accent">
            {dict.auth.forgotPassword}
          </Link>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="field-label" htmlFor="login">
            {dict.auth.loginLabel}
          </label>
          <input
            id="login"
            type="text"
            autoComplete="username"
            required
            className="input"
            placeholder={dict.auth.loginPlaceholder}
            value={login}
            onChange={(e) => setLogin(e.target.value)}
          />
        </div>
        <div>
          <label className="field-label" htmlFor="password">
            {dict.auth.password}
          </label>
          <input
            id="password"
            type="password"
            autoComplete="current-password"
            required
            className="input"
            placeholder={dict.auth.passwordDots}
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
          {dict.auth.rememberMe}
        </label>

        {SITE_KEY && <Turnstile siteKey={SITE_KEY} onToken={onToken} resetKey={turnstileReset} />}

        {error && <p className="text-sm text-danger">{error}</p>}

        <button type="submit" className="btn btn-primary w-full" disabled={loading}>
          {loading ? dict.auth.submittingLogin : dict.auth.submitLogin}
        </button>
      </form>
    </AuthShell>
  );
}
