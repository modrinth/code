import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, watch } from 'vue'

import {
	appSettingsKeys,
	appSettingsQueryOptions,
	cancel_directory_change,
	get,
	set,
} from '@/helpers/settings'
import { restartApp } from '@/helpers/utils.js'
import { getDefaultAppDirectory } from '@/platform/app-lib/settings/commands'

export function useAppDirectory() {
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const settingsQuery = useQuery(appSettingsQueryOptions())
	const defaultDirQuery = useQuery({
		queryKey: ['app-directory', 'default'],
		networkMode: 'always',
		queryFn: getDefaultAppDirectory,
		staleTime: Infinity,
	})

	watch([settingsQuery.error, defaultDirQuery.error], ([settingsError, defaultDirError]) => {
		const error = settingsError ?? defaultDirError
		if (error) handleError(error)
	})

	const defaultDir = computed(() => defaultDirQuery.data.value ?? null)

	const currentDir = computed(() => {
		const settings = settingsQuery.data.value
		return settings?.prev_custom_dir ?? settings?.custom_dir ?? null
	})

	const pendingDir = computed(() => {
		const settings = settingsQuery.data.value
		if (!settings?.prev_custom_dir || !settings.custom_dir) return null
		return settings.custom_dir === settings.prev_custom_dir ? null : settings.custom_dir
	})

	const settingsMutationOptions = {
		networkMode: 'always',
		mutationKey: appSettingsKeys.update,
		scope: { id: 'app-settings' },
		onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
		onError: handleError,
		onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
	} as const

	const changeMutation = useMutation({
		...settingsMutationOptions,
		mutationFn: async ({ directory, restartNow }: { directory: string; restartNow: boolean }) => {
			await set({ ...(await get()), custom_dir: directory })
			if (restartNow) await restartApp()
		},
	})

	const cancelMutation = useMutation({
		...settingsMutationOptions,
		mutationFn: () => cancel_directory_change(),
	})

	const busy = computed(() => changeMutation.isPending.value || cancelMutation.isPending.value)

	function changeDirectory(directory: string, restartNow: boolean) {
		changeMutation.mutate({ directory, restartNow })
	}

	function cancelMove() {
		cancelMutation.mutate()
	}

	function restartNow() {
		restartApp().catch(handleError)
	}

	return { currentDir, defaultDir, pendingDir, busy, changeDirectory, cancelMove, restartNow }
}
