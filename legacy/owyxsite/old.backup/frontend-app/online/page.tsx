"use client";

import { useState, useEffect, useCallback } from "react";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

interface Player {
  name?: string;
  nickname?: string;
  role?: string;
  trust_level?: number;
  playtime?: number;
  session_start?: string;
}

interface ServerData {
  status?: string;
  online?: boolean;
  players?: {
    online?: number;
    max?: number;
    list?: Player[];
  };
  server?: {
    version?: string;
  };
  performance?: {
    tps?: number;
    ping?: number;
    uptime_seconds?: number;
  };
}

function formatPlayTime(minutes: number): string {
  if (minutes < 60) return `${minutes} мин`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} ч`;
  const days = Math.floor(hours / 24);
  return `${days} дн`;
}

function getSessionTime(sessionStart?: string): string {
  if (!sessionStart) return "Неизвестно";
  const start = new Date(sessionStart);
  const now = new Date();
  const diff = Math.floor((now.getTime() - start.getTime()) / 1000 / 60);
  if (diff < 60) return `${diff} мин`;
  const hours = Math.floor(diff / 60);
  const minutes = diff % 60;
  return `${hours}ч ${minutes}м`;
}

function getTrustLevelInfo(level: number): { name: string; color: string } {
  const levels: Record<number, { name: string; color: string }> = {
    0: { name: "Проходимец", color: "text-gray-400" },
    1: { name: "Новичок", color: "text-blue-400" },
    2: { name: "Проверенный", color: "text-green-400" },
    3: { name: "Ветеран", color: "text-amber-400" },
    4: { name: "Легенда", color: "text-purple-400" },
  };
  return levels[level] ?? levels[0];
}

function getRoleBadge(role?: string) {
  if (!role) return null;
  const badges: Record<string, { label: string; color: string }> = {
    admin: { label: "ADMIN", color: "bg-red-600" },
    moderator: { label: "MOD", color: "bg-blue-600" },
    helper: { label: "HELPER", color: "bg-green-600" },
  };
  const badge = badges[role];
  if (!badge) return null;
  return (
    <span className={`ml-2 px-2 py-0.5 text-xs ${badge.color} text-white rounded font-bold`}>
      {badge.label}
    </span>
  );
}

function formatUptime(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) return "N/A";
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}д ${hours}ч ${minutes}м`;
  if (hours > 0) return `${hours}ч ${minutes}м`;
  return `${minutes}м`;
}

function TpsDisplay({ tps }: { tps: number | null | undefined }) {
  if (tps == null) return <span className="text-3xl font-bold text-gray-400">N/A</span>;
  const colorClass = tps >= 19 ? "text-green-400" : tps >= 15 ? "text-yellow-400" : "text-red-400";
  return <span className={`text-3xl font-bold ${colorClass}`}>{tps.toFixed(1)}</span>;
}

export default function OnlinePage() {
  const [serverData, setServerData] = useState<ServerData | null>(null);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [error, setError] = useState(false);

  const loadServerData = useCallback(async () => {
    try {
      const res = await fetch("/api/settings/server-info");
      if (!res.ok) throw new Error("fetch failed");
      const data: ServerData = await res.json();
      setServerData(data);
      setError(false);
    } catch {
      setError(true);
      setServerData(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadServerData();
    const interval = setInterval(loadServerData, 15000);
    return () => clearInterval(interval);
  }, [loadServerData]);

  const isOnline = serverData?.status === "online" && serverData?.online === true;
  const players: Player[] = isOnline ? (serverData?.players?.list ?? []) : [];

  const filteredPlayers = searchQuery.trim()
    ? players.filter((p) =>
        (p.name ?? p.nickname ?? "").toLowerCase().includes(searchQuery.toLowerCase())
      )
    : players;

  return (
    <>
      <Header />
      <main className="relative z-10 flex-1">
        <div className="max-w-6xl mx-auto px-4 py-8 pt-24">
          {/* Заголовок */}
          <div className="text-center mb-10">
            <h1 className="text-4xl font-bold text-[#FFAA00] mb-3 text-shadow">
              <i className="fas fa-users mr-3" />
              Игроки онлайн
            </h1>
            <p className="text-gray-300 text-lg">
              Список игроков, находящихся в данный момент на сервере
            </p>
          </div>

          {/* Основная статистика */}
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
              value={loading ? "..." : error ? "—" : (isOnline ? (serverData?.players?.online ?? 0) : 0).toString()}
              label="Игроков онлайн"
              color="text-[#FFAA00]"
            />
            <StatCard
              value={loading ? "..." : error ? "—" : isOnline ? "Онлайн" : "Оффлайн"}
              label="Статус сервера"
              color={isOnline ? "text-green-400" : "text-red-400"}
            />
            <StatCard
              value={loading ? "..." : error ? "—" : isOnline ? (serverData?.server?.version ?? "Unknown") : "Оффлайн"}
              label="Версия"
              color="text-blue-400"
            />
            <div className="glass-effect rounded-xl p-6 text-center">
              {loading
                ? <span className="text-3xl font-bold text-gray-400">...</span>
                : <TpsDisplay tps={isOnline ? serverData?.performance?.tps : null} />
              }
              <div className="text-gray-300 mt-2">TPS</div>
            </div>
          </div>

          {/* Производительность */}
          <div className="grid md:grid-cols-3 gap-6 mb-8">
            <StatCard
              value={loading ? "..." : error ? "—" : isOnline ? (serverData?.performance?.ping ?? 0).toString() : "0"}
              label="Пинг (мс)"
              color="text-orange-400"
              small
            />
            <StatCard
              value={loading ? "..." : error ? "—" : (serverData?.players?.max ?? 50).toString()}
              label="Максимум игроков"
              color="text-cyan-400"
              small
            />
            <StatCard
              value={loading ? "..." : error ? "—" : formatUptime(isOnline ? serverData?.performance?.uptime_seconds : null)}
              label="Время работы"
              color="text-yellow-400"
              small
            />
          </div>

          {/* Список игроков */}
          <div className="glass-effect rounded-xl p-6">
            <h2 className="text-2xl font-semibold text-[#FFFF55] mb-6 flex items-center">
              <i className="fas fa-list mr-3" />
              Игроки в сети
            </h2>

            {/* Поиск */}
            <div className="mb-6 relative">
              <input
                type="text"
                id="player-search"
                placeholder="Поиск игрока..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full px-4 py-3 pl-12 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-[#FFAA00] transition-colors"
              />
              <i className="fas fa-search absolute left-4 top-1/2 -translate-y-1/2 text-gray-400" />
            </div>

            {/* Контент */}
            {loading ? (
              <div className="text-center py-12">
                <i className="fas fa-spinner fa-spin text-[#FFAA00] text-3xl mb-4" />
                <p className="text-gray-300">Загрузка данных сервера...</p>
              </div>
            ) : error ? (
              <div className="text-center py-12">
                <i className="fas fa-server text-red-400 text-4xl mb-4" />
                <h3 className="text-xl font-semibold text-red-400 mb-2">Нет связи с сервером</h3>
                <p className="text-gray-500">Не удалось получить данные от сервера</p>
              </div>
            ) : filteredPlayers.length === 0 ? (
              <div className="text-center py-12">
                <i className="fas fa-user-slash text-gray-500 text-4xl mb-4" />
                <h3 className="text-xl font-semibold text-gray-400 mb-2">
                  {searchQuery ? "Игрок не найден" : "Сервер пуст"}
                </h3>
                <p className="text-gray-500">
                  {searchQuery
                    ? `По запросу "${searchQuery}" ничего не найдено`
                    : "В данный момент на сервере нет игроков"}
                </p>
              </div>
            ) : (
              <>
                {searchQuery && (
                  <p className="text-gray-400 mb-4">Найдено: {filteredPlayers.length} игроков</p>
                )}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {filteredPlayers.map((player, idx) => {
                    const nickname = player.name ?? player.nickname ?? "Неизвестный";
                    const trustLevel = getTrustLevelInfo(player.trust_level ?? 0);
                    return (
                      <div
                        key={`${nickname}-${idx}`}
                        className="bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg p-4 hover:border-[#FFAA00]/50 transition-all"
                      >
                        <div className="flex items-center space-x-4">
                          {/* Аватар */}
                          <div
                            className="w-12 h-12 rounded-lg border-2 border-[#FFAA00]/30 bg-cover bg-center flex-shrink-0"
                            style={{
                              backgroundImage: `url('https://mc-heads.net/avatar/${nickname}/64')`,
                              imageRendering: "pixelated",
                            }}
                          />
                          <div className="flex-1 min-w-0">
                            <h3 className="font-bold text-white flex items-center flex-wrap">
                              <span className="truncate">{nickname}</span>
                              {getRoleBadge(player.role)}
                            </h3>
                            <div className="flex items-center space-x-2 text-sm text-gray-400 mt-1">
                              <span className="flex items-center">
                                <i className="fas fa-clock mr-1" />
                                {formatPlayTime(player.playtime ?? 0)}
                              </span>
                              <span className="text-[#FFAA00]">•</span>
                              <span className={trustLevel.color}>{trustLevel.name}</span>
                            </div>
                            <div className="text-xs text-gray-500 mt-1">
                              В сети: {getSessionTime(player.session_start)}
                            </div>
                          </div>
                          <div className="text-green-400 flex-shrink-0">
                            <i className="fas fa-circle text-xs animate-pulse" />
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </>
            )}
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

function StatCard({
  value,
  label,
  color,
  small = false,
}: {
  value: string;
  label: string;
  color: string;
  small?: boolean;
}) {
  return (
    <div className="glass-effect rounded-xl p-6 text-center">
      <div className={`${small ? "text-2xl" : "text-3xl"} font-bold ${color} mb-2`}>{value}</div>
      <div className="text-gray-300">{label}</div>
    </div>
  );
}
