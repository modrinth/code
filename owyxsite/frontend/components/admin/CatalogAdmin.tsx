"use client";

import { useCallback, useEffect, useState, type ReactNode } from "react";

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
  sortOrder: number;
};

const SOURCE_TYPES: { id: SourceType; label: string }[] = [
  { id: "http_zip", label: "HTTP zip" },
  { id: "http_manifest", label: "HTTP манифест" },
  { id: "google_drive", label: "Google Drive" },
  { id: "mrpack", label: "mrpack" },
  { id: "sftp", label: "SFTP (только админ)" },
  { id: "local_ingest", label: "Локальная заливка" },
];

const LOADERS = ["vanilla", "fabric", "forge", "neoforge", "quilt"];

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
};

const emptyServer = {
  name: "",
  iconUrl: "",
  address: "play.owyx.site",
  port: "25565",
  kind: "owyx" as "owyx" | "community",
  packId: "",
  minecraft: "1.21.1",
  loader: "vanilla",
  requiresAccount: false,
  published: true,
  sortOrder: "0",
};

export default function CatalogAdmin({
  authHeaders,
  showMessage,
}: {
  authHeaders: () => Record<string, string>;
  showMessage: (t: string, k: "success" | "error") => void;
}) {
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
    });
    setOpenPack(true);
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
      sortOrder: String(s.sortOrder ?? 0),
    });
    setOpenServer(true);
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
      showMessage(editPackId ? "Пак обновлён" : "Пак создан", "success");
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
      showMessage(editServerId ? "Сервер обновлён" : "Сервер создан", "success");
      setOpenServer(false);
      await load();
    } catch {
      showMessage("Ошибка соединения", "error");
    } finally {
      setBusy(false);
    }
  }

  async function togglePack(p: PackRow) {
    const res = await fetch(`/api/admin/packs/${p.id}`, {
      method: "PUT",
      headers: authHeaders(),
      body: JSON.stringify({ published: !p.published }),
    });
    if (res.ok) await load();
    else showMessage("Не удалось обновить пак", "error");
  }

  async function toggleServer(s: ServerRow) {
    const res = await fetch(`/api/admin/servers/${s.id}`, {
      method: "PUT",
      headers: authHeaders(),
      body: JSON.stringify({ published: !s.published }),
    });
    if (res.ok) await load();
    else showMessage("Не удалось обновить сервер", "error");
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
          <h2 className="font-display text-lg font-bold tracking-tight">Каталог лаунчера</h2>
          <p className="mt-1 text-sm text-muted">
            Серверы и сборки, которые видит лаунчер. SFTP — только для админа.
          </p>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            className={`cabinet-rail-item ${tab === "servers" ? "is-active" : ""}`}
            onClick={() => setTab("servers")}
          >
            Серверы
          </button>
          <button
            type="button"
            className={`cabinet-rail-item ${tab === "packs" ? "is-active" : ""}`}
            onClick={() => setTab("packs")}
          >
            Сборки
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
                  className="flex flex-wrap items-center gap-3 rounded-xl border border-line px-3 py-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-text truncate">{s.name}</p>
                    <p className="text-xs text-muted truncate font-mono">
                      {s.address}:{s.port}
                      <span className="text-muted"> · </span>
                      {s.minecraft || "—"} {s.loader || ""}
                    </p>
                  </div>
                  <span className={`badge ${s.published ? "badge-accent" : ""}`}>
                    {s.published ? "опубл." : "скрыт"}
                  </span>
                  <button
                    type="button"
                    className="btn btn-secondary btn-sm"
                    onClick={() => startEditServer(s)}
                  >
                    Изменить
                  </button>
                  <button
                    type="button"
                    className="btn btn-ghost btn-sm"
                    onClick={() => void toggleServer(s)}
                  >
                    {s.published ? "Скрыть" : "Показать"}
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
                  className="flex flex-wrap items-center gap-3 rounded-xl border border-line px-3 py-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-text truncate">{p.name}</p>
                    <p className="text-xs text-muted truncate">
                      {p.minecraft} {p.loader} · {p.sourceType}
                    </p>
                  </div>
                  <span className={`badge ${p.published ? "badge-accent" : ""}`}>
                    {p.published ? "опубл." : "скрыт"}
                  </span>
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
                    className="btn btn-ghost btn-sm"
                    onClick={() => void togglePack(p)}
                  >
                    {p.published ? "Скрыть" : "Показать"}
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
                <input
                  className="input"
                  value={packForm.minecraft}
                  onChange={(e) => setPackForm((f) => ({ ...f, minecraft: e.target.value }))}
                />
              </Field>
              <Field label="Лоадер">
                <select
                  className="select"
                  value={packForm.loader}
                  onChange={(e) => setPackForm((f) => ({ ...f, loader: e.target.value }))}
                >
                  {LOADERS.map((l) => (
                    <option key={l} value={l}>
                      {l}
                    </option>
                  ))}
                </select>
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
              Опубликован
            </label>
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
                <input
                  className="input"
                  value={serverForm.minecraft}
                  onChange={(e) => setServerForm((f) => ({ ...f, minecraft: e.target.value }))}
                />
              </Field>
              <Field label="Лоадер">
                <select
                  className="select"
                  value={serverForm.loader}
                  onChange={(e) => setServerForm((f) => ({ ...f, loader: e.target.value }))}
                >
                  {LOADERS.map((l) => (
                    <option key={l} value={l}>
                      {l}
                    </option>
                  ))}
                </select>
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
              Опубликован
            </label>
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
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/65 py-10 px-4"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="w-full max-w-xl rounded-2xl border border-line bg-panel p-5 sm:p-6 shadow-xl"
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
    </div>
  );
}
