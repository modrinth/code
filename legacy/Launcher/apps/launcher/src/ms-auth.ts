/**
 * Microsoft account auth pipeline (Path A when MICROSOFT_CLIENT_ID is set).
 *
 * Phases from Rust:
 * - unconfigured — no CLIENT_ID
 * - waiting — real Azure device-code (userCode + verificationUri)
 * - success — Minecraft profile + opaque authBlob
 * - error — Azure / Xbox / Minecraft failure
 */

import { invoke } from "@tauri-apps/api/core";

export type MsAuthPhase =
  | "idle"
  | "starting"
  | "waiting"
  | "scaffold"
  | "not_implemented"
  | "success"
  | "error"
  | "unconfigured";

export type MsAuthStatus = {
  phase: MsAuthPhase;
  message?: string;
  userCode?: string;
  verificationUri?: string;
  gamertag?: string;
  /** Opaque ms1:… blob from Path A success */
  authBlob?: string;
};

export async function msAuthStart(): Promise<MsAuthStatus> {
  try {
    return await invoke<MsAuthStatus>("ms_auth_start");
  } catch (e) {
    const msg = typeof e === "string" ? e : (e as { message?: string })?.message || String(e);
    if (/MICROSOFT_CLIENT_ID|not configured|unconfigured/i.test(msg)) {
      return { phase: "unconfigured", message: msg };
    }
    return { phase: "error", message: msg };
  }
}

export async function msAuthPoll(): Promise<MsAuthStatus> {
  try {
    return await invoke<MsAuthStatus>("ms_auth_poll");
  } catch (e) {
    const msg = typeof e === "string" ? e : (e as { message?: string })?.message || String(e);
    return { phase: "error", message: msg };
  }
}

export async function msAuthCancel(): Promise<void> {
  try {
    await invoke("ms_auth_cancel");
  } catch {
    /* ignore */
  }
}
