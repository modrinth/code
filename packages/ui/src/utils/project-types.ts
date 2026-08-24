export const PROJECT_TYPE_ORDER = [
	'mod',
	'resourcepack',
	'datapack',
	'shader',
	'modpack',
	'plugin',
	'server',
	'collection',
] as const

export type OrderedProjectType = (typeof PROJECT_TYPE_ORDER)[number]

const PROJECT_TYPE_SORT_ALIASES: Record<string, string> = {
	minecraft_java_server: 'server',
	shaderpack: 'shader',
}

export function getProjectTypeSortIndex(type: string): number {
	const normalized = PROJECT_TYPE_SORT_ALIASES[type] ?? type
	const index = (PROJECT_TYPE_ORDER as readonly string[]).indexOf(normalized)
	return index === -1 ? PROJECT_TYPE_ORDER.length : index
}

export function compareProjectTypes(a: string, b: string): number {
	return getProjectTypeSortIndex(a) - getProjectTypeSortIndex(b)
}

export function sortProjectTypes<T extends string>(types: Iterable<T>): T[] {
	return [...types].sort(compareProjectTypes)
}
