<script setup lang="ts">
import { FolderOpenIcon, TrashIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, Toggle, useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { purge_cache_types } from '@/helpers/cache.js'
import { appSettingsKeys, get, set } from '@/helpers/settings.ts'
import { showAppDbBackupsFolder } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const appSettings = useAppSettings()
const queryClient = useQueryClient()
const clearCacheConfirmModal = ref<InstanceType<typeof ConfirmModalWrapper> | null>(null)
const alwaysShowCopyDetailsFlag = 'always_show_copy_details'

const copyDetailsMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (enabled: boolean) => {
		const settings = await get()
		const nextSettings = {
			...settings,
			feature_flags: { ...settings.feature_flags, [alwaysShowCopyDetailsFlag]: enabled },
		}
		await set(nextSettings)
		return nextSettings
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onSuccess: (settings) => {
		appSettings.featureFlags[alwaysShowCopyDetailsFlag] =
			settings.feature_flags[alwaysShowCopyDetailsFlag]
		queryClient.setQueryData(appSettingsKeys.all, settings)
	},
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

const messages = defineMessages({
	appCacheTitle: {
		id: 'app.settings.resource-management.app-cache.title',
		defaultMessage: 'App cache',
	},
	clearCache: {
		id: 'app.settings.troubleshooting.clear-cache',
		defaultMessage: 'Clear cache',
	},
	clearCacheConfirmTitle: {
		id: 'app.settings.troubleshooting.clear-cache.confirm.title',
		defaultMessage: 'Clear the app cache?',
	},
	clearCacheConfirmDescription: {
		id: 'app.settings.resource-management.app-cache.confirm.description',
		defaultMessage: 'The app may load more slowly until the cache is rebuilt.',
	},
	appCacheDescription: {
		id: 'app.settings.resource-management.app-cache.description',
		defaultMessage:
			'Clear cached data and download it again from Modrinth. The app may load more slowly until the cache is rebuilt.',
	},
	appDatabaseBackupsTitle: {
		id: 'app.settings.resource-management.app-database-backups.title',
		defaultMessage: 'App database backups',
	},
	openBackupsFolder: {
		id: 'app.settings.resource-management.app-database-backups.open-folder',
		defaultMessage: 'Open backups folder',
	},
	appDatabaseBackupsDescription: {
		id: 'app.settings.resource-management.app-database-backups.description',
		defaultMessage:
			'Backups of important app data are stored here in case you need to recover them later.',
	},
	alwaysShowCopyDetailsTitle: {
		id: 'app.settings.resource-management.always-show-copy-details.title',
		defaultMessage: 'Always show copy details',
	},
	alwaysShowCopyDetailsDescription: {
		id: 'app.settings.resource-management.always-show-copy-details.description',
		defaultMessage:
			'Show the Copy details action while an install is queued or running. It is always available for failed or interrupted installs.',
	},
})

async function clearCache() {
	await purge_cache_types([
		'project',
		'project_v3',
		'version',
		'user',
		'team',
		'organization',
		'file',
		'loader_manifest',
		'minecraft_manifest',
		'categories',
		'report_types',
		'loaders',
		'game_versions',
		'donation_platforms',
		'file_hash',
		'file_update',
		'search_results',
		'search_results_v3',
	]).catch(handleError)
}

function handleClearCacheClick() {
	if (appSettings.getFeatureFlag('skip_non_essential_warnings')) {
		void clearCache()
		return
	}

	clearCacheConfirmModal.value?.show()
}

async function openDbBackupsFolder() {
	await showAppDbBackupsFolder().catch(handleError)
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-2.5">
			<ConfirmModalWrapper
				ref="clearCacheConfirmModal"
				:title="formatMessage(messages.clearCacheConfirmTitle)"
				:description="formatMessage(messages.clearCacheConfirmDescription)"
				:has-to-type="false"
				:proceed-label="formatMessage(messages.clearCache)"
				:show-ad-on-close="false"
				@proceed="clearCache"
			/>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.appCacheTitle) }}
			</h2>
			<Button id="clear-cache" class="w-fit" @click="handleClearCacheClick">
				<TrashIcon aria-hidden="true" />
				{{ formatMessage(messages.clearCache) }}
			</Button>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.appCacheDescription) }}
			</p>
		</div>
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.appDatabaseBackupsTitle) }}
			</h2>
			<Button id="open-db-backups-folder" class="w-fit" @click="openDbBackupsFolder">
				<FolderOpenIcon aria-hidden="true" />
				{{ formatMessage(messages.openBackupsFolder) }}
			</Button>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.appDatabaseBackupsDescription) }}
			</p>
		</div>
		<div class="flex items-center justify-between gap-4">
			<div>
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.alwaysShowCopyDetailsTitle) }}
				</h2>
				<p class="m-0 mt-1">
					{{ formatMessage(messages.alwaysShowCopyDetailsDescription) }}
				</p>
			</div>
			<Toggle
				id="always-show-copy-details"
				:model-value="appSettings.getFeatureFlag(alwaysShowCopyDetailsFlag)"
				:disabled="copyDetailsMutation.isPending.value"
				:aria-label="formatMessage(messages.alwaysShowCopyDetailsTitle)"
				@update:model-value="copyDetailsMutation.mutate"
			/>
		</div>
	</div>
</template>
