"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import ServerStatus from "@/components/server/ServerStatus";
import CopyIPButton from "@/components/server/CopyIPButton";

const FALLBACK_IP = "play.owyx.site";
const FALLBACK_NAME = "Owyx Survival";

export default function ServerConnectCard() {
  const [name, setName] = useState(FALLBACK_NAME);
  const [ip, setIp] = useState(FALLBACK_IP);

  useEffect(() => {
    let cancelled = false;
    const ctrl = new AbortController();
    const timer = window.setTimeout(() => ctrl.abort(), 2500);
    (async () => {
      try {
        const res = await fetch("/api/settings/public", { signal: ctrl.signal });
        if (!res.ok) return;
        const data = await res.json();
        if (cancelled) return;
        if (data.serverName) setName(data.serverName);
        if (data.serverIp) setIp(data.serverIp);
      } catch {
        /* keep fallbacks */
      } finally {
        window.clearTimeout(timer);
      }
    })();
    return () => {
      cancelled = true;
      ctrl.abort();
      window.clearTimeout(timer);
    };
  }, []);

  return (
    <div className="mt-10 panel p-6 sm:p-8 flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <div className="text-sm text-muted">Основной сервер</div>
        <div className="mt-1 font-display text-2xl font-semibold tracking-tight">{name}</div>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <ServerStatus />
          <CopyIPButton ip={ip} />
        </div>
      </div>
      <div className="flex flex-wrap gap-3">
        <Link href="/download" className="btn btn-primary">
          Скачать лаунчер
        </Link>
        <Link href="/profile" className="btn btn-ghost">
          Кабинет
        </Link>
      </div>
    </div>
  );
}
