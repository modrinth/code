import type { GameVersion } from '@modrinth/ui'
import dayjs from 'dayjs'

import type { ServerWorld, SingleplayerWorld, World } from '@/platform/app-lib/worlds/types'

const MINECRAFT_FORMATTING_CODE = /§[0-9A-FK-ORX]/gi
const FIRST_SERVER_QUICK_PLAY_VERSION = 'a1.0.5_01'
const FIRST_WORLD_QUICK_PLAY_VERSION = '23w14a'

export function getWorldIdentifier(world: World) {
	return world.type === 'singleplayer' ? world.path : world.address
}

export function sortWorlds(worlds: World[]) {
	worlds.sort((a, b) => {
		if (!a.last_played) return 1
		if (!b.last_played) return -1
		return dayjs(b.last_played).diff(dayjs(a.last_played))
	})
}

export function isSingleplayerWorld(world: World): world is SingleplayerWorld {
	return world.type === 'singleplayer'
}

export function isServerWorld(world: World): world is ServerWorld {
	return world.type === 'server'
}

export function stripMinecraftFormatting(text: string): string {
	return text.replace(MINECRAFT_FORMATTING_CODE, '').replace(/§/g, '')
}

export function getWorldDisplayName(world: World): string {
	const name = stripMinecraftFormatting(world.name).trim()
	if (name) return name
	return world.type === 'server' ? world.address : world.path
}

export function worldNameMatchesQuery(name: string, query: string): boolean {
	const normalizedQuery = query.trim().toLocaleLowerCase()
	if (!normalizedQuery) return true
	return (
		name.toLocaleLowerCase().includes(normalizedQuery) ||
		stripMinecraftFormatting(name).toLocaleLowerCase().includes(normalizedQuery)
	)
}

export function hasServerQuickPlaySupport(gameVersions: GameVersion[], currentVersion: string) {
	if (!gameVersions.length) return true

	const versionIndex = gameVersions.findIndex((v) => v.version === currentVersion)
	const targetIndex = gameVersions.findIndex((v) => v.version === FIRST_SERVER_QUICK_PLAY_VERSION)

	return versionIndex === -1 || targetIndex === -1 || versionIndex <= targetIndex
}

export function hasWorldQuickPlaySupport(gameVersions: GameVersion[], currentVersion: string) {
	if (!gameVersions.length) return false

	const versionIndex = gameVersions.findIndex((v) => v.version === currentVersion)
	const targetIndex = gameVersions.findIndex((v) => v.version === FIRST_WORLD_QUICK_PLAY_VERSION)

	return versionIndex !== -1 && targetIndex !== -1 && versionIndex <= targetIndex
}
