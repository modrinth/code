<template>
	<div class="relative h-full w-full">
		<ConfirmModal
			ref="rotationModal"
			:title="formatMessage(messages.rotationTitle)"
			:description="formatMessage(messages.rotationDescription, { count: rotatedBackupCount })"
			:proceed-label="formatMessage(messages.rotationProceed)"
			:markdown="false"
			@proceed="saveOptions(true)"
		/>

		<div v-if="!options" class="flex h-40 items-center justify-center">
			<SpinnerIcon class="size-6 animate-spin text-secondary" aria-hidden="true" />
		</div>

		<div v-else class="flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<label for="auto-backups-toggle" class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.autoTitle) }}
					</label>
					<span>{{ formatMessage(messages.autoDescription) }}</span>
				</div>
				<Toggle
					id="auto-backups-toggle"
					v-model="autoEnabled"
					v-tooltip="manageTooltip"
					:disabled="!canManageBackups"
				/>
			</div>

			<template v-if="autoEnabled">
				<div class="flex flex-col gap-2.5">
					<span id="backup-interval-label" class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.intervalTitle) }}
					</span>
					<Chips
						v-model="interval"
						:items="intervalOptions"
						:format-label="formatInterval"
						:capitalize="false"
						:disabled-items="canManageBackups ? [] : intervalOptions"
						:disabled-tooltip="manageTooltip"
						:aria-label="formatMessage(messages.intervalTitle)"
						hide-checkmark-icon
					/>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.manualSlotsTitle) }}
					</span>
					<Slider
						v-model="manualSlots"
						:min="0"
						:max="options.limit"
						:step="1"
						:disabled="!canManageBackups"
						:aria-label="formatMessage(messages.manualSlotsTitle)"
					/>
					<span>{{
						formatMessage(messages.manualSlotsDescription, { limit: options.limit })
					}}</span>
				</div>
			</template>
		</div>

		<SaveBanner
			:is-visible="hasUnsavedChanges || isPending"
			:server-id="serverId"
			:is-updating="isPending"
			:save="() => saveOptions(false)"
			:reset="syncForm"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { SpinnerIcon } from '@modrinth/assets'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import Chips from '#ui/components/base/Chips.vue'
import Slider from '#ui/components/base/Slider.vue'
import Toggle from '#ui/components/base/Toggle.vue'
import ConfirmModal from '#ui/components/modal/ConfirmModal.vue'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import {
	BACKUP_INTERVAL_PRESETS,
	useBackupIntervalFormat,
} from '#ui/composables/hosting/backup-interval'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerBackupsQueue } from '#ui/composables/server-backups-queue'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const DEFAULT_AUTO_SLOTS = 5

const messages = defineMessages({
	autoTitle: {
		id: 'servers.settings.backups.auto.title',
		defaultMessage: 'Auto backups',
	},
	autoDescription: {
		id: 'servers.settings.backups.auto.description',
		defaultMessage: 'Automatically back up your server at regular intervals.',
	},
	intervalTitle: {
		id: 'servers.settings.backups.interval.title',
		defaultMessage: 'Backup interval',
	},
	manualSlotsTitle: {
		id: 'servers.settings.backups.manual-slots.title',
		defaultMessage: 'Manual backup slots',
	},
	manualSlotsDescription: {
		id: 'servers.settings.backups.manual-slots.description',
		defaultMessage:
			'Choose how many of your {limit} backup slots to use for manual and automatic backups.',
	},
	rotationTitle: {
		id: 'servers.settings.backups.rotation.title',
		defaultMessage: 'Remove auto backups?',
	},
	rotationDescription: {
		id: 'servers.settings.backups.rotation.description',
		defaultMessage:
			'Fewer auto backup slots means {count, plural, one {your oldest auto backup} other {your # oldest auto backups}} will be deleted.',
	},
	rotationProceed: {
		id: 'servers.settings.backups.rotation.proceed',
		defaultMessage: 'Save and delete',
	},
	saveSuccessTitle: {
		id: 'servers.settings.backups.save.success-title',
		defaultMessage: 'Backup settings updated',
	},
	saveErrorTitle: {
		id: 'servers.settings.backups.save.error-title',
		defaultMessage: 'Failed to update backup settings',
	},
})

const { formatMessage } = useVIntl()
const { formatInterval } = useBackupIntervalFormat()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { serverId, worldId } = injectModrinthServerContext()
const { canManageBackups, permissionDeniedMessage } = useServerPermissions()
const { options, invalidate } = useServerBackupsQueue(
	computed(() => serverId),
	worldId,
)

const manageTooltip = computed(() =>
	canManageBackups.value ? undefined : permissionDeniedMessage.value,
)

const autoEnabled = ref(false)
const interval = ref(24)
const manualSlots = ref(0)

const intervalOptions = computed(() => {
	const saved = options.value?.interval
	if (saved === undefined || BACKUP_INTERVAL_PRESETS.includes(saved)) return BACKUP_INTERVAL_PRESETS
	return [...BACKUP_INTERVAL_PRESETS, saved].sort((a, b) => a - b)
})

const pendingPatch = computed<Archon.BackupsQueue.v1.PatchWorldBackupOptions | null>(() => {
	if (!options.value) return null
	const { limit } = options.value
	return autoEnabled.value
		? {
				split_automatic: limit - manualSlots.value,
				split_manual: manualSlots.value,
				interval: interval.value,
			}
		: { split_automatic: 0, split_manual: limit }
})

const hasUnsavedChanges = computed(() => {
	const saved = options.value
	if (!saved) return false
	if (autoEnabled.value !== saved.split_automatic > 0) return true
	return (
		autoEnabled.value &&
		(manualSlots.value !== saved.split_manual || interval.value !== saved.interval)
	)
})

function syncForm() {
	if (!options.value) return
	autoEnabled.value = options.value.split_automatic > 0
	interval.value = options.value.interval
	manualSlots.value = options.value.split_manual
}

watch(
	options,
	(saved, previous) => {
		if (saved && (!previous || !hasUnsavedChanges.value)) syncForm()
	},
	{ immediate: true },
)

watch(autoEnabled, (enabled) => {
	const limit = options.value?.limit ?? 0
	if (enabled && manualSlots.value >= limit) {
		manualSlots.value = Math.max(limit - DEFAULT_AUTO_SLOTS, 0)
	}
})

const rotationModal = ref<InstanceType<typeof ConfirmModal>>()
const rotatedBackupCount = ref(0)

const { mutateAsync: patchOptions, isPending } = useMutation({
	mutationFn: ({ acknowledgeRotation }: { acknowledgeRotation: boolean }) =>
		client.archon.backups_queue_v1.patchOptions(
			serverId,
			worldId.value!,
			pendingPatch.value!,
			acknowledgeRotation,
		),
})

async function saveOptions(acknowledgeRotation: boolean) {
	if (!canManageBackups.value || !pendingPatch.value) return false

	try {
		const result = await patchOptions({ acknowledgeRotation })
		if (!result.applied) {
			rotatedBackupCount.value = result.backup_ids?.length ?? 0
			rotationModal.value?.show()
			return false
		}

		await invalidate()
		syncForm()
		addNotification({ type: 'success', title: formatMessage(messages.saveSuccessTitle) })
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.saveErrorTitle),
			text: error instanceof Error ? error.message : String(error),
		})
		return false
	}
}
</script>
