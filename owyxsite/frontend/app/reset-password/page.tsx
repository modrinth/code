"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams, useRouter } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";
import { useLocale } from "@/hooks/useLocale";

function ResetPasswordInner() {
  const { dict } = useLocale();
  const a = dict.auth;
  const c = dict.common;

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
        if (!cancelled) {
          setValid(false);
          setChecking(false);
        }
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
    return () => {
      cancelled = true;
    };
  }, [token]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (password.length < 8) {
      setError(a.passwordPlaceholder);
      return;
    }
    if (password !== confirm) {
      setError(a.passwordsMismatch);
      return;
    }
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
        setError(data.error || a.resetInvalid);
      }
    } catch {
      setError(c.networkError);
    } finally {
      setLoading(false);
    }
  }

  if (checking) {
    return <div className="panel p-5 text-sm text-muted">{c.checking}</div>;
  }
  if (!valid) {
    return (
      <div className="panel p-5 text-sm text-muted">
        {token ? a.resetInvalid : a.resetMissing}{" "}
        <Link href="/forgot-password" className="link-accent">
          {a.forgotSubmit}
        </Link>
      </div>
    );
  }
  if (done) {
    return <div className="panel p-5 text-sm text-ok">{a.resetDone}</div>;
  }
  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="field-label" htmlFor="password">
          {a.password}
        </label>
        <input
          id="password"
          type="password"
          autoComplete="new-password"
          required
          minLength={8}
          className="input"
          placeholder={a.passwordPlaceholder}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </div>
      <div>
        <label className="field-label" htmlFor="confirm">
          {a.confirmPassword}
        </label>
        <input
          id="confirm"
          type="password"
          autoComplete="new-password"
          required
          className="input"
          placeholder={a.passwordDots}
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
        />
      </div>
      {error && <p className="text-sm text-danger">{error}</p>}
      <button type="submit" className="btn btn-primary w-full" disabled={loading}>
        {loading ? c.saving : a.resetSubmit}
      </button>
    </form>
  );
}

export default function ResetPasswordPage() {
  const { dict } = useLocale();
  const a = dict.auth;
  const c = dict.common;

  return (
    <AuthShell
      title={a.resetTitle}
      subtitle={a.resetSubtitle}
      footer={
        <Link href="/login" className="link-accent">
          {a.backToLogin}
        </Link>
      }
    >
      <Suspense fallback={<div className="panel p-5 text-sm text-muted">{c.loading}</div>}>
        <ResetPasswordInner />
      </Suspense>
    </AuthShell>
  );
}
