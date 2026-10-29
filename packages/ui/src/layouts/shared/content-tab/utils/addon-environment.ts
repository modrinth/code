import type { Archon, Labrinth } from '@modrinth/api-client'

import type { ContentSide } from '../types'

export function friendlyAddonName(addon: Archon.Content.v1.Addon): string {
	if (addon.name) return addon.name
	if (addon.manifest?.name) return addon.manifest.name
	let cleanName = addon.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

export function getAddonEnvironment(
	addon: Archon.Content.v1.Addon,
): Labrinth.Projects.v3.Environment | undefined {
	const versionEnvironment = addon.version?.environment
	if (versionEnvironment && versionEnvironment !== 'unknown') return versionEnvironment
	return addon.manifest?.environment ?? versionEnvironment ?? undefined
}

export function hasDetectedEnvironment(addon: Archon.Content.v1.Addon) {
	const environment = getAddonEnvironment(addon)
	return environment !== undefined && environment !== 'unknown'
}

export function isPlayerOnlyContent(addon: Archon.Content.v1.Addon) {
	return addon.kind === 'resourcepack' || addon.kind === 'shader'
}

export function isEnabledForAnySide(addon: Archon.Content.v1.Addon) {
	return (!isPlayerOnlyContent(addon) && !addon.disabled_server) || !addon.disabled_player
}

export function isIncompatibleEnvironment(addon: Archon.Content.v1.Addon, side: ContentSide) {
	const environment = getAddonEnvironment(addon)
	return side === 'server'
		? environment === 'client_only' || environment === 'singleplayer_only'
		: environment === 'server_only' || environment === 'dedicated_server_only'
}

/** Falls back to the project's side requirements when the addon has no detected environment. */
export function isRequiredEnvironment(
	addon: Archon.Content.v1.Addon,
	side: ContentSide,
	project?: Pick<Labrinth.Projects.v2.Project, 'server_side' | 'client_side'>,
) {
	const environment = getAddonEnvironment(addon)
	if (!environment || environment === 'unknown') {
		return side === 'server'
			? project?.server_side === 'required'
			: project?.client_side === 'required'
	}
	return side === 'server'
		? environment === 'client_and_server' ||
				environment === 'server_only' ||
				environment === 'server_only_client_optional' ||
				environment === 'dedicated_server_only'
		: environment === 'client_and_server' ||
				environment === 'client_only' ||
				environment === 'client_only_server_optional' ||
				environment === 'singleplayer_only'
}
