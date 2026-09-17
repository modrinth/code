"use client";

import { useState } from "react";
import Link from "next/link";
import AuthShell from "@/components/layout/AuthShell";
import { useLocale } from "@/hooks/useLocale";

export default function ForgotPasswordPage() {
  const { dict } = useLocale();
  const a = dict.auth;

  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const res = await fetch("/api/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      if (res.ok) {
        setSent(true);
      } else {
        const data = await res.json().catch(() => ({}));
        setError(data.error || a.forgotFailed);
      }
    } catch {
      setError(dict.common.networkError);
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthShell
      title={a.forgotTitle}
      subtitle={a.forgotSubtitle}
      footer={
        <Link href="/login" className="link-accent">
          {a.backToLogin}
        </Link>
      }
    >
      {sent ? (
        <div className="panel p-5 text-sm text-muted">{a.forgotSent}</div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="field-label" htmlFor="email">
              {a.email}
            </label>
            <input
              id="email"
              type="email"
              autoComplete="email"
              required
              className="input"
              placeholder={a.emailPlaceholder}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          {error && <p className="text-sm text-danger">{error}</p>}
          <button type="submit" className="btn btn-primary w-full" disabled={loading}>
            {loading ? a.forgotSending : a.forgotSubmit}
          </button>
        </form>
      )}
    </AuthShell>
  );
}
