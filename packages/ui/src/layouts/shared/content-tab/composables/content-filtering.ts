import type { ClientWarningType, ContentItem, ContentWarningType } from '../types'

const CLIENT_ONLY_ENVIRONMENTS = new Set(['client_only', 'singleplayer_only'])

export function isClientOnlyEnvironment(env?: string | null): boolean {
	return !!env && CLIENT_ONLY_ENVIRONMENTS.has(env)
}

export function getClientWarningType(
	item: ContentItem,
	showEnvironmentWarnings = false,
): ClientWarningType | null {
	if (item.pack_client_retained) return 'retained'
	if (item.pack_client_depends) return 'depends'
	if (showEnvironmentWarnings && isClientOnlyEnvironment(item.environment)) return 'environment'
	return null
}

export function getContentWarningType(
	item: ContentItem,
	showEnvironmentWarnings = false,
): ContentWarningType | null {
	if (!item.enabledFor) return getClientWarningType(item, showEnvironmentWarnings)
	return item.enabledFor.warningKind ?? null
}
