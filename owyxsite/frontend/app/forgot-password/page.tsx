"use client";

import { useState } from "react";
import Link from "next/link";
import AuthShell from "@/components/layout/AuthShell";

export default function ForgotPasswordPage() {
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
        setError(data.error || "Не удалось отправить письмо.");
      }
    } catch {
      setError("Ошибка сети. Попробуйте позже.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthShell
      title="Восстановление пароля"
      subtitle="Укажите email — пришлём ссылку для сброса пароля."
      footer={<><Link href="/login" className="link-accent">← Вернуться ко входу</Link></>}
    >
      {sent ? (
        <div className="panel p-5 text-sm text-muted">
          Если такой email есть в системе, мы отправили на него ссылку для сброса пароля.
          Проверьте почту (и папку «Спам»).
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
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
          {error && <p className="text-sm text-danger">{error}</p>}
          <button type="submit" className="btn btn-primary w-full" disabled={loading}>
            {loading ? "Отправляем…" : "Отправить ссылку"}
          </button>
        </form>
      )}
    </AuthShell>
  );
}
