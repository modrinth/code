"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";
import { useLocale } from "@/hooks/useLocale";

function VerifyInner() {
  const { dict } = useLocale();
  const a = dict.auth;
  const c = dict.common;

  const params = useSearchParams();
  const token = params.get("token") || "";
  const [state, setState] = useState<"checking" | "ok" | "error">("checking");
  const [message, setMessage] = useState("");

  useEffect(() => {
    let cancelled = false;
    async function run() {
      if (!token) {
        if (!cancelled) {
          setState("error");
          setMessage(a.verifyMissing);
        }
        return;
      }
      try {
        const res = await fetch(`/api/auth/verify-email-token?token=${encodeURIComponent(token)}`);
        const data = await res.json().catch(() => ({}));
        if (!cancelled) {
          if (res.ok) {
            setState("ok");
            setMessage(data.message || a.verifyOk);
          } else {
            setState("error");
            setMessage(data.error || a.verifyBad);
          }
        }
      } catch {
        if (!cancelled) {
          setState("error");
          setMessage(c.networkError);
        }
      }
    }
    run();
    return () => {
      cancelled = true;
    };
  }, [token, a.verifyMissing, a.verifyOk, a.verifyBad, c.networkError]);

  if (state === "checking") {
    return <div className="panel p-5 text-sm text-muted">{a.verifyChecking}</div>;
  }
  return (
    <div className={`panel p-5 text-sm ${state === "ok" ? "text-ok" : "text-danger"}`}>{message}</div>
  );
}

export default function VerifyPage() {
  const { dict } = useLocale();
  const a = dict.auth;
  const c = dict.common;

  return (
    <AuthShell
      title={a.verifyTitle}
      footer={
        <Link href="/login" className="link-accent">
          {a.backToLogin}
        </Link>
      }
    >
      <Suspense fallback={<div className="panel p-5 text-sm text-muted">{c.loading}</div>}>
        <VerifyInner />
      </Suspense>
    </AuthShell>
  );
}
