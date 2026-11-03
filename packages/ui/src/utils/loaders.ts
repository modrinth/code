import type { Archon } from '@modrinth/api-client'

export type ServerLoader = Archon.Servers.v0.Loader | 'Bukkit'

export const loaderDisplayNames: Record<string, string> = {
	fabric: 'Fabric',
	neoforge: 'NeoForge',
	neo_forge: 'NeoForge',
	forge: 'Forge',
	quilt: 'Quilt',
	paper: 'Paper',
	spigot: 'Spigot',
	purpur: 'Purpur',
	bukkit: 'Bukkit',
	vanilla: 'Vanilla',
}

export const formatLoaderLabel = (item: string) =>
	loaderDisplayNames[item] ?? item.charAt(0).toUpperCase() + item.slice(1)

export type ServerPlatform = {
	loader: string | null
	gameVersion: string | null
}

export function resolveServerPlatform(
	server: Pick<Archon.Servers.v0.Server, 'loader' | 'mc_version'> | null | undefined,
	content:
		| Pick<Archon.Content.v1.Addons, 'modloader' | 'game_version' | 'installing'>
		| null
		| undefined,
): ServerPlatform {
	const contentLoader = content?.modloader ? formatLoaderLabel(content.modloader) : null
	if (content?.installing && contentLoader) {
		return { loader: contentLoader, gameVersion: content.game_version }
	}
	return {
		loader: server?.loader ?? contentLoader,
		gameVersion: server?.mc_version ?? content?.game_version ?? null,
	}
}
