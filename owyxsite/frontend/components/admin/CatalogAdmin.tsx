"use client";

import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useLocale } from "@/hooks/useLocale";

type SourceType = "http_zip" | "http_manifest" | "google_drive" | "mrpack" | "sftp" | "local_ingest";

type PackRow = {
  id: string;
  name: string;
  minecraft: string;
  loader: string;
  iconUrl?: string | null;
  description?: string;
  sourceType: SourceType;
  source?: { type: SourceType; config: Record<string, unknown> };
  published: boolean;
  accessMode?: "open" | "whitelist" | "blacklist";
};

type ServerRow = {
  id: string;
  name: string;
  iconUrl?: string | null;
  address: string;
  port: number;
  kind: "owyx" | "community";
  packId?: string | null;
  minecraft?: string | null;
  loader?: string | null;
  requiresAccount: boolean;
  published: boolean;
  accessMode?: "open" | "whitelist" | "blacklist";
  sortOrder: number;
};

const LOADERS = ["vanilla", "fabric", "forge", "neoforge", "quilt"];

const MC_VERSION_SUGGESTIONS = [
  "1.21.8",
  "1.21.7",
  "1.21.6",
  "1.21.5",
  "1.21.4",
  "1.21.3",
  "1.21.1",
  "1.21",
  "1.20.6",
  "1.20.4",
  "1.20.1",
  "1.19.4",
  "1.19.2",
  "1.18.2",
  "1.16.5",
  "1.12.2",
  "1.8.9",
  "1.7.10",
];

function SearchableSelect({
  value,
  onChange,
  options,
  placeholder,
  listId,
}: {
  value: string;
  onChange: (v: string) => void;
  options: string[];
  placeholder?: string;
  listId: string;
}) {
  const [query, setQuery] = useState(value);
  const [open, setOpen] = useState(false);
  useEffect(() => {
    setQuery(value);
  }, [value]);
  const filtered = options.filter((o) => o.toLowerCase().includes(query.toLowerCase()));
  return (
    <div className="relative">
      <input
        className="input w-full"
        value={query}
        placeholder={placeholder}
        autoComplete="off"
        list={listId}
        onFocus={() => setOpen(true)}
        onBlur={() => setTimeout(() => setOpen(false), 120)}
        onChange={(e) => {
          setQuery(e.target.value);
          onChange(e.target.value);
          setOpen(true);
        }}
      />
      <datalist id={listId}>
        {options.map((o) => (
          <option key={o} value={o} />
        ))}
      </datalist>
      {open && filtered.length > 0 && (
        <div className="absolute z-20 mt-1 max-h-48 w-full overflow-auto rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-lg">
          {filtered.slice(0, 40).map((o) => (
            <button
              key={o}
              type="button"
              className="block w-full cursor-pointer border-0 bg-transparent px-3 py-1.5 text-left text-sm hover:bg-[var(--bg-hover)]"
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => {
                setQuery(o);
                onChange(o);
                setOpen(false);
              }}
            >
              {o}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}


const emptyPack = {
  name: "",
  minecraft: "1.21.1",
  loader: "vanilla",
  iconUrl: "",
  description: "",
  sourceType: "http_zip" as SourceType,
  url: "",
  sha256: "",
  manifestUrl: "",
  host: "",
  port: "22",
  user: "",
  path: "",
  password: "",
  published: true,
  accessMode: "open" as "open" | "whitelist" | "blacklist",
  aclNicknames: "",
};

const emptyServer = {
  name: "",
  iconUrl: "",
  address: "",
  port: "25565",
  kind: "owyx" as "owyx" | "community",
  packId: "",
  minecraft: "1.21.1",
  loader: "vanilla",
  requiresAccount: false,
  published: true,
  accessMode: "open" as "open" | "whitelist" | "blacklist",
  aclNicknames: "",
  sortOrder: "0",
};

export default function CatalogAdmin({
  authHeaders,
  showMessage,
}: {
  authHeaders: () => Record<string, string>;
  showMessage: (t: string, k: "success" | "error") => void;
}) {
  const { locale } = useLocale();
  const en = locale === "en_US";
  const t = (enText: string, ruText: string) => (en ? enText : ruText);
  const SOURCE_TYPES: { id: SourceType; label: string }[] = useMemo(
    () => [
      { id: "http_zip", label: "HTTP zip" },
      { id: "http_manifest", label: t("HTTP manifest", "HTTP манифест") },
      { id: "google_drive", label: "Google Drive" },
      { id: "mrpack", label: "mrpack" },
      { id: "sftp", label: t("SFTP (admin only)", "SFTP (только админ)") },
      { id: "local_ingest", label: t("Local upload", "Локальная заливка") },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps -- locale drives labels
    [locale],
  );

  const [tab, setTab] = useState<"servers" | "packs">("servers");
  const [servers, setServers] = useState<ServerRow[]>([]);
  const [packs, setPacks] = useState<PackRow[]>([]);
  const [packForm, setPackForm] = useState(emptyPack);
  const [serverForm, setServerForm] = useState(emptyServer);
  const [editPackId, setEditPackId] = useState<string | null>(null);
  const [editServerId, setEditServerId] = useState<string | null>(null);
  const [openPack, setOpenPack] = useState(false);
  const [openServer, setOpenServer] = useState(false);
  const [busy, setBusy] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const load = useCallback(async () => {
    try {
      const [sRes, pRes] = await Promise.all([
        fetch("/api/admin/servers", { headers: authHeaders() }),
        fetch("/api/admin/packs", { headers: authHeaders() }),
      ]);
      if (sRes.ok) {
        const data = await sRes.json();
        setServers(data.servers ?? []);
      }
      if (pRes.ok) {
        const data = await pRes.json();
        setPacks(data.packs ?? []);
      }
    } catch {
      /* ignore */
    } finally {
      setLoaded(true);
    }
  }, [authHeaders]);

  useEffect(() => {
    void load();
  }, [load]);

  function sourcePayload() {
    const type = packForm.sourceType;
    if (type === "http_zip" || type === "local_ingest") {
      return { type, config: { url: packForm.url, sha256: packForm.sha256 || undefined } };
    }
    if (type === "http_manifest") {
      return { type, config: { manifestUrl: packForm.manifestUrl || packForm.url } };
    }
    if (type === "google_drive" || type === "mrpack") {
      return { type, config: { url: packForm.url } };
    }
    return {
      type,
      config: {
        host: packForm.host,
        port: parseInt(packForm.port, 10) || 22,
        user: packForm.user,
        path: packForm.path,
        password: packForm.password || undefined,
      },
    };
  }

  function startCreatePack() {
    setEditPackId(null);
    setPackForm(emptyPack);
    setOpenPack(true);
  }

  function startEditPack(p: PackRow) {
    const cfg = p.source?.config ?? {};
    setEditPackId(p.id);
    setPackForm({
      ...emptyPack,
      name: p.name,
      minecraft: p.minecraft,
      loader: p.loader,
      iconUrl: p.iconUrl || "",
      description: p.description || "",
      sourceType: p.sourceType,
      url: String(cfg.url || cfg.directDownloadUrl || ""),
      sha256: String(cfg.sha256 || ""),
      manifestUrl: String(cfg.manifestUrl || ""),
      host: String(cfg.host || ""),
      port: String(cfg.port || "22"),
      user: String(cfg.user || ""),
      path: String(cfg.path || ""),
      password: "",
      published: p.published,
      accessMode: p.accessMode || "open",
      aclNicknames: "",
    });
    setOpenPack(true);
    void (async () => {
      try {
        const res = await fetch(`/api/admin/packs/${p.id}/acl`, { headers: authHeaders() });
        const data = await res.json().catch(() => ({}));
        if (res.ok && Array.isArray(data.entries)) {
          setPackForm((f) => ({
            ...f,
            aclNicknames: data.entries.map((e: { nickname: string }) => e.nickname).join(", "),
          }));
        }
      } catch {
        /* ignore */
      }
    })();
  }

  function startCreateServer() {
    setEditServerId(null);
    setServerForm({
      ...emptyServer,
      packId: packs[0]?.id || "",
      minecraft: packs[0]?.minecraft || "1.21.1",
      loader: packs[0]?.loader || "vanilla",
    });
    setOpenServer(true);
  }

  function startEditServer(s: ServerRow) {
    setEditServerId(s.id);
    setServerForm({
      name: s.name,
      iconUrl: s.iconUrl || "",
      address: s.address,
      port: String(s.port),
      kind: s.kind,
      packId: s.packId || "",
      minecraft: s.minecraft || "",
      loader: s.loader || "vanilla",
      requiresAccount: s.requiresAccount,
      published: s.published,
      accessMode: s.accessMode || "open",
      aclNicknames: "",
      sortOrder: String(s.sortOrder ?? 0),
    });
    setOpenServer(true);
    void (async () => {
      try {
        const res = await fetch(`/api/admin/servers/${s.id}/acl`, { headers: authHeaders() });
        const data = await res.json().catch(() => ({}));
        if (res.ok && Array.isArray(data.entries)) {
          setServerForm((f) => ({
            ...f,
            aclNicknames: data.entries.map((e: { nickname: string }) => e.nickname).join(", "),
          }));
        }
      } catch {
        /* ignore */
      }
    })();
  }

  async function savePack(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      const payload = {
        name: packForm.name,
        minecraft: packForm.minecraft,
        loader: packForm.loader,
        iconUrl: packForm.iconUrl || null,
        description: packForm.description,
        sourceType: packForm.sourceType,
        source: sourcePayload(),
        published: packForm.published,
        accessMode: packForm.accessMode,
      };
      const res = await fetch(editPackId ? `/api/admin/packs/${editPackId}` : "/api/admin/packs", {
        method: editPackId ? "PUT" : "POST",
        headers: authHeaders(),
        body: JSON.stringify(payload),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        showMessage(data.error || "Не удалось сохранить пак", "error");
        return;
      }
      const packId = editPackId || data.pack?.id;
      if (packId) {
        const nicknames = packForm.aclNicknames
          .split(/[,;\s]+/)
          .map((n) => n.trim())
          .filter(Boolean);
        await fetch(`/api/admin/packs/${packId}/acl`, {
          method: "PUT",
          headers: authHeaders(),
          body: JSON.stringify({ accessMode: packForm.accessMode, nicknames }),
        });
      }      showMessage(editPackId ? "Пак обновлён" : "Пак создан", "success");
      setOpenPack(false);
      await load();
    } catch {
      showMessage("Ошибка соединения", "error");
    } finally {
      setBusy(false);
    }
  }

  async function saveServer(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      const payload = {
        name: serverForm.name,
        iconUrl: serverForm.iconUrl || null,
        address: serverForm.address,
        port: parseInt(serverForm.port, 10),
        kind: serverForm.kind,
        packId: serverForm.packId || null,
        minecraft: serverForm.minecraft || null,
        loader: serverForm.loader || null,
        requiresAccount: serverForm.requiresAccount,
        published: serverForm.published,
        accessMode: serverForm.accessMode,
        sortOrder: parseInt(serverForm.sortOrder, 10) || 0,
      };
      const res = await fetch(
        editServerId ? `/api/admin/servers/${editServerId}` : "/api/admin/servers",
        {
          method: editServerId ? "PUT" : "POST",
          headers: authHeaders(),
          body: JSON.stringify(payload),
        }
      );
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        showMessage(data.error || "Не удалось сохранить сервер", "error");
        return;
      }
      const serverId = editServerId || data.server?.id;
      if (serverId) {
        const nicknames = serverForm.aclNicknames
          .split(/[,;\s]+/)
          .map((n) => n.trim())
          .filter(Boolean);
        await fetch(`/api/admin/servers/${serverId}/acl`, {
          method: "PUT",
          headers: authHeaders(),
          body: JSON.stringify({ accessMode: serverForm.accessMode, nicknames }),
        });
      }      showMessage(editServerId ? "Сервер обновлён" : "Сервер создан", "success");
      setOpenServer(false);
      await load();
    } catch {
      showMessage("Ошибка соединения", "error");
    } finally {
      setBusy(false);
    }
  }

  async function deleteServer(s: ServerRow) {
    if (!window.confirm(`Удалить сервер «${s.name}»?`)) return;
    const res = await fetch(`/api/admin/servers/${s.id}`, {
      method: "DELETE",
      headers: authHeaders(),
    });
    if (res.ok) {
      showMessage("Сервер удалён", "success");
      await load();
    } else showMessage("Не удалось удалить сервер", "error");
  }

  async function deletePack(p: PackRow) {
    if (!window.confirm(`Удалить пак «${p.name}»?`)) return;
    const res = await fetch(`/api/admin/packs/${p.id}`, {
      method: "DELETE",
      headers: authHeaders(),
    });
    if (res.ok) {
      showMessage("Пак удалён", "success");
      await load();
    } else showMessage("Не удалось удалить пак", "error");
  }

  async function ingestPack(p: PackRow, file: File) {
    const fd = new FormData();
    fd.append("archive", file);
    const headers = { Authorization: authHeaders().Authorization };
    const res = await fetch(`/api/admin/packs/${p.id}/ingest`, {
      method: "POST",
      headers,
      body: fd,
    });
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      showMessage("Архив принят, downloadUrl обновлён", "success");
      await load();
    } else {
      showMessage(data.error || "Не удалось залить архив", "error");
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h2 className="font-display text-lg font-bold tracking-tight">
            {t("Launcher catalog", "Каталог лаунчера")}
          </h2>
          <p className="mt-1 text-sm text-muted">
            {t(
              "Servers and packs the launcher can see. SFTP is admin-only.",
              "Серверы и сборки, которые видит лаунчер. SFTP — только для админа.",
            )}
          </p>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            className={`cabinet-rail-item ${tab === "servers" ? "is-active" : ""}`}
            onClick={() => setTab("servers")}
          >
            {t("Servers", "Серверы")}
          </button>
          <button
            type="button"
            className={`cabinet-rail-item ${tab === "packs" ? "is-active" : ""}`}
            onClick={() => setTab("packs")}
          >
            {t("Packs", "Сборки")}
          </button>
        </div>
      </div>

      {!loaded ? (
        <p className="text-sm text-muted py-8 text-center">Загрузка каталога…</p>
      ) : tab === "servers" ? (
        <div className="space-y-4">
          <button type="button" className="btn btn-primary" onClick={startCreateServer}>
            Добавить сервер
          </button>
          {servers.length === 0 ? (
            <div className="empty-surface">
              <h3>Серверов пока нет</h3>
              <p>Создай карточку — лаунчер подхватит список.</p>
            </div>
          ) : (
            <ul className="space-y-2">
              {servers.map((s) => (
                <li
                  key={s.id}
                  className="list-row"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-text truncate">{s.name}</p>
                    <p className="text-xs text-muted truncate font-mono">
                      {s.address}:{s.port}
                      <span className="text-muted"> · </span>
                      {s.minecraft || "—"} {s.loader || ""}
                      {!s.published && <span className="text-muted"> · скрыт</span>}
                    </p>
                  </div>
                  <button
                    type="button"
                    className="btn btn-secondary btn-sm"
                    onClick={() => startEditServer(s)}
                  >
                    Изменить
                  </button>
                  <button
                    type="button"
                    className="btn btn-danger btn-sm"
                    onClick={() => void deleteServer(s)}
                  >
                    Удалить
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : (
        <div className="space-y-4">
          <button type="button" className="btn btn-primary" onClick={startCreatePack}>
            Добавить пак
          </button>
          {packs.length === 0 ? (
            <div className="empty-surface">
              <h3>Сборок пока нет</h3>
              <p>Добавь пак и привяжи его к серверу.</p>
            </div>
          ) : (
            <ul className="space-y-2">
              {packs.map((p) => (
                <li
                  key={p.id}
                  className="list-row"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-text truncate">{p.name}</p>
                    <p className="text-xs text-muted truncate">
                      {p.minecraft} {p.loader} · {p.sourceType}
                      {!p.published && <span className="text-muted"> · скрыт</span>}
                    </p>
                  </div>
                  <label className="btn btn-secondary btn-sm cursor-pointer">
                    Залить zip
                    <input
                      type="file"
                      accept=".zip,.mrpack"
                      className="hidden"
                      onChange={(e) => {
                        const file = e.target.files?.[0];
                        if (file) void ingestPack(p, file);
                        e.target.value = "";
                      }}
                    />
                  </label>
                  <button
                    type="button"
                    className="btn btn-secondary btn-sm"
                    onClick={() => startEditPack(p)}
                  >
                    Изменить
                  </button>
                  <button
                    type="button"
                    className="btn btn-danger btn-sm"
                    onClick={() => void deletePack(p)}
                  >
                    Удалить
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      {openPack && (
        <ModalShell title={editPackId ? "Изменить пак" : "Добавить пак"} onClose={() => setOpenPack(false)}>
          <form onSubmit={savePack} className="space-y-3">
            <Field label="Название">
              <input
                className="input"
                required
                value={packForm.name}
                onChange={(e) => setPackForm((f) => ({ ...f, name: e.target.value }))}
              />
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Minecraft">
                <SearchableSelect
                  listId="pack-mc-versions"
                  value={packForm.minecraft}
                  options={MC_VERSION_SUGGESTIONS}
                  placeholder="Search version…"
                  onChange={(minecraft) => setPackForm((f) => ({ ...f, minecraft }))}
                />
              </Field>
              <Field label="Лоадер">
                <SearchableSelect
                  listId="pack-loaders"
                  value={packForm.loader}
                  options={LOADERS}
                  placeholder="Search loader…"
                  onChange={(loader) => setPackForm((f) => ({ ...f, loader }))}
                />
              </Field>
            </div>
            <Field label="Иконка (URL)">
              <input
                className="input"
                value={packForm.iconUrl}
                onChange={(e) => setPackForm((f) => ({ ...f, iconUrl: e.target.value }))}
              />
            </Field>
            <Field label="Описание">
              <textarea
                className="textarea"
                value={packForm.description}
                onChange={(e) => setPackForm((f) => ({ ...f, description: e.target.value }))}
              />
            </Field>
            <Field label="Источник">
              <select
                className="select"
                value={packForm.sourceType}
                onChange={(e) =>
                  setPackForm((f) => ({ ...f, sourceType: e.target.value as SourceType }))
                }
              >
                {SOURCE_TYPES.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
            </Field>
            {(packForm.sourceType === "http_zip" ||
              packForm.sourceType === "local_ingest" ||
              packForm.sourceType === "google_drive" ||
              packForm.sourceType === "mrpack") && (
              <>
                <Field label="URL архива">
                  <input
                    className="input"
                    value={packForm.url}
                    onChange={(e) => setPackForm((f) => ({ ...f, url: e.target.value }))}
                  />
                </Field>
                <Field label="sha256 (необязательно)">
                  <input
                    className="input font-mono text-sm"
                    value={packForm.sha256}
                    onChange={(e) => setPackForm((f) => ({ ...f, sha256: e.target.value }))}
                  />
                </Field>
              </>
            )}
            {packForm.sourceType === "http_manifest" && (
              <Field label="URL манифеста">
                <input
                  className="input"
                  value={packForm.manifestUrl}
                  onChange={(e) => setPackForm((f) => ({ ...f, manifestUrl: e.target.value }))}
                />
              </Field>
            )}
            {packForm.sourceType === "sftp" && (
              <>
                <p className="text-xs text-muted">
                  Игрокам не отдаётся. После склада укажи HTTP zip.
                </p>
                <Field label="host">
                  <input
                    className="input"
                    value={packForm.host}
                    onChange={(e) => setPackForm((f) => ({ ...f, host: e.target.value }))}
                  />
                </Field>
                <div className="grid grid-cols-2 gap-3">
                  <Field label="port">
                    <input
                      className="input"
                      value={packForm.port}
                      onChange={(e) => setPackForm((f) => ({ ...f, port: e.target.value }))}
                    />
                  </Field>
                  <Field label="user">
                    <input
                      className="input"
                      value={packForm.user}
                      onChange={(e) => setPackForm((f) => ({ ...f, user: e.target.value }))}
                    />
                  </Field>
                </div>
                <Field label="path">
                  <input
                    className="input"
                    value={packForm.path}
                    onChange={(e) => setPackForm((f) => ({ ...f, path: e.target.value }))}
                  />
                </Field>
                <Field label="пароль">
                  <input
                    className="input"
                    type="password"
                    value={packForm.password}
                    onChange={(e) => setPackForm((f) => ({ ...f, password: e.target.value }))}
                  />
                </Field>
              </>
            )}
            <label className="flex items-center gap-2 text-sm text-muted min-h-11">
              <input
                type="checkbox"
                className="accent-[var(--color-accent)] h-4 w-4"
                checked={packForm.published}
                onChange={(e) => setPackForm((f) => ({ ...f, published: e.target.checked }))}
              />
              Показывать в лаунчере
            </label>
            <Field label="Доступ (ACL)">
              <select
                className="select"
                value={packForm.accessMode}
                onChange={(e) =>
                  setPackForm((f) => ({
                    ...f,
                    accessMode: e.target.value as "open" | "whitelist" | "blacklist",
                  }))
                }
              >
                <option value="open">open — всем</option>
                <option value="whitelist">whitelist — только ники ниже</option>
                <option value="blacklist">blacklist — все кроме ников ниже</option>
              </select>
            </Field>
            {packForm.accessMode !== "open" && (
              <Field label="Ники (через запятую)">
                <input
                  className="input"
                  value={packForm.aclNicknames}
                  onChange={(e) => setPackForm((f) => ({ ...f, aclNicknames: e.target.value }))}
                  placeholder="Alice, Bob"
                />
              </Field>
            )}
            <div className="flex justify-end gap-2 pt-2">
              <button type="button" className="btn btn-ghost" onClick={() => setOpenPack(false)}>
                Отмена
              </button>
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "…" : "Сохранить"}
              </button>
            </div>
          </form>
        </ModalShell>
      )}

      {openServer && (
        <ModalShell
          title={editServerId ? "Изменить сервер" : "Добавить сервер"}
          onClose={() => setOpenServer(false)}
        >
          <form onSubmit={saveServer} className="space-y-3">
            <Field label="Название">
              <input
                className="input"
                required
                value={serverForm.name}
                onChange={(e) => setServerForm((f) => ({ ...f, name: e.target.value }))}
              />
            </Field>
            <Field label="Иконка (URL)">
              <input
                className="input"
                value={serverForm.iconUrl}
                onChange={(e) => setServerForm((f) => ({ ...f, iconUrl: e.target.value }))}
              />
            </Field>
            <div className="grid grid-cols-[1fr_7rem] gap-3">
              <Field label="IP / хост">
                <input
                  className="input"
                  value={serverForm.address}
                  onChange={(e) => setServerForm((f) => ({ ...f, address: e.target.value }))}
                />
              </Field>
              <Field label="Порт">
                <input
                  className="input"
                  value={serverForm.port}
                  onChange={(e) => setServerForm((f) => ({ ...f, port: e.target.value }))}
                />
              </Field>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Тип">
                <select
                  className="select"
                  value={serverForm.kind}
                  onChange={(e) =>
                    setServerForm((f) => ({
                      ...f,
                      kind: e.target.value as "owyx" | "community",
                    }))
                  }
                >
                  <option value="owyx">owyx</option>
                  <option value="community">community</option>
                </select>
              </Field>
              <Field label="Пак">
                <select
                  className="select"
                  value={serverForm.packId}
                  onChange={(e) => {
                    const pack = packs.find((p) => p.id === e.target.value);
                    setServerForm((f) => ({
                      ...f,
                      packId: e.target.value,
                      minecraft: pack?.minecraft || f.minecraft,
                      loader: pack?.loader || f.loader,
                    }));
                  }}
                >
                  <option value="">без пака</option>
                  {packs.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </Field>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Minecraft">
                <SearchableSelect
                  listId="server-mc-versions"
                  value={serverForm.minecraft}
                  options={MC_VERSION_SUGGESTIONS}
                  placeholder="Search version…"
                  onChange={(minecraft) => setServerForm((f) => ({ ...f, minecraft }))}
                />
              </Field>
              <Field label="Лоадер">
                <SearchableSelect
                  listId="server-loaders"
                  value={serverForm.loader}
                  options={LOADERS}
                  placeholder="Search loader…"
                  onChange={(loader) => setServerForm((f) => ({ ...f, loader }))}
                />
              </Field>
            </div>
            <label className="flex items-center gap-2 text-sm text-muted min-h-11">
              <input
                type="checkbox"
                className="accent-[var(--color-accent)] h-4 w-4"
                checked={serverForm.requiresAccount}
                onChange={(e) =>
                  setServerForm((f) => ({ ...f, requiresAccount: e.target.checked }))
                }
              />
              Нужен аккаунт Owyx
            </label>
            <label className="flex items-center gap-2 text-sm text-muted min-h-11">
              <input
                type="checkbox"
                className="accent-[var(--color-accent)] h-4 w-4"
                checked={serverForm.published}
                onChange={(e) => setServerForm((f) => ({ ...f, published: e.target.checked }))}
              />
              Показывать в лаунчере
            </label>
            <Field label="Доступ (ACL)">
              <select
                className="select"
                value={serverForm.accessMode}
                onChange={(e) =>
                  setServerForm((f) => ({
                    ...f,
                    accessMode: e.target.value as "open" | "whitelist" | "blacklist",
                  }))
                }
              >
                <option value="open">open — всем</option>
                <option value="whitelist">whitelist — только ники ниже</option>
                <option value="blacklist">blacklist — все кроме ников ниже</option>
              </select>
            </Field>
            {serverForm.accessMode !== "open" && (
              <Field label="Ники (через запятую)">
                <input
                  className="input"
                  value={serverForm.aclNicknames}
                  onChange={(e) => setServerForm((f) => ({ ...f, aclNicknames: e.target.value }))}
                  placeholder="Alice, Bob"
                />
              </Field>
            )}
            <div className="flex justify-end gap-2 pt-2">
              <button type="button" className="btn btn-ghost" onClick={() => setOpenServer(false)}>
                Отмена
              </button>
              <button type="submit" disabled={busy} className="btn btn-primary">
                {busy ? "…" : "Сохранить"}
              </button>
            </div>
          </form>
        </ModalShell>
      )}
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="field">
      <span className="field-label">{label}</span>
      {children}
    </div>
  );
}

function ModalShell({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => {
    setMounted(true);
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, []);

  if (!mounted) return null;

  return createPortal(
    <div
      className="fixed inset-0 z-[80] flex items-center justify-center p-4 bg-black/65"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="w-full max-w-xl max-h-[min(90vh,44rem)] overflow-y-auto rounded-2xl border border-line bg-panel p-5 sm:p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="catalog-modal-title"
      >
        <h3 id="catalog-modal-title" className="font-display text-lg font-bold tracking-tight mb-4">
          {title}
        </h3>
        {children}
      </div>
    </div>,
    document.body
  );
}
