"use client";

import { useState, useCallback } from "react";
import Link from "next/link";
import AuthShell from "@/components/layout/AuthShell";
import Turnstile from "@/components/ui/Turnstile";
import { useAuth } from "@/hooks/useAuth";

const SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || "";

export default function RegisterPage() {
  useAuth({ redirectIfAuth: true });

  const [nick, setNick] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [done, setDone] = useState(false);
  const [loading, setLoading] = useState(false);

  const onToken = useCallback((t: string | null) => setTurnstileToken(t), []);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (!/^[a-zA-Z0-9_]{3,32}$/.test(nick)) {
      setError("Ник: 3–32 символа, только буквы, цифры и _.");
      return;
    }
    if (password.length < 8) {
      setError("Пароль должен быть минимум 8 символов.");
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
          minecraft_nick: nick,
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
          <label className="field-label" htmlFor="nick">Minecraft-ник</label>
          <input
            id="nick"
            className="input"
            required
            minLength={3}
            maxLength={32}
            placeholder="Steve"
            value={nick}
            onChange={(e) => setNick(e.target.value)}
          />
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
          />
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

        <button type="submit" className="btn btn-primary w-full" disabled={loading}>
          {loading ? "Создаём…" : "Создать аккаунт"}
        </button>
      </form>
    </AuthShell>
  );
}
