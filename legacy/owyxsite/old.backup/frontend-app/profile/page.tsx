"use client";

import Link from "next/link";
import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";

// ==== Trust Level helpers ====
function getTrustLevelInfo(level: number) {
  const levels: Record<number, { name: string; color: string; textColor: string }> = {
    0: { name: "Проходимец", color: "text-gray-400", textColor: "text-gray-400" },
    1: { name: "Новичок", color: "text-blue-400", textColor: "text-blue-400" },
    2: { name: "Проверенный", color: "text-green-400", textColor: "text-green-400" },
    3: { name: "Ветеран", color: "text-[#FFAA00]", textColor: "text-[#FFAA00]" },
    4: { name: "Легенда", color: "text-purple-400", textColor: "text-purple-400" },
  };
  return levels[level] ?? levels[0];
}

function formatPlaytime(minutes: number): string {
  if (minutes < 60) return `${minutes} мин`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} ч`;
  const days = Math.floor(hours / 24);
  return `${days} дн`;
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return "Неизвестно";
  return new Date(dateStr).toLocaleDateString("ru-RU", {
    day: "2-digit", month: "long", year: "numeric",
  });
}

function getRoleBadge(role?: string) {
  if (role === "admin") return <span className="ml-2 px-2 py-0.5 text-xs bg-red-600 text-white rounded font-bold">ADMIN</span>;
  if (role === "moderator") return <span className="ml-2 px-2 py-0.5 text-xs bg-blue-600 text-white rounded font-bold">MOD</span>;
  return null;
}

// ==== Stat Card ====
function StatCard({ icon, value, label, color }: { icon: React.ReactNode; value: string; label: string; color: string }) {
  return (
    <div className="glass-effect rounded-xl p-5 flex items-center gap-4 hover:-translate-y-1 transition-transform duration-300">
      <div className={`w-12 h-12 rounded-lg flex items-center justify-center shrink-0 ${color}`}>
        {icon}
      </div>
      <div>
        <div className="text-xl font-bold text-white">{value}</div>
        <div className="text-sm text-gray-400">{label}</div>
      </div>
    </div>
  );
}

// ==== Profile section — вкладка «Информация» ====
function InfoTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  if (!user) return null;
  const trust = getTrustLevelInfo(user.trust_level ?? 0);
  const trustMax = [10, 50, 200, 500];
  const nextThreshold = trustMax[(user.trust_level ?? 0)] ?? 999;
  // Используем reputation (если придёт) или заглушку
  const reputation = 0; // будет заполнено из API профиля
  const progress = Math.min(100, (reputation / nextThreshold) * 100);

  const daysWithUs = user.created_at
    ? Math.floor((Date.now() - new Date(user.created_at).getTime()) / (1000 * 60 * 60 * 24))
    : 0;

  return (
    <div className="space-y-8">
      {/* Карточка игрока */}
      <div className="glass-effect rounded-2xl p-8">
        <div className="flex flex-col sm:flex-row items-center sm:items-start gap-6">
          {/* Аватар */}
          <div className="relative shrink-0">
            <div className="w-24 h-24 rounded-full overflow-hidden border-4 border-[#FFAA00] shadow-lg shadow-[#FFAA00]/30">
              <img
                src={`https://crafatar.com/avatars/${encodeURIComponent(user.nickname || "Steve")}?size=96&overlay`}
                alt="Minecraft аватар"
                className="w-full h-full object-cover pixelated"
                onError={(e) => {
                  (e.target as HTMLImageElement).src = "/images/default-avatar.png";
                }}
              />
            </div>
            <div className="absolute -bottom-1 -right-1 w-5 h-5 bg-green-500 rounded-full border-2 border-[#1a1a1a]" />
          </div>

          {/* Данные */}
          <div className="flex-1 text-center sm:text-left">
            <div className="flex items-center justify-center sm:justify-start flex-wrap gap-2 mb-1">
              <h1 className="text-3xl font-bold text-[#FFFF55] text-shadow">
                {user.nickname || user.email || "Игрок"}
              </h1>
              {getRoleBadge(user.role)}
            </div>
            <p className={`font-medium mb-3 ${trust.textColor}`}>{trust.name}</p>
            {/* Trust progress */}
            <div className="max-w-xs mx-auto sm:mx-0">
              <div className="w-full bg-[#2a2a2a] rounded-full h-2">
                <div
                  className="h-2 rounded-full bg-gradient-to-r from-red-500 via-[#FFAA00] to-green-400 transition-all duration-500"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-xs text-gray-400 mt-1">{reputation} / {nextThreshold} репутации</p>
            </div>
          </div>

          {/* Статус + дата */}
          <div className="text-center sm:text-right shrink-0">
            <div className="text-xs text-gray-500 mb-1">Статус</div>
            <div className={`font-bold ${user.status === "banned" ? "text-red-400" : "text-green-400"}`}>
              {user.status === "banned" ? "Заблокирован" : "Активен"}
            </div>
            <div className="text-xs text-gray-500 mt-3">Регистрация</div>
            <div className="text-sm text-gray-300">{formatDate(user.created_at)}</div>
          </div>
        </div>
      </div>

      {/* Быстрая статистика */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <StatCard
          icon={<svg className="w-6 h-6 text-[#FFAA00]" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>}
          value="—"
          label="Время игры"
          color="bg-[#FFAA00]/20"
        />
        <StatCard
          icon={<svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" /></svg>}
          value={String(reputation)}
          label="Репутация"
          color="bg-green-500/20"
        />
        <StatCard
          icon={<svg className="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" /></svg>}
          value="—"
          label="Входов"
          color="bg-blue-500/20"
        />
        <StatCard
          icon={<svg className="w-6 h-6 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>}
          value={`${daysWithUs}`}
          label="Дней с нами"
          color="bg-yellow-500/20"
        />
      </div>

      {/* Основная информация */}
      <div className="glass-effect rounded-2xl p-6">
        <h3 className="text-lg font-bold text-[#FFAA00] mb-5 flex items-center gap-2">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V8a2 2 0 00-2-2h-5m-4 0V5a2 2 0 114 0v1m-4 0a2 2 0 104 0m-5 8a2 2 0 100-4 2 2 0 000 4zm0 0c1.306 0 2.417.835 2.83 2M9 14a3.001 3.001 0 00-2.83 2M15 11h3m-3 4h2" />
          </svg>
          Основная информация
        </h3>
        <div className="space-y-3">
          {[
            { label: "Имя", value: user.first_name || "—" },
            { label: "Minecraft ник", value: user.nickname || "—" },
            { label: "Email", value: user.email || "—" },
            { label: "Возраст", value: user.age ? `${user.age} лет` : "Не указан" },
            { label: "Discord", value: user.discord || "Не указан" },
            { label: "Дата регистрации", value: formatDate(user.created_at) },
          ].map(({ label, value }) => (
            <div key={label} className="flex justify-between items-center py-2 border-b border-[#404040] last:border-0">
              <span className="text-gray-400 text-sm">{label}</span>
              <span className="text-white font-medium text-sm">{value}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Уровень доверия */}
      <div className="glass-effect rounded-2xl p-6">
        <h3 className="text-lg font-bold text-[#FFAA00] mb-5 flex items-center gap-2">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
          </svg>
          Уровень доверия
        </h3>
        <div className="text-center mb-6">
          <div className="text-5xl font-bold text-[#FFAA00] mb-1">{user.trust_level ?? 0}</div>
          <div className={`text-lg font-medium ${trust.textColor}`}>{trust.name}</div>
        </div>
        <div className="w-full bg-[#2a2a2a] rounded-full h-3 mb-2">
          <div
            className="h-3 rounded-full bg-gradient-to-r from-red-500 via-[#FFAA00] to-green-400 transition-all duration-700"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="flex justify-between text-sm text-gray-400">
          <span>Репутация: <span className="text-[#FFAA00] font-bold">{reputation}</span></span>
          <span>До следующего: <span className="text-[#FFAA00] font-bold">{nextThreshold}</span></span>
        </div>
        <div className="mt-4 bg-[#0a0a0a]/50 rounded-lg p-3 text-sm text-gray-400">
          Привилегии уровня: <span className="text-[#FFAA00]">Базовые права пользователя</span>
        </div>
        <div className="mt-4 text-center">
          <Link
            href="/profile/application"
            className="inline-flex items-center gap-2 text-[#FFAA00] hover:text-[#FFFF55] transition-colors text-sm font-medium"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
            </svg>
            Подать заявку на повышение →
          </Link>
        </div>
      </div>
    </div>
  );
}

// ==== Tab button ====
function TabBtn({ active, onClick, icon, label }: { active: boolean; onClick: () => void; icon: React.ReactNode; label: string }) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 py-4 px-1 border-b-2 font-medium text-sm transition-all duration-200 whitespace-nowrap ${
        active
          ? "border-[#FFAA00] text-[#FFAA00]"
          : "border-transparent text-gray-400 hover:text-[#FFAA00] hover:border-[#FFAA00]/50"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

// ==== Main component ====
export default function ProfilePage() {
  const { user, loading } = useAuth({ requireAuth: true });
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<"info" | "application" | "token" | "settings">("info");

  // Logout
  function handleLogout() {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("remember_me");
    localStorage.removeItem("token_expires");
    router.replace("/login");
  }

  // Loading state
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <svg className="animate-spin w-12 h-12 text-[#FFAA00] mx-auto mb-4" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          <p className="text-gray-400">Загрузка профиля...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <Header />

      <main className="min-h-[calc(100vh-64px)] py-8 px-4">
        <div className="max-w-5xl mx-auto">

          {/* Хлебные крошки */}
          <div className="flex items-center gap-2 text-sm text-gray-500 mb-6">
            <Link href="/" className="hover:text-[#FFAA00] transition-colors">Главная</Link>
            <span>/</span>
            <span className="text-[#FFAA00]">Личный кабинет</span>
          </div>

          {/* Заголовок страницы */}
          <div className="flex items-center justify-between mb-8">
            <h1 className="text-3xl font-bold text-[#FFFF55] text-shadow">Личный кабинет</h1>
            <button
              onClick={handleLogout}
              className="flex items-center gap-2 text-red-400 hover:text-red-300 text-sm transition-colors border border-red-400/30 px-4 py-2 rounded-lg hover:bg-red-400/10"
              id="profile-logout-btn"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
              Выйти
            </button>
          </div>

          {/* Tabs */}
          <div className="glass-effect rounded-2xl overflow-hidden mb-8">
            <div className="border-b border-[#FFAA00]/20 px-6 overflow-x-auto">
              <nav className="flex gap-6">
                <TabBtn
                  active={activeTab === "info"}
                  onClick={() => setActiveTab("info")}
                  label="Информация"
                  icon={<svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>}
                />
                <TabBtn
                  active={activeTab === "application"}
                  onClick={() => setActiveTab("application")}
                  label="Заявка"
                  icon={<svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>}
                />
                <TabBtn
                  active={activeTab === "token"}
                  onClick={() => setActiveTab("token")}
                  label="Игровой токен"
                  icon={<svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>}
                />
                <TabBtn
                  active={activeTab === "settings"}
                  onClick={() => setActiveTab("settings")}
                  label="Настройки"
                  icon={<svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>}
                />
              </nav>
            </div>

            {/* Содержимое вкладок */}
            <div className="p-6">
              {activeTab === "info" && <InfoTab user={user} />}

              {activeTab === "application" && <ApplicationTab user={user} />}

              {activeTab === "token" && <GameTokenTab />}

              {activeTab === "settings" && <SettingsTab user={user} />}
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </>
  );
}

// ==== Application Tab ====
function ApplicationTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const [application, setApplication] = useState<{
    id: number;
    minecraft_nick: string;
    status: string;
    submitted_at: string;
    reviewed_at?: string;
    review_comment?: string;
    experience?: string;
    motivation?: string;
    plans?: string;
    discord?: string;
    age?: string | number;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [form, setForm] = useState({
    minecraft_nick: user?.nickname || "",
    age: user?.age ? String(user.age) : "",
    discord: user?.discord || "",
    email: user?.email || "",
    experience: "intermediate",
    motivation: "",
    plans: "",
  });

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const res = await fetch("/api/applications/mine", {
          headers: { Authorization: `Bearer ${localStorage.getItem("auth_token")}` },
        });
        if (res.ok) {
          const data = await res.json();
          if (!cancelled) setApplication(data.application ?? null);
        }
      } catch { /* ignore */ }
      if (!cancelled) setLoading(false);
    }
    load();
    return () => { cancelled = true; };
  }, []);

  async function submitApplication(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setMsg(null);
    try {
      const res = await fetch("/api/applications", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          ...form,
          age: parseInt(form.age, 10),
        }),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setMsg({ text: data.message || "Заявка отправлена", type: "success" });
        const mine = await fetch("/api/applications/mine", {
          headers: { Authorization: `Bearer ${localStorage.getItem("auth_token")}` },
        });
        if (mine.ok) {
          const mineData = await mine.json();
          setApplication(mineData.application ?? null);
        }
      } else {
        setMsg({ text: data.error || "Не удалось отправить заявку", type: "error" });
      }
    } catch {
      setMsg({ text: "Ошибка соединения с сервером", type: "error" });
    } finally {
      setSaving(false);
    }
  }

  const statusLabel: Record<string, string> = {
    pending: "На рассмотрении",
    approved: "Одобрена",
    rejected: "Отклонена",
  };
  const statusColor: Record<string, string> = {
    pending: "text-yellow-400",
    approved: "text-green-400",
    rejected: "text-red-400",
  };

  if (loading) {
    return <p className="text-gray-400 text-center py-10">Загрузка заявки...</p>;
  }

  if (application) {
    return (
      <div className="space-y-6">
        <div className="glass-effect rounded-xl p-6 border border-[#FFAA00]/20">
          <h3 className="text-lg font-bold text-[#FFAA00] mb-4">Ваша заявка</h3>
          <div className="grid sm:grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-500">Ник</span>
              <p className="text-white font-medium">{application.minecraft_nick}</p>
            </div>
            <div>
              <span className="text-gray-500">Статус</span>
              <p className={`font-bold ${statusColor[application.status] || "text-white"}`}>
                {statusLabel[application.status] || application.status}
              </p>
            </div>
            <div>
              <span className="text-gray-500">Отправлена</span>
              <p className="text-white">{formatDate(application.submitted_at)}</p>
            </div>
            {application.reviewed_at && (
              <div>
                <span className="text-gray-500">Рассмотрена</span>
                <p className="text-white">{formatDate(application.reviewed_at)}</p>
              </div>
            )}
          </div>
          {application.review_comment && (
            <div className="mt-4 p-3 rounded-lg bg-[#0a0a0a]/50 text-gray-300 text-sm">
              Комментарий: {application.review_comment}
            </div>
          )}
          {application.status === "rejected" && (
            <p className="mt-4 text-sm text-gray-400">
              Вы можете подать новую заявку после отклонения — обновите форму ниже.
            </p>
          )}
        </div>
        {application.status === "rejected" && (
          <ApplicationForm
            form={form}
            setForm={setForm}
            saving={saving}
            msg={msg}
            onSubmit={submitApplication}
          />
        )}
      </div>
    );
  }

  return (
    <ApplicationForm
      form={form}
      setForm={setForm}
      saving={saving}
      msg={msg}
      onSubmit={submitApplication}
    />
  );
}

function ApplicationForm({
  form,
  setForm,
  saving,
  msg,
  onSubmit,
}: {
  form: {
    minecraft_nick: string;
    age: string;
    discord: string;
    email: string;
    experience: string;
    motivation: string;
    plans: string;
  };
  setForm: React.Dispatch<React.SetStateAction<typeof form>>;
  saving: boolean;
  msg: { text: string; type: "success" | "error" } | null;
  onSubmit: (e: React.FormEvent) => void;
}) {
  return (
    <form onSubmit={onSubmit} className="space-y-4 max-w-2xl">
      <h3 className="text-lg font-bold text-[#FFAA00] mb-2">Заявка на whitelist</h3>
      <p className="text-gray-400 text-sm mb-4">
        Расскажите о себе — администрация рассмотрит заявку в ближайшее время.
      </p>
      {msg && (
        <div className={`p-3 rounded-lg text-sm ${msg.type === "success" ? "bg-green-900/40 text-green-300 border border-green-500/30" : "bg-red-900/40 text-red-300 border border-red-500/30"}`}>
          {msg.text}
        </div>
      )}
      <div className="grid sm:grid-cols-2 gap-4">
        <label className="block text-sm">
          <span className="text-gray-400">Minecraft ник</span>
          <input
            required
            value={form.minecraft_nick}
            onChange={(e) => setForm((f) => ({ ...f, minecraft_nick: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
        <label className="block text-sm">
          <span className="text-gray-400">Возраст</span>
          <input
            required
            type="number"
            min={10}
            max={100}
            value={form.age}
            onChange={(e) => setForm((f) => ({ ...f, age: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
        <label className="block text-sm">
          <span className="text-gray-400">Discord</span>
          <input
            required
            value={form.discord}
            onChange={(e) => setForm((f) => ({ ...f, discord: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
        <label className="block text-sm">
          <span className="text-gray-400">Email</span>
          <input
            required
            type="email"
            value={form.email}
            onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
      </div>
      <label className="block text-sm">
        <span className="text-gray-400">Опыт</span>
        <select
          value={form.experience}
          onChange={(e) => setForm((f) => ({ ...f, experience: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        >
          <option value="beginner">Новичок</option>
          <option value="intermediate">Средний</option>
          <option value="advanced">Продвинутый</option>
          <option value="expert">Эксперт</option>
        </select>
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">Мотивация (мин. 50 символов)</span>
        <textarea
          required
          minLength={50}
          maxLength={800}
          rows={4}
          value={form.motivation}
          onChange={(e) => setForm((f) => ({ ...f, motivation: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">Планы на сервере (мин. 30 символов)</span>
        <textarea
          required
          minLength={30}
          maxLength={600}
          rows={3}
          value={form.plans}
          onChange={(e) => setForm((f) => ({ ...f, plans: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <button
        type="submit"
        disabled={saving}
        className="btn-minecraft px-8 py-3 rounded-xl font-bold disabled:opacity-60"
      >
        {saving ? "Отправка..." : "Отправить заявку"}
      </button>
    </form>
  );
}

// ==== Settings Tab ====
function SettingsTab({ user }: { user: ReturnType<typeof useAuth>["user"] }) {
  const [form, setForm] = useState({
    email: user?.email || "",
    first_name: user?.first_name || "",
    age: user?.age ? String(user.age) : "",
    discord_username: user?.discord || "",
    bio: user?.bio || "",
    current_password: "",
    new_password: "",
  });
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<{ text: string; type: "success" | "error" } | null>(null);

  useEffect(() => {
    async function loadProfile() {
      try {
        const res = await fetch("/api/profile", {
          headers: { Authorization: `Bearer ${localStorage.getItem("auth_token")}` },
        });
        if (res.ok) {
          const data = await res.json();
          setForm((f) => ({
            ...f,
            email: data.email || f.email,
            first_name: data.first_name || "",
            age: data.age ? String(data.age) : "",
            discord_username: data.discord || "",
            bio: data.bio || "",
          }));
        }
      } catch { /* ignore */ }
    }
    loadProfile();
  }, []);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setMsg(null);
    try {
      const body: Record<string, unknown> = {
        email: form.email,
        first_name: form.first_name,
        discord_username: form.discord_username,
        bio: form.bio,
      };
      if (form.age) body.age = parseInt(form.age, 10);
      if (form.new_password) {
        body.current_password = form.current_password;
        body.new_password = form.new_password;
      }
      const res = await fetch("/api/profile", {
        method: "PUT",
        headers: {
          Authorization: `Bearer ${localStorage.getItem("auth_token")}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setMsg({ text: data.message || "Профиль обновлён", type: "success" });
        setForm((f) => ({ ...f, current_password: "", new_password: "" }));
      } else {
        setMsg({ text: data.error || "Ошибка сохранения", type: "error" });
      }
    } catch {
      setMsg({ text: "Ошибка соединения с сервером", type: "error" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={save} className="space-y-4 max-w-xl">
      <h3 className="text-lg font-bold text-[#FFAA00] mb-2">Настройки профиля</h3>
      {msg && (
        <div className={`p-3 rounded-lg text-sm ${msg.type === "success" ? "bg-green-900/40 text-green-300 border border-green-500/30" : "bg-red-900/40 text-red-300 border border-red-500/30"}`}>
          {msg.text}
        </div>
      )}
      <label className="block text-sm">
        <span className="text-gray-400">Email</span>
        <input
          type="email"
          value={form.email}
          onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">Имя</span>
        <input
          value={form.first_name}
          onChange={(e) => setForm((f) => ({ ...f, first_name: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">Возраст</span>
        <input
          type="number"
          min={10}
          max={120}
          value={form.age}
          onChange={(e) => setForm((f) => ({ ...f, age: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">Discord</span>
        <input
          value={form.discord_username}
          onChange={(e) => setForm((f) => ({ ...f, discord_username: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <label className="block text-sm">
        <span className="text-gray-400">О себе</span>
        <textarea
          rows={3}
          maxLength={1000}
          value={form.bio}
          onChange={(e) => setForm((f) => ({ ...f, bio: e.target.value }))}
          className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
        />
      </label>
      <div className="border-t border-[#FFAA00]/20 pt-4 mt-4">
        <h4 className="text-[#FFAA00] font-medium mb-3">Смена пароля</h4>
        <label className="block text-sm mb-3">
          <span className="text-gray-400">Текущий пароль</span>
          <input
            type="password"
            value={form.current_password}
            onChange={(e) => setForm((f) => ({ ...f, current_password: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
        <label className="block text-sm">
          <span className="text-gray-400">Новый пароль</span>
          <input
            type="password"
            minLength={6}
            value={form.new_password}
            onChange={(e) => setForm((f) => ({ ...f, new_password: e.target.value }))}
            className="mt-1 w-full px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
          />
        </label>
      </div>
      <button
        type="submit"
        disabled={saving}
        className="btn-minecraft px-8 py-3 rounded-xl font-bold disabled:opacity-60"
      >
        {saving ? "Сохранение..." : "Сохранить"}
      </button>
    </form>
  );
}

// ==== Game Token Tab ====
function GameTokenTab() {
  const [token, setToken] = useState<string | null>(null);
  const [expiresAt, setExpiresAt] = useState<string | null>(null);
  const [timeLeft, setTimeLeft] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Обратный отсчет
  useEffect(() => {
    if (!expiresAt) return;
    const interval = setInterval(() => {
      const diff = new Date(expiresAt).getTime() - Date.now();
      if (diff <= 0) {
        setToken(null);
        setExpiresAt(null);
        setTimeLeft("");
        clearInterval(interval);
        return;
      }
      const m = Math.floor(diff / 60000);
      const s = Math.floor((diff % 60000) / 1000);
      setTimeLeft(`${m}м ${s}с`);
    }, 1000);
    return () => clearInterval(interval);
  }, [expiresAt]);

  async function generateToken() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/auth/generate-game-token", {
        method: "POST",
        headers: {
          "Authorization": `Bearer ${localStorage.getItem("auth_token")}`,
          "Content-Type": "application/json",
        },
      });
      const data = await res.json();
      if (data.success || data.token) {
        setToken(data.token);
        setExpiresAt(data.expiresAt || data.expires_at);
      } else {
        setError(data.error || "Ошибка генерации токена");
      }
    } catch {
      setError("Ошибка соединения с сервером");
    } finally {
      setLoading(false);
    }
  }

  async function copyToken() {
    if (!token) return;
    await navigator.clipboard.writeText(token);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="space-y-6">
      <div className="glass-effect rounded-2xl p-6">
        <h3 className="text-lg font-bold text-[#FFAA00] mb-2 flex items-center gap-2">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg>
          Игровой токен
        </h3>
        <p className="text-gray-400 text-sm mb-6">
          Используется для авторизации на сервере Minecraft. Каждый токен действует 15 минут и может быть использован только один раз.
        </p>

        {error && (
          <div className="mb-4 p-3 rounded-lg bg-red-900/40 border border-red-500/40 text-red-300 text-sm">
            {error}
          </div>
        )}

        {token && (
          <div className="mb-6 bg-[#2a2a2a]/60 border border-[#FFAA00]/30 rounded-xl p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-[#FFAA00] text-sm font-medium">Активный токен:</span>
              <button
                onClick={copyToken}
                className={`text-sm px-3 py-1 rounded-md font-medium transition-colors ${
                  copied
                    ? "bg-green-600 text-white"
                    : "bg-[#FFAA00] hover:bg-[#FFFF55] text-[#0a0a0a]"
                }`}
                id="copy-token-btn"
              >
                {copied ? "Скопировано!" : "Копировать"}
              </button>
            </div>
            <div className="font-mono text-xs text-[#FFFF55] break-all bg-[#0a0a0a]/50 p-3 rounded-lg">
              {token}
            </div>
            {timeLeft && (
              <p className="mt-2 text-xs text-gray-400">
                Истекает через: <span className="text-[#FFAA00] font-bold">{timeLeft}</span>
              </p>
            )}
          </div>
        )}

        <button
          onClick={generateToken}
          disabled={loading}
          className="btn-minecraft px-8 py-3 rounded-xl font-bold text-base inline-flex items-center gap-2 disabled:opacity-60 disabled:cursor-not-allowed"
          id="generate-token-btn"
        >
          {loading ? (
            <>
              <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              Генерация...
            </>
          ) : (
            <>
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              Сгенерировать токен
            </>
          )}
        </button>
      </div>

      {/* Инструкция */}
      <div className="glass-effect rounded-2xl p-6">
        <h4 className="text-base font-bold text-[#FFAA00] mb-4">Как использовать токен</h4>
        <div className="space-y-3">
          {[
            "Нажмите кнопку «Сгенерировать токен» выше",
            "Скопируйте полученный токен",
            "Подключитесь к серверу Minecraft: play.chiwawa.site",
            "Введите команду: /auth ваш_токен",
            "Готово! Вы авторизованы на сервере",
          ].map((step, i) => (
            <div key={i} className="flex items-start gap-3">
              <div className="w-7 h-7 rounded-full bg-[#FFAA00] text-[#0a0a0a] font-bold text-sm flex items-center justify-center shrink-0 mt-0.5">
                {i + 1}
              </div>
              <p className="text-gray-300 text-sm">{step}</p>
            </div>
          ))}
        </div>
        <div className="mt-5 p-4 bg-orange-900/20 border border-orange-500/30 rounded-xl">
          <p className="text-orange-400 font-bold text-sm mb-2">⚠ Важно:</p>
          <ul className="text-gray-400 text-sm space-y-1">
            <li>• Токен действует ограниченное время</li>
            <li>• Не передавайте токен другим игрокам</li>
            <li>• Один токен можно использовать только один раз</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
