import { queryOptions } from '@tanstack/vue-query'

import {
	findFilteredJres,
	getJavaVersions,
	getJre,
	getMaxMemory,
} from '@/platform/app-lib/jre/commands'

export const javaKeys = {
	versions: ['system', 'java-versions'] as const,
	jre: (path: string) => ['system', 'jre', path] as const,
	detected: (version: number | null) => ['system', 'detected-jres', version] as const,
	maxMemory: ['system', 'memory', 'total'] as const,
}

export function javaVersionsQueryOptions() {
	return queryOptions({
		queryKey: javaKeys.versions,
		networkMode: 'always',
		queryFn: getJavaVersions,
		staleTime: 0,
	})
}

export function jreQueryOptions(path: string) {
	return queryOptions({
		queryKey: javaKeys.jre(path),
		networkMode: 'always',
		queryFn: () => getJre(path).catch(() => null),
		staleTime: Infinity,
	})
}

export function detectedJresQueryOptions(version: number | null) {
	return queryOptions({
		queryKey: javaKeys.detected(version),
		networkMode: 'always',
		queryFn: () => findFilteredJres(version),
		staleTime: 0,
	})
}

export function maxMemoryQueryOptions() {
	return queryOptions({
		queryKey: javaKeys.maxMemory,
		networkMode: 'always',
		queryFn: getMaxMemory,
		staleTime: Infinity,
		gcTime: Infinity,
	})
}
