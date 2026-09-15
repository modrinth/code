"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";

type TabId = "applications" | "users" | "trust-levels" | "settings" | "logs";

interface Application {
  id: number;
  user_id: number;
  status: "pending" | "approved" | "rejected";
  nickname?: string;
  email?: string;
  created_at: string;
  reason?: string;
  age?: number;
  about?: string;
}

interface AdminUser {
  id: number;
  nickname?: string;
  email: string;
  role: string;
  status: string;
  trust_level?: number;
  created_at: string;
}

interface Stats {
  totalUsers: number;
  pendingApps: number;
  activeSessions: number;
  bannedUsers: number;
}

export default function AdminPage() {
  const { user, loading: authLoading, isAuth } = useAuth({ requireAuth: true });
  const router = useRouter();

  const [activeTab, setActiveTab] = useState<TabId>("applications");
  const [stats, setStats] = useState<Stats>({ totalUsers: 0, pendingApps: 0, activeSessions: 0, bannedUsers: 0 });
  const [applications, setApplications] = useState<Application[]>([]);
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [appFilter, setAppFilter] = useState("all");
  const [userSearch, setUserSearch] = useState("");
  const [dataLoading, setDataLoading] = useState(false);
  const [actionMsg, setActionMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  const isAdmin = user?.role === "admin" || user?.role === "moderator";

  // Redirect if not admin
  useEffect(() => {
    if (!authLoading && (!isAuth || !isAdmin)) {
      if (!isAuth) router.replace("/login");
    }
  }, [authLoading, isAuth, isAdmin, router]);

  const getAuthHeaders = useCallback(() => ({
    Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
    "Content-Type": "application/json",
  }), []);

  const showMessage = useCallback((text: string, type: "success" | "error") => {
    setActionMsg({ text, type });
    setTimeout(() => setActionMsg(null), 4000);
  }, []);

  // Загрузка статистики
  useEffect(() => {
    if (!isAdmin || authLoading) return;
    async function loadStats() {
      try {
        const res = await fetch("/api/admin/stats", { headers: getAuthHeaders() });
        if (res.ok) {
          const data = await res.json();
          setStats({
            totalUsers: Number(data.users?.total_users ?? data.totalUsers ?? 0),
            pendingApps: Number(data.applications?.pending ?? data.pendingApps ?? 0),
            activeSessions: Number(data.sessions?.active_sessions ?? data.activeSessions ?? 0),
            bannedUsers: Number(data.users?.banned_users ?? data.bannedUsers ?? 0),
          });
        }
      } catch { /* ignore */ }
    }
    loadStats();
  }, [isAdmin, authLoading, getAuthHeaders]);

  // Загрузка заявок
  useEffect(() => {
    if (activeTab !== "applications" || !isAdmin) return;
    setDataLoading(true);
    async function loadApplications() {
      try {
        const url = appFilter === "all"
          ? "/api/admin/applications"
          : `/api/admin/applications?status=${appFilter}`;
        const res = await fetch(url, { headers: getAuthHeaders() });
        if (res.ok) {
          const data = await res.json();
          setApplications(data.applications ?? data ?? []);
        }
      } catch { /* ignore */ }
      setDataLoading(false);
    }
    loadApplications();
  }, [activeTab, appFilter, isAdmin, getAuthHeaders]);

  // Загрузка пользователей
  useEffect(() => {
    if (activeTab !== "users" || !isAdmin) return;
    setDataLoading(true);
    async function loadUsers() {
      try {
        const res = await fetch("/api/admin/users", { headers: getAuthHeaders() });
        if (res.ok) {
          const data = await res.json();
          setUsers(data.users ?? data ?? []);
        }
      } catch { /* ignore */ }
      setDataLoading(false);
    }
    loadUsers();
  }, [activeTab, isAdmin, getAuthHeaders]);

  async function handleApplicationAction(id: number, action: "approve" | "reject") {
    try {
      const res = await fetch(`/api/admin/applications/${id}/${action}`, {
        method: "POST",
        headers: getAuthHeaders(),
      });
      if (res.ok) {
        showMessage(action === "approve" ? "Заявка одобрена" : "Заявка отклонена", "success");
        setApplications((prev) =>
          prev.map((a) =>
            a.id === id ? { ...a, status: action === "approve" ? "approved" : "rejected" } : a
          )
        );
      } else {
        showMessage("Ошибка выполнения действия", "error");
      }
    } catch {
      showMessage("Ошибка соединения с сервером", "error");
    }
  }

  async function handleBanUser(id: number, ban: boolean) {
    try {
      const res = await fetch(`/api/admin/users/${id}/${ban ? "ban" : "unban"}`, {
        method: "POST",
        headers: getAuthHeaders(),
      });
      if (res.ok) {
        showMessage(ban ? "Пользователь заблокирован" : "Пользователь разблокирован", "success");
        setUsers((prev) =>
          prev.map((u) => u.id === id ? { ...u, status: ban ? "banned" : "active" } : u)
        );
      } else {
        showMessage("Ошибка выполнения действия", "error");
      }
    } catch {
      showMessage("Ошибка соединения с сервером", "error");
    }
  }

  const filteredUsers = users.filter(
    (u) =>
      !userSearch ||
      (u.nickname ?? "").toLowerCase().includes(userSearch.toLowerCase()) ||
      u.email.toLowerCase().includes(userSearch.toLowerCase())
  );

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <i className="fas fa-spinner fa-spin text-[#FFAA00] text-4xl" />
      </div>
    );
  }

  if (!isAdmin && !authLoading) {
    return (
      <>
        <Header />
        <main className="relative z-10 flex-1 pt-24 pb-12">
          <div className="max-w-md mx-auto px-4">
            <div className="glass-effect rounded-xl p-8 text-center border border-red-500/30">
              <i className="fas fa-lock text-red-400 text-5xl mb-4 block" />
              <h1 className="text-2xl font-bold text-white mb-3">Доступ запрещён</h1>
              <p className="text-gray-300 mb-6">У вас нет прав администратора</p>
              <button
                onClick={() => router.push("/profile")}
                className="btn-minecraft px-6 py-3 rounded-lg font-bold hover:scale-105 transition-all"
              >
                Вернуться в профиль
              </button>
            </div>
          </div>
        </main>
        <Footer />
      </>
    );
  }

  return (
    <>
      <Header />
      <main className="relative z-10 flex-1 pt-20 pb-12">
        <div className="max-w-7xl mx-auto px-4 py-8">
          {/* Заголовок */}
          <div className="flex items-center justify-between mb-8">
            <div>
              <h1 className="text-3xl font-bold text-white flex items-center gap-3">
                <i className="fas fa-crown text-[#FFAA00]" />
                Панель администратора
              </h1>
              <p className="text-gray-400 mt-1">Управление сервером и пользователями</p>
            </div>
            <div className="flex items-center gap-2 bg-red-500/20 border border-red-500/40 px-4 py-2 rounded-lg">
              <i className="fas fa-shield-alt text-red-400" />
              <span className="text-red-300 font-bold text-sm uppercase">
                {user?.role === "admin" ? "Admin" : "Moderator"}
              </span>
            </div>
          </div>

          {/* Сообщение о результате действия */}
          {actionMsg && (
            <div
              className={`mb-6 p-4 rounded-xl border flex items-center gap-3 ${
                actionMsg.type === "success"
                  ? "bg-green-900/50 text-green-300 border-green-500/50"
                  : "bg-red-900/50 text-red-300 border-red-500/50"
              }`}
            >
              <i className={`fas ${actionMsg.type === "success" ? "fa-check-circle" : "fa-exclamation-circle"}`} />
              {actionMsg.text}
            </div>
          )}

          {/* Статистика */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
            {[
              { icon: "fa-users", color: "text-[#FFAA00]", bg: "bg-[#FFAA00]/20", value: stats.totalUsers, label: "Пользователей" },
              { icon: "fa-file-alt", color: "text-green-400", bg: "bg-green-500/20", value: stats.pendingApps, label: "Заявок ожидает" },
              { icon: "fa-clock", color: "text-yellow-400", bg: "bg-yellow-500/20", value: stats.activeSessions, label: "Активных сессий" },
              { icon: "fa-ban", color: "text-red-400", bg: "bg-red-500/20", value: stats.bannedUsers, label: "Заблокированных" },
            ].map(({ icon, color, bg, value, label }) => (
              <div
                key={label}
                className="glass-effect rounded-xl p-5 hover:-translate-y-1 transition-transform"
              >
                <div className="flex items-center gap-4">
                  <div className={`w-11 h-11 ${bg} rounded-lg flex items-center justify-center flex-shrink-0`}>
                    <i className={`fas ${icon} ${color} text-xl`} />
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-white">{value}</div>
                    <div className="text-xs text-gray-400">{label}</div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Вкладки */}
          <div className="glass-effect rounded-xl">
            <div className="border-b border-[#FFAA00]/20">
              <nav className="flex overflow-x-auto px-4">
                {(
                  [
                    { id: "applications", icon: "fa-file-alt", label: "Заявки" },
                    { id: "users", icon: "fa-users", label: "Пользователи" },
                    { id: "trust-levels", icon: "fa-medal", label: "Trust Level" },
                    { id: "settings", icon: "fa-cog", label: "Настройки" },
                    { id: "logs", icon: "fa-list", label: "Логи" },
                  ] as { id: TabId; icon: string; label: string }[]
                ).map(({ id, icon, label }) => (
                  <button
                    key={id}
                    id={`tab-${id}`}
                    onClick={() => setActiveTab(id)}
                    className={`flex items-center gap-2 py-4 px-4 border-b-2 font-medium whitespace-nowrap transition-colors ${
                      activeTab === id
                        ? "border-[#FFAA00] text-[#FFAA00]"
                        : "border-transparent text-gray-400 hover:text-[#FFAA00]"
                    }`}
                  >
                    <i className={`fas ${icon}`} />
                    {label}
                  </button>
                ))}
              </nav>
            </div>

            <div className="p-6">
              {/* Заявки */}
              {activeTab === "applications" && (
                <div>
                  <div className="flex justify-between items-center mb-6">
                    <h2 className="text-lg font-semibold text-white">Заявки на whitelist</h2>
                    <select
                      value={appFilter}
                      onChange={(e) => setAppFilter(e.target.value)}
                      className="px-4 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
                    >
                      <option value="all">Все заявки</option>
                      <option value="pending">На рассмотрении</option>
                      <option value="approved">Одобренные</option>
                      <option value="rejected">Отклонённые</option>
                    </select>
                  </div>

                  {dataLoading ? (
                    <div className="text-center py-12">
                      <i className="fas fa-spinner fa-spin text-[#FFAA00] text-3xl mb-3" />
                      <p className="text-gray-400">Загрузка заявок...</p>
                    </div>
                  ) : applications.length === 0 ? (
                    <div className="text-center py-12">
                      <i className="fas fa-inbox text-gray-500 text-4xl mb-3" />
                      <p className="text-gray-400">Нет заявок</p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {applications.map((app) => (
                        <div
                          key={app.id}
                          className="bg-[#2a2a2a]/50 border border-[#FFAA00]/10 rounded-lg p-5 hover:border-[#FFAA00]/30 transition-colors"
                        >
                          <div className="flex items-start justify-between gap-4">
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-3 flex-wrap mb-2">
                                <span className="font-bold text-white">{app.nickname ?? "Неизвестный"}</span>
                                <span className="text-gray-400 text-sm">{app.email}</span>
                                <StatusBadge status={app.status} />
                              </div>
                              {app.about && (
                                <p className="text-gray-400 text-sm mt-2 line-clamp-2">{app.about}</p>
                              )}
                              <p className="text-gray-600 text-xs mt-2">
                                {new Date(app.created_at).toLocaleString("ru-RU")}
                              </p>
                            </div>
                            {app.status === "pending" && (
                              <div className="flex gap-2 flex-shrink-0">
                                <button
                                  onClick={() => handleApplicationAction(app.id, "approve")}
                                  className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium text-sm transition-colors"
                                >
                                  <i className="fas fa-check mr-1" />
                                  Одобрить
                                </button>
                                <button
                                  onClick={() => handleApplicationAction(app.id, "reject")}
                                  className="px-4 py-2 bg-red-700 hover:bg-red-600 text-white rounded-lg font-medium text-sm transition-colors"
                                >
                                  <i className="fas fa-times mr-1" />
                                  Отклонить
                                </button>
                              </div>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {/* Пользователи */}
              {activeTab === "users" && (
                <div>
                  <div className="flex justify-between items-center mb-6 gap-4 flex-wrap">
                    <h2 className="text-lg font-semibold text-white">Управление пользователями</h2>
                    <input
                      type="text"
                      placeholder="Поиск по нику или email..."
                      value={userSearch}
                      onChange={(e) => setUserSearch(e.target.value)}
                      className="px-4 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-[#FFAA00]"
                    />
                  </div>
                  {dataLoading ? (
                    <div className="text-center py-12">
                      <i className="fas fa-spinner fa-spin text-[#FFAA00] text-3xl mb-3" />
                      <p className="text-gray-400">Загрузка пользователей...</p>
                    </div>
                  ) : filteredUsers.length === 0 ? (
                    <div className="text-center py-12">
                      <i className="fas fa-users-slash text-gray-500 text-4xl mb-3" />
                      <p className="text-gray-400">
                        {userSearch ? "Пользователи не найдены" : "Нет пользователей"}
                      </p>
                    </div>
                  ) : (
                    <div className="overflow-x-auto">
                      <table className="w-full text-sm">
                        <thead>
                          <tr className="text-gray-400 border-b border-[#FFAA00]/10">
                            <th className="text-left py-3 px-4">Пользователь</th>
                            <th className="text-left py-3 px-4">Роль</th>
                            <th className="text-left py-3 px-4">Статус</th>
                            <th className="text-left py-3 px-4">Trust</th>
                            <th className="text-left py-3 px-4">Дата регистрации</th>
                            <th className="text-right py-3 px-4">Действия</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-[#FFAA00]/5">
                          {filteredUsers.map((u) => (
                            <tr key={u.id} className="hover:bg-[#FFAA00]/5 transition-colors">
                              <td className="py-3 px-4">
                                <div>
                                  <div className="font-medium text-white">{u.nickname ?? "—"}</div>
                                  <div className="text-gray-500 text-xs">{u.email}</div>
                                </div>
                              </td>
                              <td className="py-3 px-4">
                                <RoleBadge role={u.role} />
                              </td>
                              <td className="py-3 px-4">
                                <span
                                  className={`px-2 py-0.5 rounded text-xs font-medium ${
                                    u.status === "banned"
                                      ? "bg-red-500/20 text-red-400"
                                      : "bg-green-500/20 text-green-400"
                                  }`}
                                >
                                  {u.status === "banned" ? "Заблокирован" : "Активен"}
                                </span>
                              </td>
                              <td className="py-3 px-4 text-gray-300">{u.trust_level ?? 0}</td>
                              <td className="py-3 px-4 text-gray-500 text-xs">
                                {new Date(u.created_at).toLocaleDateString("ru-RU")}
                              </td>
                              <td className="py-3 px-4 text-right">
                                {u.role !== "admin" && (
                                  <button
                                    onClick={() => handleBanUser(u.id, u.status !== "banned")}
                                    className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
                                      u.status === "banned"
                                        ? "bg-green-700 hover:bg-green-600 text-white"
                                        : "bg-red-700 hover:bg-red-600 text-white"
                                    }`}
                                  >
                                    {u.status === "banned" ? (
                                      <><i className="fas fa-unlock mr-1" />Разблокировать</>
                                    ) : (
                                      <><i className="fas fa-ban mr-1" />Заблокировать</>
                                    )}
                                  </button>
                                )}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              )}

              {/* Trust Level */}
              {activeTab === "trust-levels" && (
                <div>
                  <h2 className="text-lg font-semibold text-white mb-6">Система Trust Level</h2>
                  <div className="grid md:grid-cols-2 gap-6">
                    <div className="glass-effect rounded-lg p-6">
                      <h3 className="font-semibold text-white mb-4">
                        <i className="fas fa-info-circle mr-2 text-[#FFAA00]" />
                        Уровни доверия
                      </h3>
                      <div className="space-y-3 text-sm">
                        {[
                          { level: 0, emoji: "🚶", name: "Проходимец", desc: "Лимит 10 часов до верификации почты" },
                          { level: 1, emoji: "👤", name: "Новичок", desc: "Стандартный уровень после верификации" },
                          { level: 2, emoji: "✅", name: "Проверенный", desc: "25ч + верификация + 10+ репутации" },
                          { level: 3, emoji: "🏆", name: "Ветеран", desc: "50ч + верификация + 20+ репутации" },
                          { level: 4, emoji: "👑", name: "Легенда", desc: "Выдаётся вручную администратором" },
                        ].map(({ level, emoji, name, desc }) => (
                          <div key={level} className="flex items-start gap-3 p-3 rounded-lg bg-[#2a2a2a]/50">
                            <span className="text-xl">{emoji}</span>
                            <div>
                              <div className="text-white font-medium">
                                Уровень {level}: {name}
                              </div>
                              <div className="text-gray-400 text-xs mt-0.5">{desc}</div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                    <div className="glass-effect rounded-lg p-6">
                      <h3 className="font-semibold text-white mb-4">
                        <i className="fas fa-chart-bar mr-2 text-[#FFAA00]" />
                        Инструкция
                      </h3>
                      <div className="text-sm text-gray-300 space-y-3">
                        <p>Для изменения Trust Level пользователя:</p>
                        <ol className="list-decimal list-inside space-y-2 text-gray-400">
                          <li>Перейдите на вкладку &ldquo;Пользователи&rdquo;</li>
                          <li>Найдите нужного пользователя</li>
                          <li>Откройте его профиль</li>
                          <li>Измените уровень доверия вручную</li>
                        </ol>
                        <div className="mt-4 p-3 rounded-lg bg-orange-900/20 border border-orange-500/30">
                          <i className="fas fa-exclamation-triangle text-orange-400 mr-2" />
                          <span className="text-orange-300 text-xs">
                            Изменение Trust Level влияет на доступ пользователя к функциям сервера
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Настройки */}
              {activeTab === "settings" && (
                <AdminSettingsTab getAuthHeaders={getAuthHeaders} showMessage={showMessage} />
              )}

              {/* Логи */}
              {activeTab === "logs" && (
                <AdminLogsTab getAuthHeaders={getAuthHeaders} />
              )}
            </div>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

function AdminSettingsTab({
  getAuthHeaders,
  showMessage,
}: {
  getAuthHeaders: () => Record<string, string>;
  showMessage: (text: string, type: "success" | "error") => void;
}) {
  const [settings, setSettings] = useState<Record<string, string>>({
    serverName: "",
    serverDescription: "",
    serverIp: "",
    serverPort: "",
    maxPlayers: "",
    discordInvite: "",
    telegramInvite: "",
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const fields = [
    { key: "serverName", label: "Название сервера" },
    { key: "serverDescription", label: "Описание" },
    { key: "serverIp", label: "IP / хост" },
    { key: "serverPort", label: "Порт" },
    { key: "maxPlayers", label: "Макс. игроков" },
    { key: "discordInvite", label: "Discord invite" },
    { key: "telegramInvite", label: "Telegram" },
  ];

  useEffect(() => {
    async function load() {
      try {
        const res = await fetch("/api/admin/settings", { headers: getAuthHeaders() });
        if (res.ok) {
          const data = await res.json();
          const s = data.settings || {};
          setSettings({
            serverName: String(s.serverName ?? ""),
            serverDescription: String(s.serverDescription ?? ""),
            serverIp: String(s.serverIp ?? ""),
            serverPort: String(s.serverPort ?? ""),
            maxPlayers: String(s.serverMaxPlayers ?? s.maxPlayers ?? ""),
            discordInvite: String(s.discordInvite ?? ""),
            telegramInvite: String(s.telegramInvite ?? ""),
          });
        } else {
          const pub = await fetch("/api/settings/public");
          if (pub.ok) {
            const p = await pub.json();
            setSettings({
              serverName: p.serverName || "",
              serverDescription: p.serverDescription || "",
              serverIp: p.serverIp || "",
              serverPort: String(p.serverPort || ""),
              maxPlayers: "",
              discordInvite: p.discordInvite || "",
              telegramInvite: p.telegramInvite || "",
            });
          }
        }
      } catch { /* ignore */ }
      setLoading(false);
    }
    load();
  }, [getAuthHeaders]);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    try {
      const body: Record<string, unknown> = {
        serverName: settings.serverName,
        serverDescription: settings.serverDescription,
        serverIp: settings.serverIp,
        discordInvite: settings.discordInvite,
        telegramInvite: settings.telegramInvite,
      };
      if (settings.serverPort) body.serverPort = parseInt(settings.serverPort, 10);
      if (settings.maxPlayers) body.maxPlayers = parseInt(settings.maxPlayers, 10);

      const res = await fetch("/api/admin/settings", {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify(body),
      });
      if (res.ok) {
        showMessage("Настройки сохранены", "success");
      } else {
        showMessage("Не удалось сохранить настройки", "error");
      }
    } catch {
      showMessage("Ошибка соединения", "error");
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return <p className="text-gray-400">Загрузка настроек...</p>;
  }

  return (
    <div>
      <h2 className="text-lg font-semibold text-white mb-6">Настройки сервера</h2>
      <form onSubmit={save} className="space-y-4 max-w-2xl">
        {fields.map((f) => (
          <label key={f.key} className="block text-sm">
            <span className="text-gray-400">{f.label}</span>
            <input
              value={settings[f.key] ?? ""}
              onChange={(e) => setSettings((s) => ({ ...s, [f.key]: e.target.value }))}
              className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
            />
          </label>
        ))}
        <button
          type="submit"
          disabled={saving}
          className="btn-minecraft px-6 py-2 rounded-lg font-bold disabled:opacity-60"
        >
          {saving ? "Сохранение..." : "Сохранить"}
        </button>
      </form>
    </div>
  );
}

function AdminLogsTab({ getAuthHeaders }: { getAuthHeaders: () => Record<string, string> }) {
  const [logs, setLogs] = useState<Array<{
    id: number;
    action: string;
    details?: string;
    created_at: string;
    admin_nickname?: string;
    target_user_nickname?: string;
  }>>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);

  useEffect(() => {
    setLoading(true);
    async function load() {
      try {
        const res = await fetch(`/api/admin/logs?page=${page}&limit=30`, {
          headers: getAuthHeaders(),
        });
        if (res.ok) {
          const data = await res.json();
          setLogs(data.logs ?? []);
          setTotalPages(data.pagination?.totalPages ?? 1);
        }
      } catch { /* ignore */ }
      setLoading(false);
    }
    load();
  }, [page, getAuthHeaders]);

  return (
    <div>
      <h2 className="text-lg font-semibold text-white mb-6">Системные логи</h2>
      {loading ? (
        <p className="text-gray-400">Загрузка...</p>
      ) : logs.length === 0 ? (
        <p className="text-gray-500 text-center py-10">Логов пока нет</p>
      ) : (
        <div className="overflow-x-auto rounded-xl border border-[#FFAA00]/20">
          <table className="w-full text-sm">
            <thead className="bg-[#2a2a2a] text-gray-400">
              <tr>
                <th className="text-left px-4 py-3">Время</th>
                <th className="text-left px-4 py-3">Админ</th>
                <th className="text-left px-4 py-3">Действие</th>
                <th className="text-left px-4 py-3">Детали</th>
                <th className="text-left px-4 py-3">Цель</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr key={log.id} className="border-t border-[#FFAA00]/10 text-gray-300">
                  <td className="px-4 py-2 whitespace-nowrap">
                    {new Date(log.created_at).toLocaleString("ru-RU")}
                  </td>
                  <td className="px-4 py-2">{log.admin_nickname || "—"}</td>
                  <td className="px-4 py-2 text-[#FFAA00]">{log.action}</td>
                  <td className="px-4 py-2 max-w-xs truncate">{log.details || "—"}</td>
                  <td className="px-4 py-2">{log.target_user_nickname || "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {totalPages > 1 && (
        <div className="flex gap-2 mt-4">
          <button
            disabled={page <= 1}
            onClick={() => setPage((p) => p - 1)}
            className="px-3 py-1 rounded bg-[#2a2a2a] text-white disabled:opacity-40"
          >
            Назад
          </button>
          <span className="text-gray-400 self-center text-sm">
            {page} / {totalPages}
          </span>
          <button
            disabled={page >= totalPages}
            onClick={() => setPage((p) => p + 1)}
            className="px-3 py-1 rounded bg-[#2a2a2a] text-white disabled:opacity-40"
          >
            Далее
          </button>
        </div>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const styles: Record<string, string> = {
    pending: "bg-yellow-500/20 text-yellow-400",
    approved: "bg-green-500/20 text-green-400",
    rejected: "bg-red-500/20 text-red-400",
  };
  const labels: Record<string, string> = {
    pending: "На рассмотрении",
    approved: "Одобрена",
    rejected: "Отклонена",
  };
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-medium ${styles[status] ?? "bg-gray-500/20 text-gray-400"}`}>
      {labels[status] ?? status}
    </span>
  );
}

function RoleBadge({ role }: { role: string }) {
  const styles: Record<string, string> = {
    admin: "bg-red-500/20 text-red-400",
    moderator: "bg-blue-500/20 text-blue-400",
    user: "bg-gray-500/20 text-gray-400",
  };
  const labels: Record<string, string> = {
    admin: "Admin",
    moderator: "Moder",
    user: "User",
  };
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-medium ${styles[role] ?? "bg-gray-500/20 text-gray-400"}`}>
      {labels[role] ?? role}
    </span>
  );
}
