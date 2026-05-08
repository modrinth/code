import type { LocationQueryValue, RouteRecordNameGeneric } from 'vue-router'

export function queryAsStringOrEmpty(query: LocationQueryValue | LocationQueryValue[]): string {
	return Array.isArray(query) ? (query[0] ?? '') : (query ?? '')
}

export function queryAsString(query: LocationQueryValue | LocationQueryValue[]): string | null {
	return Array.isArray(query) ? (query[0] ?? null) : (query ?? null)
}

export function queryAsStringArray(
	query: LocationQueryValue | LocationQueryValue[],
): string | null {
	if (query === undefined || query === null) {
		return []
	}
	return Array.isArray(query) ? query.map(String) : [String(query)]
}

export function routeNameAsString(name: RouteRecordNameGeneric | undefined): string | undefined {
	return name && typeof name === 'string' ? (name as string) : undefined
}
