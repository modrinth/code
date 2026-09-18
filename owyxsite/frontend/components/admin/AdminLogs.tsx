"use client";

import { useCallback, useEffect, useState } from "react";
import { useLocale } from "@/hooks/useLocale";

type AuthHeaders = () => Record<string, string>;

type LogTab = "account" | "admin" | "launcher";

type Pagination = {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
};

type ActivityRow = {
  id: number;
  user_id: number;
  activity_type: string;
  description: string;
  metadata?: unknown;
  ip_address?: string | null;
  created_at: string;
  nickname?: string | null;
  display_nickname?: string | null;
  email?: string | null;
};

type AdminLogRow = {
  id: number;
  action: string;
  details?: string | null;
  created_at: string;
  admin_nickname?: string | null;
  target_user_nickname?: string | null;
  target_user_id?: number | null;
};

type TelemetryRow = {
  id: number;
  install_id: string;
  user_id?: number | null;
  event_kind: string;
  message?: string | null;
  app_version?: string | null;
  os_name?: string | null;
  os_version?: string | null;
  arch?: string | null;
  cpu_cores?: number | null;
  ram_mb?: number | null;
  locale?: string | null;
  created_at: string;
  linked_nickname?: string | null;
};

type TelemetryStats = {
  total_events: number;
  last_24h: number;
  errors: number;
  installs: number;
};

function fmtWhen(iso: string, en: boolean) {
  try {
    return new Date(iso).toLocaleString(en ? "en-US" : "ru-RU", {
      day: "2-digit",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

function kindBadge(kind: string) {
  if (kind === "error" || kind === "crash" || kind.includes("ban") || kind.includes("delete")) {
    return "badge badge-danger";
  }
  if (kind === "session_start" || kind.includes("login") || kind.includes("register")) {
    return "badge badge-ok";
  }
  return "badge badge-accent";
}

/** Logs are Russian-only (ops language) — no locale switch for stored/type labels. */
const ACTIVITY_LABELS_RU: Record<string, string> = {
  login: "Вход",
  logout: "Выход",
  register: "Регистрация",
  password_reset: "Сброс пароля",
  password_change: "Смена пароля",
  profile_update: "Профиль",
  email_change: "Смена почты",
  nickname_change: "Смена логина",
  display_nickname_change: "Отображаемый ник",
  avatar_update: "Аватар",
  avatar_delete: "Удаление аватара",
  skin_update: "Скин",
  skin_delete: "Удаление скина",
  discord_linked: "Discord привязан",
  discord_unlinked: "Discord отвязан",
  social_settings: "Соц. приватность",
};

const TELEMETRY_LABELS_RU: Record<string, string> = {
  session_start: "Старт сессии",
  heartbeat: "Heartbeat",
  error: "Ошибка",
  crash: "Крах",
  perf: "Производительность",
  feature: "Фича",
};

const ADMIN_ACTION_LABELS_RU: Record<string, string> = {
  role_changed: "Смена роли",
  profile_update: "Правка профиля",
  user_banned: "Бан",
  user_unbanned: "Разбан",
  user_deleted: "Удаление",
  user_moderation: "Модерация",
  application_reviewed: "Заявка",
  settings_updated: "Настройки",
  email_sent: "Письмо",
  email_test: "Тест почты",
  cache_cleared: "Сброс кэша",
  api_token_created: "API-токен создан",
  api_token_updated: "API-токен обновлён",
  api_token_deleted: "API-токен удалён",
};

function humanLabel(raw: string, map: Record<string, string>) {
  return map[raw] || raw.replace(/_/g, " ");
}

export default function AdminLogs({
  authHeaders,
  showMessage,
}: {
  authHeaders: AuthHeaders;
  showMessage: (text: string, type: "success" | "error") => void;
}) {
  const { locale } = useLocale();
  const en = locale === "en_US";
  const t = (a: string, b: string) => (en ? a : b);

  const [tab, setTab] = useState<LogTab>("account");
  const [q, setQ] = useState("");
  const [filter, setFilter] = useState("all");
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);

  const [activity, setActivity] = useState<ActivityRow[]>([]);
  const [activityTypes, setActivityTypes] = useState<{ type: string; count: number }[]>([]);
  const [adminLogs, setAdminLogs] = useState<AdminLogRow[]>([]);
  const [telemetry, setTelemetry] = useState<TelemetryRow[]>([]);
  const [telemetryKinds, setTelemetryKinds] = useState<{ kind: string; count: number }[]>([]);
  const [stats, setStats] = useState<TelemetryStats | null>(null);
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    limit: 50,
    total: 0,
    totalPages: 1,
  });

  const load = useCallback(async () => {
    setLoading(true);
    try {
      if (tab === "account") {
        const params = new URLSearchParams({
          page: String(page),
          limit: "50",
          type: filter,
        });
        if (q.trim()) params.set("q", q.trim());
        const res = await fetch(`/api/admin/activity?${params}`, { headers: authHeaders() });
        const data = await res.json().catch(() => ({}));
        if (!res.ok) throw new Error(data.error || "Failed");
        setActivity(data.activity || []);
        setActivityTypes(data.types || []);
        setPagination(data.pagination || { page: 1, limit: 50, total: 0, totalPages: 1 });
      } else if (tab === "admin") {
        const params = new URLSearchParams({
          page: String(page),
          limit: "50",
          action: filter,
        });
        const res = await fetch(`/api/admin/logs?${params}`, { headers: authHeaders() });
        const data = await res.json().catch(() => ({}));
        if (!res.ok) throw new Error(data.error || "Failed");
        setAdminLogs(data.logs || []);
        setPagination(data.pagination || { page: 1, limit: 50, total: 0, totalPages: 1 });
      } else {
        const params = new URLSearchParams({
          page: String(page),
          limit: "50",
          kind: filter,
        });
        if (q.trim()) params.set("q", q.trim());
        const res = await fetch(`/api/admin/telemetry?${params}`, { headers: authHeaders() });
        const data = await res.json().catch(() => ({}));
        if (!res.ok) throw new Error(data.error || "Failed");
        setTelemetry(data.events || []);
        setTelemetryKinds(data.kinds || []);
        setStats(data.stats || null);
        setPagination(data.pagination || { page: 1, limit: 50, total: 0, totalPages: 1 });
      }
    } catch (err) {
      showMessage(
        err instanceof Error
          ? err.message
          : en
            ? "Failed to load logs"
            : "Не удалось загрузить логи",
        "error",
      );
    } finally {
      setLoading(false);
    }
  }, [authHeaders, en, filter, page, q, showMessage, tab]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    setPage(1);
    setFilter("all");
    setQ("");
  }, [tab]);

  const filterOptions =
    tab === "account"
      ? activityTypes.map((x) => x.type)
      : tab === "launcher"
        ? telemetryKinds.map((x) => x.kind)
        : [
            "role_changed",
            "profile_update",
            "user_banned",
            "user_unbanned",
            "user_deleted",
            "user_moderation",
            "application_reviewed",
          ];

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h2 className="font-display text-lg font-bold tracking-tight">
            {t("Logs", "Логи")}
          </h2>
          <p className="mt-1 text-sm text-muted leading-relaxed max-w-xl">
            {t(
              "Account changes, moderation actions, and anonymous launcher stats.",
              "Смены аккаунта, модерация и анонимная статистика лаунчера.",
            )}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {(
            [
              ["account", t("Account", "Аккаунт")],
              ["admin", t("Moderation", "Модерация")],
              ["launcher", t("Launcher", "Лаунчер")],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              className={`btn btn-sm ${tab === id ? "btn-primary" : "btn-ghost"}`}
              onClick={() => setTab(id)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {tab === "launcher" && stats && (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {[
            [t("Events", "События"), stats.total_events],
            [t("Last 24h", "За 24ч"), stats.last_24h],
            [t("Errors", "Ошибки"), stats.errors],
            [t("Installs", "Установки"), stats.installs],
          ].map(([label, value]) => (
            <div key={String(label)} className="section-callout !py-3 !px-4">
              <p className="text-xs uppercase tracking-wider text-muted">{label}</p>
              <p className="mt-1 font-display text-2xl font-bold text-text">{value}</p>
            </div>
          ))}
        </div>
      )}

      <div className="flex flex-wrap gap-2 items-center">
        {(tab === "account" || tab === "launcher") && (
          <input
            className="input max-w-xs"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder={t("Search…", "Поиск…")}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                setPage(1);
                void load();
              }
            }}
          />
        )}
        <select
          className="input max-w-[14rem]"
          value={filter}
          onChange={(e) => {
            setFilter(e.target.value);
            setPage(1);
          }}
        >
          <option value="all">{t("All types", "Все типы")}</option>
          {filterOptions.map((opt) => (
            <option key={opt} value={opt}>
              {tab === "account"
                ? humanLabel(opt, ACTIVITY_LABELS_RU)
                : tab === "launcher"
                  ? humanLabel(opt, TELEMETRY_LABELS_RU)
                  : humanLabel(opt, ADMIN_ACTION_LABELS_RU)}
            </option>
          ))}
        </select>
        <button
          type="button"
          className="btn btn-ghost btn-sm"
          onClick={() => {
            setPage(1);
            void load();
          }}
        >
          {t("Refresh", "Обновить")}
        </button>
        <span className="text-xs text-muted ml-auto">
          {pagination.total} {t("rows", "записей")}
        </span>
      </div>

      <div className="panel overflow-hidden">
        <div className="overflow-x-auto overflow-y-auto max-h-[min(32rem,70vh)]">
          {loading ? (
            <p className="p-6 text-sm text-muted">{t("Loading…", "Загрузка…")}</p>
          ) : tab === "account" ? (
            <table className="w-full text-sm table-fixed">
              <thead className="sticky top-0 bg-panel-2/95 backdrop-blur text-left text-xs uppercase tracking-wider text-muted">
                <tr>
                  <th className="w-28 px-4 py-3 font-medium">{t("When", "Когда")}</th>
                  <th className="w-36 px-4 py-3 font-medium">{t("User", "Игрок")}</th>
                  <th className="w-40 px-4 py-3 font-medium">{t("Type", "Тип")}</th>
                  <th className="px-4 py-3 font-medium">{t("Details", "Детали")}</th>
                </tr>
              </thead>
              <tbody>
                {activity.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-4 py-8 text-center text-muted">
                      {t("No account activity yet.", "Пока нет активности аккаунтов.")}
                    </td>
                  </tr>
                ) : (
                  activity.map((row) => (
                    <tr key={row.id} className="border-t border-line/60 hover:bg-panel-2/40 align-top">
                      <td className="px-4 py-2.5 whitespace-nowrap text-muted">
                        {fmtWhen(row.created_at, en)}
                      </td>
                      <td className="px-4 py-2.5">
                        <span className="font-medium text-text">
                          {row.display_nickname || row.nickname || `#${row.user_id}`}
                        </span>
                        {row.nickname && row.display_nickname && row.nickname !== row.display_nickname && (
                          <span className="block text-xs text-muted">{row.nickname}</span>
                        )}
                      </td>
                      <td className="px-4 py-2.5">
                        <span className={kindBadge(row.activity_type)} title={row.activity_type}>
                          {humanLabel(row.activity_type, ACTIVITY_LABELS_RU)}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-muted whitespace-normal break-words">
                        {row.description}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          ) : tab === "admin" ? (
            <table className="w-full text-sm table-fixed">
              <thead className="sticky top-0 bg-panel-2/95 backdrop-blur text-left text-xs uppercase tracking-wider text-muted">
                <tr>
                  <th className="w-28 px-4 py-3 font-medium">{t("When", "Когда")}</th>
                  <th className="w-28 px-4 py-3 font-medium">{t("Admin", "Админ")}</th>
                  <th className="w-36 px-4 py-3 font-medium">{t("Action", "Действие")}</th>
                  <th className="w-28 px-4 py-3 font-medium">{t("Target", "Цель")}</th>
                  <th className="px-4 py-3 font-medium">{t("Details", "Детали")}</th>
                </tr>
              </thead>
              <tbody>
                {adminLogs.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-muted">
                      {t("No moderation logs yet.", "Пока нет логов модерации.")}
                    </td>
                  </tr>
                ) : (
                  adminLogs.map((row) => (
                    <tr key={row.id} className="border-t border-line/60 hover:bg-panel-2/40 align-top">
                      <td className="px-4 py-2.5 whitespace-nowrap text-muted">
                        {fmtWhen(row.created_at, en)}
                      </td>
                      <td className="px-4 py-2.5 font-medium">{row.admin_nickname || "—"}</td>
                      <td className="px-4 py-2.5">
                        <span className={kindBadge(row.action)} title={row.action}>
                          {humanLabel(row.action, ADMIN_ACTION_LABELS_RU)}
                        </span>
                      </td>
                      <td className="px-4 py-2.5">{row.target_user_nickname || "—"}</td>
                      <td className="px-4 py-2.5 text-muted whitespace-normal break-words">
                        {row.details || "—"}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          ) : (
            <table className="w-full text-sm table-fixed">
              <thead className="sticky top-0 bg-panel-2/95 backdrop-blur text-left text-xs uppercase tracking-wider text-muted">
                <tr>
                  <th className="w-28 px-4 py-3 font-medium">{t("When", "Когда")}</th>
                  <th className="w-36 px-4 py-3 font-medium">{t("Kind", "Тип")}</th>
                  <th className="w-36 px-4 py-3 font-medium">{t("Device", "Устройство")}</th>
                  <th className="w-28 px-4 py-3 font-medium">{t("App", "Приложение")}</th>
                  <th className="px-4 py-3 font-medium">{t("Message", "Сообщение")}</th>
                </tr>
              </thead>
              <tbody>
                {telemetry.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-muted">
                      {t(
                        "No launcher telemetry yet. Clients will appear after opt-in telemetry events.",
                        "Пока нет телеметрии лаунчера. Появится после событий с включённой телеметрией.",
                      )}
                    </td>
                  </tr>
                ) : (
                  telemetry.map((row) => (
                    <tr key={row.id} className="border-t border-line/60 hover:bg-panel-2/40 align-top">
                      <td className="px-4 py-2.5 whitespace-nowrap text-muted">
                        {fmtWhen(row.created_at, en)}
                      </td>
                      <td className="px-4 py-2.5">
                        <span className={kindBadge(row.event_kind)} title={row.event_kind}>
                          {humanLabel(row.event_kind, TELEMETRY_LABELS_RU)}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-muted">
                        <span className="text-text">
                          {[row.os_name, row.arch].filter(Boolean).join(" · ") || "—"}
                        </span>
                        <span className="block text-xs">
                          {[
                            row.cpu_cores ? `${row.cpu_cores}c` : null,
                            row.ram_mb ? `${Math.round(row.ram_mb / 1024)} GB` : null,
                          ]
                            .filter(Boolean)
                            .join(" · ")}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-muted">
                        {row.app_version || "—"}
                        {row.linked_nickname && (
                          <span className="block text-xs text-accent">{row.linked_nickname}</span>
                        )}
                      </td>
                      <td className="px-4 py-2.5 text-muted whitespace-normal break-words">
                        {row.message || "—"}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {pagination.totalPages > 1 && (
        <div className="flex items-center justify-center gap-3">
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            disabled={page <= 1}
            onClick={() => setPage((p) => Math.max(1, p - 1))}
          >
            ←
          </button>
          <span className="text-sm text-muted">
            {page} / {pagination.totalPages}
          </span>
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            disabled={page >= pagination.totalPages}
            onClick={() => setPage((p) => p + 1)}
          >
            →
          </button>
        </div>
      )}
    </div>
  );
}
