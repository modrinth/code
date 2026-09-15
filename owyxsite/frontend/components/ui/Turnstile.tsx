"use client";

import Script from "next/script";
import { useEffect, useRef } from "react";

/**
 * Cloudflare Turnstile — explicit render (existing-widget flow).
 * Mount only when NEXT_PUBLIC_TURNSTILE_SITE_KEY is a real sitekey.
 * Docs: https://developers.cloudflare.com/turnstile/
 */
declare global {
  interface Window {
    turnstile?: {
      render: (el: HTMLElement, opts: Record<string, unknown>) => string;
      remove: (id: string) => void;
      reset?: (id?: string) => void;
    };
    onOwyxTurnstileLoad?: () => void;
  }
}

export default function Turnstile({
  siteKey,
  onToken,
  resetKey = 0,
}: {
  siteKey: string;
  onToken: (token: string | null) => void;
  /** Change to force a fresh widget (e.g. after failed login). */
  resetKey?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const widgetId = useRef<string | null>(null);
  const onTokenRef = useRef(onToken);
  onTokenRef.current = onToken;

  useEffect(() => {
    if (!siteKey) return;
    let cancelled = false;

    function tryRender() {
      if (cancelled || !window.turnstile || !ref.current || widgetId.current) return;
      widgetId.current = window.turnstile.render(ref.current, {
        sitekey: siteKey,
        theme: "dark",
        appearance: "always",
        callback: (token: string) => {
          if (!cancelled) onTokenRef.current(token);
        },
        "expired-callback": () => {
          if (!cancelled) onTokenRef.current(null);
        },
        "error-callback": () => {
          if (!cancelled) onTokenRef.current(null);
        },
      });
    }

    window.onOwyxTurnstileLoad = tryRender;
    tryRender();
    const poll = window.setInterval(tryRender, 250);

    return () => {
      cancelled = true;
      window.clearInterval(poll);
      if (widgetId.current && window.turnstile?.remove) {
        try {
          window.turnstile.remove(widgetId.current);
        } catch {
          /* ignore */
        }
      }
      widgetId.current = null;
      if (window.onOwyxTurnstileLoad === tryRender) {
        delete window.onOwyxTurnstileLoad;
      }
    };
  }, [siteKey, resetKey]);

  if (!siteKey) return null;

  return (
    <div className="space-y-1.5">
      <p className="field-hint">Подтвердите, что вы не робот</p>
      <Script
        src="https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit&onload=onOwyxTurnstileLoad"
        strategy="afterInteractive"
        async
      />
      <div ref={ref} className="cf-turnstile min-h-[65px]" />
    </div>
  );
}
