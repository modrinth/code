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

import { type SyncUpdateOption, syncUpdateOptions, useSyncInstancesUpdate } from './use-sync'

const hasSeenUpdate = useLocalStorage('sync-instances-update-modal-shown', false)
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const sourceModal = useTemplateRef<InstanceType<typeof SyncSourceModal>>('sourceModal')
const { formatMessage } = useVIntl()
const {
	globalOptionsQuery,
	allSynced,
	draftInitialized,
	draftOptions,
	syncMutation,
	sourceOptions,
	sourceInstanceId,
	sources,
	sourcesLoading,
	sourcesError,
	beginDraft,
	finishDraft,
	stageOptions,
	isInitiallyEnabled,
	applyDraft,
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
		defaultMessage:
			'Choose which instance to copy game settings, resource packs, command history, creative hotbars and multiplayer servers from. These settings are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	allSourcesTitle: {
		id: 'app.sync-instances-update.choose-source.all-title',
		defaultMessage: 'Choose sync source',
	},
	game_options_source: {
		id: 'app.settings.synced-options.choose-sync-source.game-settings-description',
		defaultMessage:
			'Choose which instance to copy your game settings from. These settings are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	game_options_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.game-settings-title',
		defaultMessage: 'Choose game settings source',
	},
	multiplayer_servers_source: {
		id: 'app.settings.synced-options.choose-sync-source.multiplayer-servers-description',
		defaultMessage:
			'Choose which instance to copy your multiplayer servers from. These servers are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	multiplayer_servers_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.multiplayer-servers-title',
		defaultMessage: 'Choose multiplayer servers source',
	},
	command_history_source: {
		id: 'app.settings.synced-options.choose-sync-source.command-history-description',
		defaultMessage:
			'Choose which instance to copy your command history from. This history is only used for the initial sync, and you can edit it from any instance afterward.',
	},
	command_history_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.command-history-title',
		defaultMessage: 'Choose command history source',
	},
	creative_hotbars_source: {
		id: 'app.settings.synced-options.choose-sync-source.creative-hotbars-description',
		defaultMessage:
			'Choose which instance to copy your saved creative hotbars from. These hotbars are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	creative_hotbars_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.creative-hotbars-title',
		defaultMessage: 'Choose creative hotbars source',
	},
	resource_packs_source: {
		id: 'app.sync-instances-update.choose-source.resource-packs',
		defaultMessage:
			'Choose which instance to copy your resource packs from. These packs are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	resource_packs_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.resource-packs-title',
		defaultMessage: 'Choose resource packs source',
	},
	data_packs_source: {
		id: 'app.sync-instances-update.choose-source.data-packs',
		defaultMessage:
			'Choose which instance to copy your data packs from. These packs are only used for the initial sync, and you can edit them from any instance afterward.',
	},
	data_packs_source_title: {
		id: 'app.settings.synced-options.choose-sync-source.data-packs-title',
		defaultMessage: 'Choose data packs source',
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
const sourceTitle = computed(() => {
	const option = sourceOptions.value[0]
	return formatMessage(
		option && sourceOptions.value.length === 1
			? messages[`${option}_source_title`]
			: messages.allSourcesTitle,
	)
})
const busy = computed(() => sourceOptions.value.length > 0 || syncMutation.isPending.value)
const controlsDisabled = computed(() => !draftInitialized.value || busy.value)
let allowHide = false

function show() {
	allowHide = false
	beginDraft()
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

function skip() {
	hasSeenUpdate.value = true
}

function handleHide() {
	finishDraft()
	skip()
}

function beforeHide() {
	if (allowHide) {
		allowHide = false
		return true
	}
	void applyAndHide()
	return false
}

async function applyAndHide() {
	if (syncMutation.isPending.value) return
	try {
		await applyDraft()
		allowHide = true
		await nextTick()
		if (modal.value?.hide() === false) allowHide = false
	} catch {
		return
	}
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
		stageOptions(options, true)
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
		if (isInitiallyEnabled(option)) {
			stageOptions([option], true)
		} else {
			void openSourcePicker([option])
		}
	} else {
		stageOptions([option], false)
	}
}

async function confirmSource() {
	if (
		sourceOptions.value.length === 0 ||
		!sources.value.some((source) => source.id === sourceInstanceId.value && source.eligible)
	) {
		return
	}

	stageOptions(sourceOptions.value, true, sourceInstanceId.value)
	await nextTick()
	sourceModal.value?.hide()
}

defineExpose({ show, showOnce, hide, skip })
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
		:before-hide="beforeHide"
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
					<Button
						v-if="!allSynced"
						size="lg"
						:disabled="busy"
						:loading="syncMutation.isPending.value"
						@click="hide"
					>
						<CircleSlashIcon />
						{{ formatMessage(messages.skip) }}
					</Button>
					<Button
						v-if="allSynced"
						type="colored"
						color="brand"
						size="lg"
						:disabled="controlsDisabled"
						:loading="syncMutation.isPending.value"
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
							:model-value="draftOptions[option]"
							:disabled="controlsDisabled"
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
		:title="sourceTitle"
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
