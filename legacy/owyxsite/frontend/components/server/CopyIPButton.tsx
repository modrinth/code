"use client";

import { useState, useCallback } from "react";

export default function CopyIPButton({ ip }: { ip: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(ip);
      } else {
        const ta = document.createElement("textarea");
        ta.value = ip;
        ta.style.cssText = "position:fixed;top:0;left:0;opacity:0";
        document.body.appendChild(ta);
        ta.focus();
        ta.select();
        document.execCommand("copy");
        document.body.removeChild(ta);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* ignore */
    }
  }, [ip]);

  return (
    <button
      type="button"
      onClick={handleCopy}
      title="Скопировать IP"
      className="inline-flex items-center gap-2 rounded-[10px] border border-line bg-panel px-3.5 py-2 font-mono text-sm text-text hover:border-accent transition-colors cursor-pointer"
    >
      <span>{ip}</span>
      <svg className={`h-4 w-4 ${copied ? "text-ok" : "text-muted"}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
        {copied ? (
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        ) : (
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        )}
      </svg>
      <span className="sr-only">{copied ? "IP скопирован" : "Скопировать IP"}</span>
    </button>
  );
}
