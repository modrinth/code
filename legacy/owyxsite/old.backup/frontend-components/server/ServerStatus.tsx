"use client";

import { useEffect, useState } from "react";

interface ServerStatusData {
  online: boolean;
  players: { online: number; max: number };
}

export default function ServerStatus() {
  const [status, setStatus] = useState<ServerStatusData>({
    online: false,
    players: { online: 0, max: 50 },
  });

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  async function fetchStatus() {
    try {
      const res = await fetch("/api/settings/server-info");
      if (res.ok) {
        const data = await res.json();
        setStatus(data);
      }
    } catch {
      // Server unavailable — keep current state
    }
  }

  return (
    <div className="flex flex-col md:flex-row items-center justify-center space-y-4 md:space-y-0 md:space-x-8">
      {/* Online/Offline status */}
      <div className="flex items-center">
        <div
          className={`w-4 h-4 rounded-full mr-3 animate-pulse ${
            status.online ? "bg-green-400" : "bg-red-400"
          }`}
          id="server-status-indicator"
        />
        <span className="text-gray-300 text-lg">Сервер:</span>
        <span
          className={`font-bold ml-2 text-lg ${
            status.online ? "text-green-400" : "text-red-400"
          }`}
          id="server-status-text"
        >
          {status.online ? "Онлайн" : "Оффлайн"}
        </span>
      </div>

      {/* Separator */}
      <div className="hidden md:block text-[#FFAA00] text-2xl">│</div>

      {/* Player count */}
      <div className="text-gray-300 text-lg">
        <span>Игроков:</span>
        <span className="text-[#FFFF55] font-bold ml-2" id="player-count">
          {status.players.online}/{status.players.max}
        </span>
      </div>
    </div>
  );
}
