// Site catalog client (servers + packs). Browser preview uses the Vite proxy;
// the packaged app goes through Rust so CORS / tauri:// origins do not matter.

import { invoke } from "@tauri-apps/api/core";

export type PackSourceType =
  | "http_zip"
  | "http_manifest"
  | "google_drive"
  | "mrpack"
  | "sftp"
  | "local_ingest";

export type CatalogPack = {
  id: string;
  name: string;
  minecraft: string;
  loader: string;
  iconUrl?: string | null;
  description?: string;
  sourceType: PackSourceType;
  downloadUrl?: string | null;
  sha256?: string | null;
  manifestUrl?: string | null;
  downloadAvailable?: boolean;
};

export type CatalogServer = {
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
  status?: { online?: boolean | null; players?: number | null; max?: number | null };
  pack?: CatalogPack | null;
};

export type CatalogLoadResult = {
  servers: CatalogServer[];
  packs: CatalogPack[];
  error?: string;
};

const DEV_PROXY = "/proxy/owyx";

async function readJson(res: Response): Promise<Record<string, unknown>> {
  try {
    return (await res.json()) as Record<string, unknown>;
  } catch {
    return {};
  }
}

export async function loadCatalog(inTauri: boolean): Promise<CatalogLoadResult> {
  if (inTauri) {
    try {
      return await invoke<CatalogLoadResult>("catalog_list");
    } catch (e) {
      return { servers: [], packs: [], error: String(e) };
    }
  }
  try {
    const [sRes, pRes] = await Promise.all([
      fetch(`${DEV_PROXY}/api/launcher/v1/servers`),
      fetch(`${DEV_PROXY}/api/launcher/v1/packs`),
    ]);
    if (!sRes.ok && !pRes.ok) {
      return { servers: [], packs: [], error: "network" };
    }
    const sData = sRes.ok ? await readJson(sRes) : {};
    const pData = pRes.ok ? await readJson(pRes) : {};
    return {
      servers: Array.isArray(sData.servers) ? (sData.servers as CatalogServer[]) : [],
      packs: Array.isArray(pData.packs) ? (pData.packs as CatalogPack[]) : [],
      error: sRes.ok ? undefined : "network",
    };
  } catch {
    return { servers: [], packs: [], error: "network" };
  }
}

export type CatalogInstallResult = {
  instanceId: string;
  ready: boolean;
  message: string;
};

export async function installCatalogServer(
  inTauri: boolean,
  server: CatalogServer,
): Promise<CatalogInstallResult> {
  if (!inTauri) {
    return {
      instanceId: `srv-${server.id}`,
      ready: false,
      message: "browser",
    };
  }
  return invoke<CatalogInstallResult>("install_catalog_server", {
    serverId: server.id,
    name: server.name,
    address: server.address,
    port: server.port,
    packId: server.packId ?? null,
    minecraft: server.minecraft ?? server.pack?.minecraft ?? "1.21.1",
    loader: server.loader ?? server.pack?.loader ?? "vanilla",
    downloadUrl: server.pack?.downloadUrl ?? null,
    sha256: server.pack?.sha256 ?? null,
    manifestUrl: server.pack?.manifestUrl ?? null,
    sourceType: server.pack?.sourceType ?? "http_zip",
  });
}

export type AdminServerInput = {
  name: string;
  address: string;
  port: number;
  kind?: string;
  packId?: string | null;
  minecraft?: string;
  loader?: string;
  requiresAccount?: boolean;
  published?: boolean;
  iconUrl?: string | null;
};

export type AdminPackInput = {
  name: string;
  minecraft: string;
  loader: string;
  sourceType: PackSourceType;
  url?: string;
  sha256?: string;
  manifestUrl?: string;
  description?: string;
  published?: boolean;
};
