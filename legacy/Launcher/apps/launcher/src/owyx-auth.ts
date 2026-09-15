// Owyx account sign-in for the launcher UI.
//
// Two paths, same contract (see owyxsite/LAUNCHER_SITE_CONTRACT.md):
//   - Tauri app: call the Rust commands (owyx_login/owyx_me/owyx_logout). The
//     JWT is stored securely in the launcher data dir and never touches the UI.
//   - Browser dev preview (npm run dev): go through the Vite proxy /proxy/owyx
//     to avoid CORS. The token is kept in localStorage (dev only).
//
// login() rejects with a stable error CODE that the UI maps to a localized,
// foolproof message: "network" | "invalid_credentials" | "account_inactive"
// | "server" | "no_session".

import { invoke } from "@tauri-apps/api/core";

export type OwyxSession = {
  nickname: string;
  email: string;
  role: string;
  trustLevel: number;
  banned: boolean;
  serverAccess: boolean;
  accessReason: string;
  skinUrl: string | null;
  skinModel: string;
  baseUrl: string;
};

// Vite dev proxy target (configured in vite.config.ts) -> backend :3001.
const DEV_PROXY = "/proxy/owyx";
const DEV_SESSION_PREFIX = "owyx.session.";

function code(e: unknown): string {
  const s = typeof e === "string" ? e : (e as { message?: string })?.message || String(e);
  return s || "server";
}

async function readJson(res: Response): Promise<Record<string, unknown>> {
  try {
    return (await res.json()) as Record<string, unknown>;
  } catch {
    return {};
  }
}

function statusToCode(status: number): string {
  if (status === 400 || status === 401) return "invalid_credentials";
  if (status === 403) return "account_inactive";
  return "server";
}

function normalizeMe(data: Record<string, unknown>, baseUrl: string): OwyxSession {
  const user = (data.user ?? {}) as Record<string, unknown>;
  const cosmetics = (data.cosmetics ?? {}) as Record<string, unknown>;
  return {
    nickname: String(user.nickname ?? ""),
    email: String(user.email ?? ""),
    role: String(user.role ?? "user"),
    trustLevel: Number(user.trustLevel ?? 0),
    banned: Boolean(user.banned),
    serverAccess: Boolean(data.serverAccess),
    accessReason: String(data.accessReason ?? ""),
    skinUrl: (data.skinUrl as string) ?? null,
    skinModel: String(cosmetics.skinModel ?? "classic"),
    baseUrl,
  };
}

/** Sign in with email + password, then load the launcher profile. */
export async function owyxLogin(
  inTauri: boolean,
  baseUrl: string | null,
  email: string,
  password: string,
): Promise<OwyxSession> {
  if (inTauri) {
    try {
      return await invoke<OwyxSession>("owyx_login", { baseUrl, email, password });
    } catch (e) {
      throw code(e);
    }
  }

  // Browser dev path via the Vite proxy.
  const base = DEV_PROXY;
  let loginRes: Response;
  try {
    loginRes = await fetch(`${base}/api/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });
  } catch {
    throw "network";
  }
  if (!loginRes.ok) throw statusToCode(loginRes.status);

  const loginData = await readJson(loginRes);
  const token = String(loginData.token ?? "");
  if (!token) throw "server";

  let meRes: Response;
  try {
    meRes = await fetch(`${base}/api/launcher/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch {
    throw "network";
  }
  if (!meRes.ok) throw "server";

  const session = normalizeMe(await readJson(meRes), base);
  try {
    localStorage.setItem(
      `${DEV_SESSION_PREFIX}${session.nickname}`,
      JSON.stringify({ token, baseUrl: base }),
    );
  } catch {
    /* ignore storage errors in dev */
  }
  return session;
}

/** Refresh a stored session (used when switching to an Owyx profile). */
export async function owyxMe(inTauri: boolean, nickname: string): Promise<OwyxSession> {
  if (inTauri) {
    try {
      return await invoke<OwyxSession>("owyx_me", { nickname });
    } catch (e) {
      throw code(e);
    }
  }
  const raw = localStorage.getItem(`${DEV_SESSION_PREFIX}${nickname}`);
  if (!raw) throw "no_session";
  const { token, baseUrl } = JSON.parse(raw) as { token: string; baseUrl: string };
  let res: Response;
  try {
    res = await fetch(`${baseUrl}/api/launcher/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch {
    throw "network";
  }
  if (!res.ok) {
    if (res.status === 401 || res.status === 403) {
      try {
        localStorage.removeItem(`${DEV_SESSION_PREFIX}${nickname}`);
      } catch {
        /* ignore */
      }
      throw "no_session";
    }
    throw "server";
  }
  return normalizeMe(await readJson(res), baseUrl);
}

/** Admin API using the stored JWT. Path must start with /api/admin/. */
export async function owyxAdminFetch(
  inTauri: boolean,
  nickname: string,
  method: string,
  path: string,
  body?: Record<string, unknown> | null,
): Promise<Record<string, unknown>> {
  if (inTauri) {
    try {
      return await invoke<Record<string, unknown>>("owyx_admin_request", {
        nickname,
        method,
        path,
        body: body ?? null,
      });
    } catch (e) {
      throw code(e);
    }
  }
  const raw = localStorage.getItem(`${DEV_SESSION_PREFIX}${nickname}`);
  if (!raw) throw "no_session";
  const { token, baseUrl } = JSON.parse(raw) as { token: string; baseUrl: string };
  let res: Response;
  try {
    res = await fetch(`${baseUrl}${path}`, {
      method,
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: body && method !== "GET" ? JSON.stringify(body) : undefined,
    });
  } catch {
    throw "network";
  }
  const data = await readJson(res);
  if (!res.ok) {
    if (res.status === 401 || res.status === 403) throw "no_session";
    throw String(data.error ?? "server");
  }
  return data;
}

export type SkinApplyResult = {
  applied: boolean;
  path?: string | null;
  model: string;
  message: string;
};

export async function owyxApplySkin(
  inTauri: boolean,
  nickname: string,
  instanceId?: string | null,
): Promise<SkinApplyResult> {
  if (inTauri) {
    try {
      return await invoke<SkinApplyResult>("apply_owyx_skin", {
        nickname,
        instanceId: instanceId ?? null,
      });
    } catch (e) {
      throw code(e);
    }
  }
  return { applied: false, path: null, model: "classic", message: "browser" };
}

/** Forget a stored session (logout / profile delete). Never throws. */
export async function owyxLogout(inTauri: boolean, nickname: string): Promise<void> {
  if (inTauri) {
    try {
      await invoke("owyx_logout", { nickname });
    } catch {
      /* ignore */
    }
    return;
  }
  try {
    localStorage.removeItem(`${DEV_SESSION_PREFIX}${nickname}`);
  } catch {
    /* ignore */
  }
}
