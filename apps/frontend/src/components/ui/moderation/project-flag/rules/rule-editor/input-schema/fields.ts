import type { ProjectFlagValue } from '@modrinth/api-client'
import Fuse from 'fuse.js'

type FieldEntries = [string, ProjectFlagValue][]
export type MatchingBranches = ReadonlyMap<ProjectFlagValue, FieldEntries>

export function createFieldSearch(fields: FieldEntries) {
	const fieldNames = new Set<string>()
	function collectNames(entries: FieldEntries, isArray = false) {
		for (const [name, value] of entries) {
			if (!isArray) fieldNames.add(name)
			if (value !== null && typeof value === 'object') {
				collectNames(Object.entries(value), Array.isArray(value))
			}
		}
	}
	collectNames(fields)
	const search = new Fuse([...fieldNames], { threshold: 0.3, ignoreLocation: true })

	return (query: string) => {
		const matches = new Set(search.search(query).map(({ item }) => item))
		const branches = new Map<ProjectFlagValue, FieldEntries>()
		function filterEntries(entries: FieldEntries, isArray = false): FieldEntries {
			return entries.filter(([name, value]) => {
				if (value !== null && typeof value === 'object') {
					const children = filterEntries(Object.entries(value), Array.isArray(value))
					if (children.length) branches.set(value, children)
				}
				return branches.has(value) || (!isArray && matches.has(name))
			})
		}
		return { entries: filterEntries(fields), branches }
	}
}

export function recordValue(value: ProjectFlagValue | undefined) {
	return value && typeof value === 'object' && !Array.isArray(value) ? value : undefined
}
