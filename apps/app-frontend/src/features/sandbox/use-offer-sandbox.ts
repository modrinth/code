import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { appSettingsQueryOptions } from '@/helpers/settings'
import type { SandboxOverrides } from '@/helpers/types'

import { sandboxUnavailableReasonQueryOptions } from './queries'

export function useOfferSandbox(overrides?: MaybeRefOrGetter<SandboxOverrides | null | undefined>) {
	const appSettings = useAppSettings()
	const settings = useQuery(appSettingsQueryOptions())
	const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())

	return computed(() => {
		if (!appSettings.getFeatureFlag('sandbox_install_ctas')) {
			return false
		}
		const defaults = settings.data.value?.sandbox
		if (!defaults || !unavailableReason.isSuccess.value || unavailableReason.data.value) {
			return false
		}
		return (toValue(overrides)?.mode ?? defaults.mode) === 'never'
	})
}
