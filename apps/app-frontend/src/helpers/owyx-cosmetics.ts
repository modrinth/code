/**
 * Persist Owyx cosmetics (skin) to disk per LAUNCHER_SITE_CONTRACT.
 * Path: %USERPROFILE%/owyx/skins/{nickname}.png (Windows) / ~/owyx/skins/…
 */

import { homeDir, join } from '@tauri-apps/api/path'
import { mkdir, writeFile } from '@tauri-apps/plugin-fs'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

import { isAllowedOwyxAssetUrl } from '@/helpers/owyx-api'

function sanitizeNick(nick: string): string {
	return nick.replace(/[^A-Za-z0-9_\-.]/g, '_').slice(0, 32) || 'player'
}

export async function syncOwyxCosmeticsToDisk(
	nickname: string,
	cosmetics: Record<string, unknown> | null | undefined,
): Promise<void> {
	if (!cosmetics || !nickname) return
	const skinUrl = cosmetics.skinUrl
		? String(cosmetics.skinUrl)
		: cosmetics.skin_url
			? String(cosmetics.skin_url)
			: null
	if (!skinUrl) return
	if (!isAllowedOwyxAssetUrl(skinUrl)) return

	let res: Response
	try {
		res = await tauriFetch(skinUrl, { method: 'GET', signal: AbortSignal.timeout(15000) })
	} catch {
		res = await fetch(skinUrl, { method: 'GET', signal: AbortSignal.timeout(15000) })
	}
	if (!res.ok) return
	const buf = new Uint8Array(await res.arrayBuffer())
	if (buf.byteLength < 64) return

	const home = await homeDir()
	const dir = await join(home, 'owyx', 'skins')
	await mkdir(dir, { recursive: true })
	const file = await join(dir, `${sanitizeNick(nickname)}.png`)
	await writeFile(file, buf)
}
