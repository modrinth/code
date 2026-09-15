"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams, useRouter } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";

function ResetPasswordInner() {
  const params = useSearchParams();
  const router = useRouter();
  const token = params.get("token") || "";

  const [checking, setChecking] = useState(true);
  const [valid, setValid] = useState(false);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [done, setDone] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function check() {
      if (!token) {
        if (!cancelled) { setValid(false); setChecking(false); }
        return;
      }
      try {
        const res = await fetch(`/api/auth/verify-reset-token?token=${encodeURIComponent(token)}`);
        if (!cancelled) setValid(res.ok);
      } catch {
        if (!cancelled) setValid(false);
      } finally {
        if (!cancelled) setChecking(false);
      }
    }
    check();
    return () => { cancelled = true; };
  }, [token]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (password.length < 8) { setError("Пароль должен быть минимум 8 символов."); return; }
    if (password !== confirm) { setError("Пароли не совпадают."); return; }
    setLoading(true);
    try {
      const res = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, password }),
      });
      if (res.ok) {
        setDone(true);
        setTimeout(() => router.replace("/login"), 1800);
      } else {
        const data = await res.json().catch(() => ({}));
        setError(data.error || "Не удалось изменить пароль.");
      }
    } catch {
      setError("Ошибка сети. Попробуйте позже.");
    } finally {
      setLoading(false);
    }
  }

  if (checking) {
    return <div className="panel p-5 text-sm text-muted">Проверяем ссылку…</div>;
  }
  if (!valid) {
    return (
      <div className="panel p-5 text-sm text-muted">
        Ссылка недействительна или истекла.{" "}
        <Link href="/forgot-password" className="link-accent">Запросить новую</Link>.
      </div>
    );
  }
  if (done) {
    return <div className="panel p-5 text-sm text-ok">Пароль изменён. Перенаправляем на вход…</div>;
  }
  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="field-label" htmlFor="password">Новый пароль</label>
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
      {error && <p className="text-sm text-danger">{error}</p>}
      <button type="submit" className="btn btn-primary w-full" disabled={loading}>
        {loading ? "Сохраняем…" : "Изменить пароль"}
      </button>
    </form>
  );
}

export default function ResetPasswordPage() {
  return (
    <AuthShell
      title="Новый пароль"
      subtitle="Придумайте новый пароль для аккаунта Owyx."
      footer={<Link href="/login" className="link-accent">← Ко входу</Link>}
    >
      <Suspense fallback={<div className="panel p-5 text-sm text-muted">Загрузка…</div>}>
        <ResetPasswordInner />
      </Suspense>
    </AuthShell>
  );
}
