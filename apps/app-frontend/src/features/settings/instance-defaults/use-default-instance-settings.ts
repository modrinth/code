import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Ref, ref, watch } from 'vue'

import {
	type AppSettings,
	appSettingsKeys,
	appSettingsQueryOptions,
	get,
	set,
} from '@/helpers/settings'

export function useDefaultInstanceSettings<T extends object>(
	select: (settings: AppSettings) => T,
	serialize: (settings: T) => Partial<AppSettings> | null,
) {
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const settingsQuery = useQuery(appSettingsQueryOptions())
	const settings = ref<T | null>(null) as Ref<T | null>
	const mutation = useMutation({
		mutationKey: appSettingsKeys.update,
		scope: { id: 'app-settings' },
		mutationFn: async (changes: Partial<AppSettings>) => {
			await set({ ...(await get()), ...changes })
		},
		onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
		onError: handleError,
		onSettled: async () => {
			if (queryClient.isMutating({ mutationKey: appSettingsKeys.update }) === 1) {
				await queryClient.invalidateQueries({ queryKey: appSettingsKeys.all })
			}
		},
	})

	watch(
		settingsQuery.data,
		(value) => {
			if (
				!value ||
				(settings.value && queryClient.isMutating({ mutationKey: appSettingsKeys.update }))
			) {
				return
			}
			settings.value = select(value)
		},
		{ immediate: true, flush: 'sync' },
	)
	watch(settingsQuery.error, (error) => {
		if (error) handleError(error)
	})
	watch(
		settings,
		(value, previous) => {
			if (!value || value !== previous) return
			const changes = serialize(value)
			if (changes) mutation.mutate(changes)
		},
		{ deep: true },
	)

	return { settings, settingsQuery }
}
