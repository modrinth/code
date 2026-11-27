import type { ProjectFlagValue } from '@modrinth/api-client'

export function recordValue(value: ProjectFlagValue | undefined) {
	return value && typeof value === 'object' && !Array.isArray(value) ? value : undefined
}
