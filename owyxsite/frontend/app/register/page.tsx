"use client";

import { useMemo, useState, useCallback } from "react";
import Link from "next/link";
import AuthShell from "@/components/layout/AuthShell";
import Turnstile from "@/components/ui/Turnstile";
import { useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";

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

const STRENGTH_COLOR = ["", "bg-danger", "bg-orange-400", "bg-accent", "bg-ok"] as const;

export default function RegisterPage() {
  const { dict } = useLocale();
  const a = dict.auth;
  useAuth({ redirectIfAuth: true });

  const strengthLabels = useMemo(
    () => ["", a.strengthWeak, a.strengthFair, a.strengthGood, a.strengthStrong] as const,
    [a.strengthWeak, a.strengthFair, a.strengthGood, a.strengthStrong]
  );

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
    if (!/^[a-zA-Z0-9_]{3,16}$/.test(login)) {
      setError(a.loginInvalid);
      return;
    }
    if (!strongEnough) {
      setError(a.passwordWeak);
      return;
    }
    if (password !== confirm) {
      setError(a.passwordsMismatch);
      return;
    }
    if (SITE_KEY && !turnstileToken) {
      setError(a.captchaRequired);
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
        setError(result.error || result.details?.[0]?.msg || a.registerFailed);
      }
    } catch {
      setError(dict.common.networkError);
    } finally {
      setLoading(false);
    }
  }

  if (done) {
    return (
      <AuthShell
        title={a.almostReady}
        subtitle={a.almostReadyBody}
        footer={
          <>
            {a.readyQuestion}{" "}
            <Link href="/login" className="link-accent">
              {a.submitLogin}
            </Link>
          </>
        }
      >
        <div className="panel p-5 text-sm text-muted">{a.almostReadyPanel}</div>
      </AuthShell>
    );
  }

  return (
    <AuthShell
      title={a.registerTitle}
      subtitle={a.registerSubtitle}
      footer={
        <>
          {a.haveAccount}{" "}
          <Link href="/login" className="link-accent">
            {a.submitLogin}
          </Link>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="field-label" htmlFor="login">
            {a.username}
          </label>
          <input
            id="login"
            className="input"
            required
            minLength={3}
            maxLength={16}
            autoComplete="username"
            placeholder={a.usernamePlaceholder}
            value={login}
            onChange={(e) => setLogin(e.target.value)}
          />
          <p className="field-hint mt-1.5">{a.usernameHint}</p>
        </div>
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
                <span className="font-mono text-[11px] text-muted shrink-0">{strengthLabels[score]}</span>
              </div>
              <ul className="text-xs text-muted space-y-1 m-0 pl-4 list-disc">
                <li className={checks.length ? "text-ok" : undefined}>{a.passwordRuleLen}</li>
                <li className={checks.upper ? "text-ok" : undefined}>{a.passwordRuleUpper}</li>
                <li className={checks.digit ? "text-ok" : undefined}>{a.passwordRuleDigit}</li>
                <li className={checks.special ? "text-ok" : undefined}>{a.passwordRuleSpecial}</li>
              </ul>
            </div>
          )}
          {!password && <p className="field-hint mt-1.5">{a.passwordHint}</p>}
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
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        </div>

        {SITE_KEY && <Turnstile siteKey={SITE_KEY} onToken={onToken} />}

        {error && <p className="text-sm text-danger">{error}</p>}

        <button type="submit" className="btn btn-primary w-full" disabled={loading || !strongEnough}>
          {loading ? a.submittingRegister : a.submitRegister}
        </button>
      </form>
    </AuthShell>
  );
}
