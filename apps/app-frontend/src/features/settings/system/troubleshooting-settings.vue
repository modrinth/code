<script setup lang="ts">
import {
	CheckIcon,
	CopyIcon,
	DownloadIcon,
	FolderOpenIcon,
	LoaderCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ref } from 'vue'

import {
	exportDebugInfo,
	exportingDebugInfo,
} from '@/components/ui/download-manager/debug-info-export'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_ads_debug_state } from '@/helpers/ads.js'
import { purge_cache_types } from '@/helpers/cache.js'
import { appSettingsKeys, get, set } from '@/helpers/settings.ts'
import { copyToClipboard, showAppDbBackupsFolder } from '@/helpers/utils.js'

const { addNotification, handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const appSettings = useAppSettings()
const queryClient = useQueryClient()
const clearCacheConfirmModal = ref<InstanceType<typeof ConfirmModalWrapper> | null>(null)
const copyingAdDetails = ref(false)
const copiedAdDetails = ref(false)

const copyDetailsMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (enabled: boolean) => {
		const settings = await get()
		const nextSettings = { ...settings, always_show_copy_details: enabled }
		await set(nextSettings)
		return nextSettings
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onSuccess: (settings) => {
		appSettings.alwaysShowCopyDetails = settings.always_show_copy_details
		queryClient.setQueryData(appSettingsKeys.all, settings)
	},
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

const messages = defineMessages({
	debugInfoTitle: {
		id: 'app.settings.resource-management.debug-info.title',
		defaultMessage: 'Debug info',
	},
	debugInfoDescription: {
		id: 'app.settings.resource-management.debug-info.description',
		defaultMessage:
			'Export app logs and diagnostic data to a ZIP file to share with Modrinth support.',
	},
	exportDebugInfo: {
		id: 'app.settings.resource-management.debug-info.export',
		defaultMessage: 'Export debug info',
	},
	reportAdTitle: {
		id: 'app.settings.troubleshooting.report-ad.title',
		defaultMessage: 'Report an ad',
	},
	reportAdDescription: {
		id: 'app.settings.troubleshooting.report-ad.description',
		defaultMessage:
			"Seen an inappropriate or offensive ad? Copy its details while it's showing and send them to <support>Modrinth Support</support>. We'll review it and remove it.",
	},
	copyAdDetails: {
		id: 'app.settings.troubleshooting.report-ad.copy',
		defaultMessage: 'Copy ad details',
	},
	copiedAdDetails: {
		id: 'app.settings.troubleshooting.report-ad.copied',
		defaultMessage: 'Copied',
	},
	noAdLoadedTitle: {
		id: 'app.settings.troubleshooting.report-ad.no-ad.title',
		defaultMessage: 'No ad found',
	},
	noAdLoadedText: {
		id: 'app.settings.troubleshooting.report-ad.no-ad.text',
		defaultMessage: 'Wait for the ad to load, then try again.',
	},
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
	if (appSettings.skipNonEssentialWarnings) {
		void clearCache()
		return
	}

	clearCacheConfirmModal.value?.show()
}

async function openDbBackupsFolder() {
	await showAppDbBackupsFolder().catch(handleError)
}

async function handleExportDebugInfo() {
	await exportDebugInfo(formatMessage(messages.exportDebugInfo)).catch(handleError)
}

function openSupport() {
	openUrl('https://support.modrinth.com')
}

async function handleCopyAdDetails() {
	copyingAdDetails.value = true

	try {
		const adReport = (await get_ads_debug_state()) as
			| ({ current_ad?: string } & Record<string, unknown>)
			| null

		if (!adReport) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.noAdLoadedTitle),
				text: formatMessage(messages.noAdLoadedText),
			})
			return
		}

		const { current_ad: currentAd, ...info } = adReport
		await copyToClipboard(
			`CURRENT_AD: ${currentAd ?? 'unknown'}\n\nINFO:\n${JSON.stringify(info, null, 2)}`,
		)

		copiedAdDetails.value = true
		setTimeout(() => {
			copiedAdDetails.value = false
		}, 3000)
	} catch (error) {
		handleError(error as Error)
	} finally {
		copyingAdDetails.value = false
	}
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
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.debugInfoTitle) }}
			</h2>
			<Button
				id="export-debug-info"
				class="w-fit"
				:disabled="exportingDebugInfo"
				:loading="exportingDebugInfo"
				@click="handleExportDebugInfo"
			>
				<LoaderCircleIcon
					v-if="exportingDebugInfo"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<DownloadIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.exportDebugInfo) }}
			</Button>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.debugInfoDescription) }}
			</p>
		</div>
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.reportAdTitle) }}
			</h2>
			<Button
				id="copy-ad-details"
				class="w-fit"
				:disabled="copyingAdDetails || copiedAdDetails"
				@click="handleCopyAdDetails"
			>
				<LoaderCircleIcon
					v-if="copyingAdDetails"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<CheckIcon v-else-if="copiedAdDetails" class="text-green" aria-hidden="true" />
				<CopyIcon v-else aria-hidden="true" />
				{{ formatMessage(copiedAdDetails ? messages.copiedAdDetails : messages.copyAdDetails) }}
			</Button>
			<p class="m-0 leading-tight text-secondary">
				<IntlFormatted :message-id="messages.reportAdDescription">
					<template #support="{ children }">
						<button
							type="button"
							class="inline cursor-pointer border-0 bg-transparent p-0 text-[length:inherit] leading-[inherit] text-blue hover:underline"
							@click="openSupport"
						>
							<component :is="() => children" />
						</button>
					</template>
				</IntlFormatted>
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
				:model-value="appSettings.alwaysShowCopyDetails"
				:disabled="copyDetailsMutation.isPending.value"
				:aria-label="formatMessage(messages.alwaysShowCopyDetailsTitle)"
				@update:model-value="copyDetailsMutation.mutate"
			/>
		</div>
	</div>
</template>
