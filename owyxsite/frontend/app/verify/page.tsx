"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import AuthShell from "@/components/layout/AuthShell";

function VerifyInner() {
  const params = useSearchParams();
  const token = params.get("token") || "";
  const [state, setState] = useState<"checking" | "ok" | "error">("checking");
  const [message, setMessage] = useState("");

  useEffect(() => {
    let cancelled = false;
    async function run() {
      if (!token) {
        if (!cancelled) { setState("error"); setMessage("Токен подтверждения не указан."); }
        return;
      }
      try {
        const res = await fetch(`/api/auth/verify-email-token?token=${encodeURIComponent(token)}`);
        const data = await res.json().catch(() => ({}));
        if (!cancelled) {
          if (res.ok) { setState("ok"); setMessage(data.message || "Email подтверждён."); }
          else { setState("error"); setMessage(data.error || "Ссылка недействительна или истекла."); }
        }
      } catch {
        if (!cancelled) { setState("error"); setMessage("Ошибка сети. Попробуйте позже."); }
      }
    }
    run();
    return () => { cancelled = true; };
  }, [token]);

  if (state === "checking") {
    return <div className="panel p-5 text-sm text-muted">Подтверждаем email…</div>;
  }
  return (
    <div className={`panel p-5 text-sm ${state === "ok" ? "text-ok" : "text-danger"}`}>
      {message}
    </div>
  );
}

export default function VerifyPage() {
  return (
    <AuthShell
      title="Подтверждение email"
      subtitle="Активируем ваш email для аккаунта Owyx."
      footer={<Link href="/login" className="link-accent">Перейти ко входу →</Link>}
    >
      <Suspense fallback={<div className="panel p-5 text-sm text-muted">Загрузка…</div>}>
        <VerifyInner />
      </Suspense>
    </AuthShell>
  );
}
