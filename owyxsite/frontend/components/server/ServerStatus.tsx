"use client";

import { useEffect, useRef, useState } from "react";
import { useLocale } from "@/hooks/useLocale";

interface ServerStatusData {
  online: boolean;
  players: { online: number; max: number };
}

export default function ServerStatus() {
  const { dict } = useLocale();
  const s = dict.servers;
  const c = dict.common;

  const [status, setStatus] = useState<ServerStatusData>({
    online: false,
    players: { online: 0, max: 0 },
  });
  const [loaded, setLoaded] = useState(false);
  const ctrlRef = useRef<AbortController | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchStatus() {
      ctrlRef.current?.abort();
      const ctrl = new AbortController();
      ctrlRef.current = ctrl;
      const timer = window.setTimeout(() => ctrl.abort(), 2500);
      try {
        const res = await fetch("/api/settings/server-info", { signal: ctrl.signal });
        if (res.ok) {
          const data = await res.json();
          if (!cancelled) {
            setStatus({
              online: Boolean(data.online),
              players: {
                online: data.players?.online ?? 0,
                max: data.players?.max ?? 0,
              },
            });
          }
        }
      } catch {
        /* keep last state / offline */
      } finally {
        window.clearTimeout(timer);
        if (!cancelled) setLoaded(true);
      }
    }

    fetchStatus();
    const interval = setInterval(fetchStatus, 30000);
    return () => {
      cancelled = true;
      clearInterval(interval);
      ctrlRef.current?.abort();
    };
  }, []);

  const online = status.online;

  return (
    <span className="inline-flex items-center gap-3 rounded-full border border-line bg-panel-2 px-4 py-1.5 text-sm">
      <span className="inline-flex items-center gap-2">
        <span
          className={`h-2.5 w-2.5 rounded-full ${online ? "bg-ok pulse-dot" : "bg-danger"}`}
          aria-hidden="true"
        />
        <span className={online ? "text-ok" : "text-danger"}>
          {!loaded ? c.checking : online ? s.statusOnline : s.statusOffline}
        </span>
      </span>
      {online && status.players.max > 0 && (
        <>
          <span className="text-line">·</span>
          <span className="text-muted">
            {s.playersLabel}{" "}
            <span className="font-mono text-text">
              {status.players.online}/{status.players.max}
            </span>
          </span>
        </>
      )}
    </span>
  );
}
