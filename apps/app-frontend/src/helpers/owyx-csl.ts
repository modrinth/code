/**
 * Wire Owyx site skins into Minecraft via CustomSkinLoader (open, GPL-3.0).
 * Docs: docs/owyx-skins-in-world.md
 */

import { homeDir, join } from '@tauri-apps/api/path'
import { mkdir, writeFile, exists } from '@tauri-apps/plugin-fs'

import { get_full_path } from '@/helpers/instance'
import { DEFAULT_OWYX_API_BASE, getStoredOwyxApiBase, sanitizeOwyxApiBase } from '@/helpers/owyx-api'

export const CUSTOM_SKIN_LOADER_MODRINTH = 'https://modrinth.com/mod/customskinloader'
export const CUSTOM_SKIN_LOADER_SOURCE = 'https://github.com/xfl03/MCCustomSkinLoader'
export const CUSTOM_SKIN_LOADER_LICENSE = 'GPL-3.0'

const SITE_ORIGIN = 'https://owyx.site'

function cslApiRoot(): string {
	// Prefer public website host so in-game CSL needs no X-Owyx-Client-Key.
	const api = sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
	if (api.includes('127.0.0.1') || api.includes('localhost')) {
		return `${api.replace(/\/+$/, '')}/api/csl/`
	}
	return `${SITE_ORIGIN}/api/csl/`
}

export function buildOwyxCustomSkinLoaderConfig(): Record<string, unknown> {
	const root = cslApiRoot()
	return {
		version: '14.25',
		buildNumber: 0,
		enable: true,
		enableCape: true,
		enableElytra: false,
		enableSkull: true,
		forceLoadAllTextures: false,
		enableTransparentSkin: true,
		forceUpdate: false,
		threadPoolSize: 8,
		enableLogStdOut: false,
		cacheExpiry: 120,
		loadlist: [
			{
				name: 'Owyx',
				type: 'Legacy',
				checkPNG: false,
				skin: `${root}skins/{USERNAME}.png`,
			},
			{
				name: 'OwyxCustomSkinAPI',
				type: 'CustomSkinAPI',
				root,
			},
			{
				name: 'LocalSkin',
				type: 'Legacy',
				skin: 'LocalSkin/skins/{USERNAME}.png',
				cape: 'LocalSkin/capes/{USERNAME}.png',
			},
			{
				name: 'Mojang',
				type: 'MojangAPI',
			},
		],
	}
}

/** Write CustomSkinLoader.json into an instance game directory. */
export async function writeOwyxCslConfigForInstance(instanceId: string): Promise<string> {
	const instancePath = await get_full_path(instanceId)
	const cslDir = await join(instancePath, 'CustomSkinLoader')
	await mkdir(cslDir, { recursive: true })
	const localSkins = await join(cslDir, 'LocalSkin', 'skins')
	await mkdir(localSkins, { recursive: true })
	const configPath = await join(cslDir, 'CustomSkinLoader.json')
	const json = `${JSON.stringify(buildOwyxCustomSkinLoaderConfig(), null, 2)}\n`
	await writeFile(configPath, new TextEncoder().encode(json))
	return configPath
}

/** Mirror ~/owyx/skins/{nick}.png into instance LocalSkin for offline fallback. */
export async function mirrorLocalOwyxSkinToInstance(
	instanceId: string,
	nickname: string,
): Promise<boolean> {
	const nick = nickname.replace(/[^A-Za-z0-9_\-.]/g, '_').slice(0, 32)
	if (!nick) return false
	const home = await homeDir()
	const src = await join(home, 'owyx', 'skins', `${nick}.png`)
	if (!(await exists(src))) return false
	const instancePath = await get_full_path(instanceId)
	const destDir = await join(instancePath, 'CustomSkinLoader', 'LocalSkin', 'skins')
	await mkdir(destDir, { recursive: true })
	const dest = await join(destDir, `${nick}.png`)
	// Read via fetch file:// is unreliable; use Tauri read if available through write from cosmetics sync.
	// Cosmetics sync already wrote src; for LocalSkin we re-fetch bytes through plugin-fs readFile.
	const { readFile } = await import('@tauri-apps/plugin-fs')
	const bytes = await readFile(src)
	await writeFile(dest, bytes)
	return true
}
