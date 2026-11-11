import type { LocationQueryValue } from 'vue-router'

import { getQueryString } from '@/composables/launcher-auth.ts'

export function authQueryString(
	value: LocationQueryValue | LocationQueryValue[] | null | undefined,
): string {
	return getQueryString(value) ?? ''
}

export function getAuthErrorMessage(error: unknown): string {
	if (error && typeof error === 'object' && 'data' in error) {
		const description = (error as { data?: { description?: unknown } }).data?.description
		if (typeof description === 'string') {
			return description
		}
	}
	if (error instanceof Error) {
		return error.message
	}
	return String(error)
}
