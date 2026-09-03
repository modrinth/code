<script setup lang="ts">
import { CircleSlashIcon, RefreshCwIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'
import { computed, nextTick, useTemplateRef } from 'vue'

import SyncSourceModal from '@/components/ui/settings/instances/SyncSourceModal.vue'

import {
	type SyncUpdateOption,
	syncUpdateOptions,
	useSyncInstancesUpdate,
} from './use-sync-instances-update'

const hasSeenUpdate = useLocalStorage('sync-instances-update-modal-shown', false)
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const sourceModal = useTemplateRef<InstanceType<typeof SyncSourceModal>>('sourceModal')
const { formatMessage } = useVIntl()
const {
	isOpen,
	globalOptionsQuery,
	allSynced,
	syncMutation,
	sourceOptions,
	sourceInstanceId,
	sources,
	sourcesLoading,
	sourcesError,
	chooseSource,
	retrySources,
} = useSyncInstancesUpdate()

const messages = defineMessages({
	badge: {
		id: 'app.sync-instances-update.badge',
		defaultMessage: 'New this update',
	},
	title: {
		id: 'app.sync-instances-update.title',
		defaultMessage: 'Sync your instances',
	},
	description: {
		id: 'app.sync-instances-update.description',
		defaultMessage:
			'You can now sync options like game settings, servers, resource packs, and more across your instances, so everything stays the same every time you play!',
	},
	manageLater: {
		id: 'app.sync-instances-update.manage-later',
		defaultMessage: 'You can enable syncing now and manage it later in your app settings.',
	},
	skip: {
		id: 'app.sync-instances-update.skip',
		defaultMessage: 'Skip',
	},
	syncAll: {
		id: 'app.sync-instances-update.sync-all',
		defaultMessage: 'Sync all',
	},
	game_options: {
		id: 'app.settings.synced-options.game-settings',
		defaultMessage: 'Sync game options',
	},
	multiplayer_servers: {
		id: 'app.settings.synced-options.multiplayer-servers',
		defaultMessage: 'Sync multiplayer servers',
	},
	command_history: {
		id: 'app.settings.synced-options.command-history',
		defaultMessage: 'Sync command history',
	},
	creative_hotbars: {
		id: 'app.sync-instances-update.creative-hotbars',
		defaultMessage: 'Sync creative hotbars',
	},
	resource_packs: {
		id: 'app.settings.synced-options.resource-packs',
		defaultMessage: 'Sync resource packs',
	},
	data_packs: {
		id: 'app.settings.synced-options.data-packs',
		defaultMessage: 'Sync data packs',
	},
	allSourcesDescription: {
		id: 'app.sync-instances-update.choose-source.all',
		defaultMessage: 'Choose the instance to use as the sync source for all six settings.',
	},
	game_options_source: {
		id: 'app.settings.synced-options.choose-sync-source.game-settings-description',
		defaultMessage: 'Choose which instance to copy game settings from.',
	},
	multiplayer_servers_source: {
		id: 'app.settings.synced-options.choose-sync-source.multiplayer-servers-description',
		defaultMessage: 'Pick the instance whose multiplayer servers become the shared copy.',
	},
	command_history_source: {
		id: 'app.settings.synced-options.choose-sync-source.command-history-description',
		defaultMessage: 'Pick the instance whose command history becomes the shared copy.',
	},
	creative_hotbars_source: {
		id: 'app.settings.synced-options.choose-sync-source.creative-hotbars-description',
		defaultMessage: 'Pick the instance whose saved creative hotbars become the shared copy.',
	},
	resource_packs_source: {
		id: 'app.sync-instances-update.choose-source.resource-packs',
		defaultMessage: 'Choose which instance to copy resource packs from.',
	},
	data_packs_source: {
		id: 'app.sync-instances-update.choose-source.data-packs',
		defaultMessage: 'Choose which instance to copy data packs from.',
	},
	loadError: {
		id: 'app.sync-instances-update.load-error',
		defaultMessage: 'Could not load your sync settings. Please try again.',
	},
	retry: {
		id: 'app.sync-instances-update.retry',
		defaultMessage: 'Try again',
	},
})

const sourceDescription = computed(() => {
	const option = sourceOptions.value[0]
	return formatMessage(
		option && sourceOptions.value.length === 1
			? messages[`${option}_source`]
			: messages.allSourcesDescription,
	)
})
const busy = computed(() => sourceOptions.value.length > 0 || syncMutation.isPending.value)
const controlsDisabled = computed(() => !globalOptionsQuery.data.value || busy.value)

function show() {
	isOpen.value = true
	modal.value?.show()
}

function showOnce() {
	if (!hasSeenUpdate.value) {
		show()
	}
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	isOpen.value = false
	hasSeenUpdate.value = true
}

async function openSourcePicker(options: readonly SyncUpdateOption[], retry = false) {
	if (syncMutation.isPending.value || (retry ? sourcesLoading.value : controlsDisabled.value)) {
		return
	}

	chooseSource(options)
	await nextTick()
	const result = await retrySources()
	if (!result.isSuccess || sources.value.some((source) => source.eligible)) {
		if (!retry) sourceModal.value?.show()
		return
	}

	try {
		await syncMutation.mutateAsync({ options: [...options], enabled: true })
		await nextTick()
		if (retry) sourceModal.value?.hide()
	} catch {
		return
	} finally {
		if (!retry) sourceOptions.value = []
	}
}

function toggleOption(option: SyncUpdateOption, enabled: boolean) {
	if (controlsDisabled.value) return
	if (enabled) {
		void openSourcePicker([option])
	} else {
		syncMutation.mutate({ options: [option], enabled: false })
	}
}

async function confirmSource() {
	if (
		syncMutation.isPending.value ||
		sourceOptions.value.length === 0 ||
		!sources.value.some((source) => source.id === sourceInstanceId.value && source.eligible)
	) {
		return
	}

	try {
		await syncMutation.mutateAsync({
			options: [...sourceOptions.value],
			enabled: true,
			baseInstanceId: sourceInstanceId.value,
		})
		await nextTick()
		sourceModal.value?.hide()
	} catch {
		return
	}
}

defineExpose({ show, showOnce, hide })
</script>

<template>
	<NewModal
		ref="modal"
		hide-header
		no-padding
		width="770px"
		max-width="calc(100vw - 2rem)"
		:aria-label="formatMessage(messages.title)"
		:disable-close="busy"
		:on-after-hide="handleHide"
		class="!overflow-y-auto !rounded-[20px]"
	>
		<div class="relative grid w-[768px] max-w-full grid-cols-2 max-[700px]:grid-cols-1">
			<IconButton
				type="quiet"
				size="sm"
				:label="formatMessage(commonMessages.closeButton)"
				class="!absolute right-4 top-4 z-10"
				:disabled="busy"
				@click="hide"
			>
				<XIcon />
			</IconButton>

			<section class="flex min-h-96 min-w-0 flex-col gap-6 bg-surface-3 p-8">
				<div
					class="flex h-8 w-fit items-center rounded-full border border-solid border-brand bg-brand-highlight px-2.5 text-sm font-medium leading-5 text-brand"
				>
					{{ formatMessage(messages.badge) }}
				</div>

				<div class="flex min-w-0 flex-col gap-4">
					<h2 class="m-0 text-2xl font-semibold leading-6 text-contrast">
						{{ formatMessage(messages.title) }}
					</h2>
					<p class="m-0 leading-6 text-primary">{{ formatMessage(messages.description) }}</p>
					<p v-if="!allSynced" class="m-0 leading-6 text-primary">
						{{ formatMessage(messages.manageLater) }}
					</p>
				</div>

				<div v-if="globalOptionsQuery.isError.value" class="flex flex-col items-start gap-2">
					<p role="alert" class="m-0 text-primary">{{ formatMessage(messages.loadError) }}</p>
					<Button @click="globalOptionsQuery.refetch()">{{ formatMessage(messages.retry) }}</Button>
				</div>

				<div class="mt-auto flex items-center gap-2.5">
					<Button v-if="!allSynced" size="lg" :disabled="busy" @click="hide">
						<CircleSlashIcon />
						{{ formatMessage(messages.skip) }}
					</Button>
					<Button
						v-if="allSynced"
						type="colored"
						color="brand"
						size="lg"
						:disabled="controlsDisabled"
						@click="hide"
					>
						{{ formatMessage(commonMessages.continueButton) }}
						<RightArrowIcon />
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						size="lg"
						:disabled="controlsDisabled"
						@click="openSourcePicker(syncUpdateOptions)"
					>
						<RefreshCwIcon />
						{{ formatMessage(messages.syncAll) }}
					</Button>
				</div>
			</section>

			<section
				class="flex min-w-0 items-center border-0 border-l border-solid border-surface-5 bg-surface-2 p-10 max-[700px]:border-l-0 max-[700px]:border-t"
			>
				<div class="flex w-full flex-col gap-5">
					<div
						v-for="option in syncUpdateOptions"
						:key="option"
						class="flex items-center justify-between gap-4"
					>
						<label
							:for="`update-sync-${option}`"
							class="text-base font-semibold leading-6 text-contrast"
						>
							{{ formatMessage(messages[option]) }}
						</label>
						<Toggle
							:id="`update-sync-${option}`"
							:model-value="globalOptionsQuery.data.value?.[option] ?? false"
							:disabled="controlsDisabled"
							class="!h-[26px] !w-[50px] !p-0.5 [&>span]:!m-0 [&>span]:!size-5"
							@update:model-value="(enabled) => toggleOption(option, enabled)"
						/>
					</div>
				</div>
			</section>
		</div>
	</NewModal>
	<SyncSourceModal
		ref="sourceModal"
		v-model="sourceInstanceId"
		:description="sourceDescription"
		:sources="sources"
		:loading="sourcesLoading"
		:pending="syncMutation.isPending.value"
		:error="sourcesError"
		@confirm="confirmSource"
		@close="sourceOptions = []"
		@retry="openSourcePicker(sourceOptions, true)"
	/>
</template>
