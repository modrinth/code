"use client";

import Script from "next/script";
import { useEffect, useRef } from "react";

// Cloudflare Turnstile explicit render. Only mounted when a public site key is
// configured; in local/dev (no NEXT_PUBLIC_TURNSTILE_SITE_KEY) it is skipped and
// the backend also skips verification (TURNSTILE_SKIP / no secret).
declare global {
  interface Window {
    turnstile?: {
      render: (el: HTMLElement, opts: Record<string, unknown>) => string;
      remove: (id: string) => void;
    };
  }
}

export default function Turnstile({
  siteKey,
  onToken,
}: {
  siteKey: string;
  onToken: (token: string | null) => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const widgetId = useRef<string | null>(null);

  useEffect(() => {
    let mounted = true;
    function render() {
      if (!window.turnstile || !ref.current || widgetId.current) return;
      widgetId.current = window.turnstile.render(ref.current, {
        sitekey: siteKey,
        theme: "dark",
        callback: (t: string) => mounted && onToken(t),
        "expired-callback": () => mounted && onToken(null),
        "error-callback": () => mounted && onToken(null),
      });
    }
    render();
    const id = setInterval(render, 300);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, [siteKey, onToken]);

  return (
    <>
      <Script src="https://challenges.cloudflare.com/turnstile/v0/api.js" strategy="lazyOnload" />
      <div ref={ref} className="mt-1" />
    </>
  );
}
