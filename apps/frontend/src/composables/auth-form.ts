import { getApiErrorMessage } from '@modrinth/api-client'
import type { LocationQueryValue } from 'vue-router'

import { getQueryString } from '@/composables/launcher-auth.ts'

export function authQueryString(
	value: LocationQueryValue | LocationQueryValue[] | null | undefined,
): string {
	return getQueryString(value) ?? ''
}

export function getAuthErrorMessage(error: unknown): string {
	return getApiErrorMessage(error)
}
