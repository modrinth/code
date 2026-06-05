import type { Archon } from '@modrinth/api-client'

export type ServerLoader = Archon.Servers.v0.Loader | 'Bukkit'

export const loaderDisplayNames: Record<string, string> = {
	fabric: 'Fabric',
	neoforge: 'NeoForge',
	neo_forge: 'NeoForge',
	forge: 'Forge',
	quilt: 'Quilt',
	paper: 'Paper',
	purpur: 'Purpur',
	bukkit: 'Bukkit',
	vanilla: 'Vanilla',
}

export const formatLoaderLabel = (item: string) =>
	loaderDisplayNames[item] ?? item.charAt(0).toUpperCase() + item.slice(1)
