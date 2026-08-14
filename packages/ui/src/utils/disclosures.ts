import type { Labrinth } from '@modrinth/api-client'

export const PROJECT_DISCLOSURE_TYPES = [
	'ai_content',
	'advertisements',
	'epilepsy_triggers',
	'system_interactions',
	'telemetry',
	'derivative_work',
	'paid_features',
	'archived',
] as const satisfies readonly Labrinth.Projects.v3.ProjectDisclosureType[]

export const AI_USAGE_TYPES = [
	'code',
	'assets',
	'text',
	'functionality',
] as const satisfies readonly Labrinth.Projects.v3.AiUsage[]

export const TELEMETRY_CONSENT_TYPES = [
	'opt_in',
	'opt_out',
	'always_active',
] as const satisfies readonly Labrinth.Projects.v3.TelemetryConsent[]

const ALL_CONTENT_PROJECT_TYPES = [
	'mod',
	'resourcepack',
	'datapack',
	'shader',
	'modpack',
	'plugin',
	'server',
] as const

export const DISCLOSURE_SUPPORTED_PROJECT_TYPES: Record<
	Labrinth.Projects.v3.ProjectDisclosureType,
	readonly (typeof ALL_CONTENT_PROJECT_TYPES)[number][]
> = {
	ai_content: ALL_CONTENT_PROJECT_TYPES,
	advertisements: ALL_CONTENT_PROJECT_TYPES,
	epilepsy_triggers: ALL_CONTENT_PROJECT_TYPES,
	derivative_work: ALL_CONTENT_PROJECT_TYPES,
	paid_features: ALL_CONTENT_PROJECT_TYPES,
	archived: ALL_CONTENT_PROJECT_TYPES,
	telemetry: ['mod', 'plugin', 'modpack', 'server'],
	system_interactions: ['mod', 'plugin', 'modpack'],
}

function normalizeProjectType(type: string): string {
	return type === 'minecraft_java_server' ? 'server' : type
}

export function isDisclosureCompatibleWithProjectTypes(
	disclosureType: Labrinth.Projects.v3.ProjectDisclosureType,
	projectTypes: readonly string[],
): boolean {
	const types = projectTypes.map(normalizeProjectType)
	return DISCLOSURE_SUPPORTED_PROJECT_TYPES[disclosureType].some((type) => types.includes(type))
}

export function isActiveDisclosure(
	disclosure: { deleted_at?: string | null } | null | undefined,
): boolean {
	return !!disclosure && disclosure.deleted_at == null
}

export function getActiveDisclosures<T extends { deleted_at?: string | null }>(
	disclosures: readonly T[] | null | undefined,
): T[] {
	return (disclosures ?? []).filter((disclosure) => isActiveDisclosure(disclosure))
}
