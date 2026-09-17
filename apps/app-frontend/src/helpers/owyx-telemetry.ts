/**
 * Owyx launcher → site telemetry (opt-in via settings.telemetry).
 * Sends anonymous PC stats + sanitized errors. No emails / nicks / paths with usernames.
 */

import { arch, platform, version as osVersion } from '@tauri-apps/plugin-os'
import { getVersion } from '@tauri-apps/api/app'

import {
	DEFAULT_OWYX_API_BASE,
	getOwyxClientKey,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { get as getSettings } from '@/helpers/settings'

const INSTALL_ID_KEY = 'owyx.telemetry.installId'
const LAST_HEARTBEAT_KEY = 'owyx.telemetry.lastHeartbeat'

export type OwyxTelemetryKind =
	| 'session_start'
	| 'heartbeat'
	| 'error'
	| 'crash'
	| 'perf'
	| 'feature'

export type OwyxTelemetryEvent = {
	kind: OwyxTelemetryKind
	message?: string
	metadata?: Record<string, string | number | boolean | null>
}

function newUuid(): string {
	if (typeof crypto !== 'undefined' && crypto.randomUUID) {
		return crypto.randomUUID()
	}
	return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
		const r = (Math.random() * 16) | 0
		const v = c === 'x' ? r : (r & 0x3) | 0x8
		return v.toString(16)
	})
}

export function getOwyxInstallId(): string {
	try {
		let id = localStorage.getItem(INSTALL_ID_KEY)
		if (!id || !/^[0-9a-f-]{36}$/i.test(id)) {
			id = newUuid()
			localStorage.setItem(INSTALL_ID_KEY, id)
		}
		return id
	} catch {
		return newUuid()
	}
}

function sanitizeClientMessage(raw: string, max = 400): string {
	return raw
		.slice(0, max)
		.replace(/Bearer\s+[A-Za-z0-9\-._~+/]+=*/gi, 'Bearer [redacted]')
		.replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi, '[email]')
		.replace(/[\\/]Users[\\/][^\\/\s]+/gi, '/Users/[user]')
		.replace(/[\\/]home[\\/][^\\/\s]+/gi, '/home/[user]')
}

async function deviceSnapshot(appVersion: string) {
	let osName = 'unknown'
	let osVer: string | undefined
	let cpuArch: string | undefined
	try {
		osName = String(platform())
	} catch {
		/* ignore */
	}
	try {
		osVer = String(osVersion())
	} catch {
		/* ignore */
	}
	try {
		cpuArch = String(arch())
	} catch {
		/* ignore */
	}

	const nav = typeof navigator !== 'undefined' ? navigator : null
	const ramGb =
		nav && 'deviceMemory' in nav && typeof (nav as Navigator & { deviceMemory?: number }).deviceMemory === 'number'
			? (nav as Navigator & { deviceMemory?: number }).deviceMemory
			: undefined

	return {
		appVersion,
		os: osName,
		osVersion: osVer,
		arch: cpuArch,
		cpuCores: nav?.hardwareConcurrency || undefined,
		ramMb: ramGb ? Math.round(ramGb * 1024) : undefined,
		locale: nav?.language?.slice(0, 16) || undefined,
	}
}

async function postTelemetry(
	events: OwyxTelemetryEvent[],
	opts?: { authToken?: string | null },
): Promise<boolean> {
	if (!events.length) return false
	try {
		const settings = await getSettings()
		if (!settings?.telemetry) return false
	} catch {
		return false
	}
	const base = sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
	const appVersion = await getVersion().catch(() => 'unknown')
	const snap = await deviceSnapshot(appVersion)
	const installId = getOwyxInstallId()

	const body = {
		installId,
		events: events.map((e) => ({
			...snap,
			kind: e.kind,
			message: e.message ? sanitizeClientMessage(e.message) : undefined,
			metadata: e.metadata,
		})),
	}

	const headers: Record<string, string> = {
		Accept: 'application/json',
		'Content-Type': 'application/json',
	}
	const key = getOwyxClientKey()
	if (key) headers['X-Owyx-Client-Key'] = key
	if (opts?.authToken) headers.Authorization = `Bearer ${opts.authToken}`

	try {
		const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http')
		const res = await tauriFetch(`${base.replace(/\/$/, '')}/api/launcher/v1/telemetry`, {
			method: 'POST',
			headers,
			body: JSON.stringify(body),
			signal: AbortSignal.timeout(8000),
		})
		return res.ok || res.status === 202
	} catch {
		try {
			const res = await fetch(`${base.replace(/\/$/, '')}/api/launcher/v1/telemetry`, {
				method: 'POST',
				headers,
				body: JSON.stringify(body),
				signal: AbortSignal.timeout(8000),
			})
			return res.ok || res.status === 202
		} catch {
			return false
		}
	}
}

/** Fire-and-forget session start + daily heartbeat. Respects telemetry opt-in. */
export async function reportOwyxLauncherSession(opts?: {
	authToken?: string | null
	dev?: boolean
}): Promise<void> {
	const events: OwyxTelemetryEvent[] = [
		{
			kind: 'session_start',
			message: opts?.dev ? 'launcher session (dev)' : 'launcher session',
			metadata: { dev: Boolean(opts?.dev) },
		},
	]

	try {
		const last = Number(localStorage.getItem(LAST_HEARTBEAT_KEY) || 0)
		if (!last || Date.now() - last > 20 * 60 * 60 * 1000) {
			events.push({ kind: 'heartbeat', message: 'daily heartbeat' })
			localStorage.setItem(LAST_HEARTBEAT_KEY, String(Date.now()))
		}
	} catch {
		/* ignore */
	}

	await postTelemetry(events, { authToken: opts?.authToken })
}

/** Report a sanitized launcher/runtime error (opt-in). */
export async function reportOwyxLauncherError(
	message: string,
	opts?: { authToken?: string | null; metadata?: OwyxTelemetryEvent['metadata'] },
): Promise<void> {
	await postTelemetry(
		[
			{
				kind: 'error',
				message,
				metadata: opts?.metadata,
			},
		],
		{ authToken: opts?.authToken },
	)
}
