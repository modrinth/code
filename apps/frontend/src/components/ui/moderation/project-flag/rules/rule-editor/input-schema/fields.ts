import type { ProjectFlagValue } from '@modrinth/api-client'

export function recordValue(value: ProjectFlagValue | undefined) {
	return value && typeof value === 'object' && !Array.isArray(value) ? value : undefined
}

export function fieldMatches(path: string, value: ProjectFlagValue, filter: string): boolean {
	if (!filter || path.toLowerCase().includes(filter)) return true
	if (typeof value === 'string' && /^scanned_files\.\d+\.found_patterns\.\d+$/.test(path)) {
		return value.toLowerCase().includes(filter)
	}
	if (!value || typeof value !== 'object') return false
	return Object.entries(value).some(([name, child]) =>
		fieldMatches(`${path}.${name}`, child, filter),
	)
}
