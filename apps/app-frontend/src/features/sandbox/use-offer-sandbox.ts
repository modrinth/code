import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { appSettingsQueryOptions } from '@/helpers/settings'
import type { SandboxOverrides } from '@/helpers/types'

import { sandboxUnavailableReasonQueryOptions } from './queries'

export function useOfferSandbox(overrides?: MaybeRefOrGetter<SandboxOverrides | null | undefined>) {
	const settings = useQuery(appSettingsQueryOptions())
	const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())

	return computed(() => {
		const defaults = settings.data.value?.sandbox
		if (!defaults || !unavailableReason.isSuccess.value || unavailableReason.data.value) {
			return false
		}
		return (toValue(overrides)?.mode ?? defaults.mode) === 'never'
	})
}
