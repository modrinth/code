<template>
	<div class="relative h-full w-full">
		<BackupRotationModal ref="rotationModal" @proceed="saveOptions(true)" />

		<div v-if="!options" class="flex h-40 items-center justify-center">
			<SpinnerIcon class="size-6 animate-spin text-secondary" aria-hidden="true" />
		</div>

		<div v-else class="flex flex-col gap-6">
			<div
				class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
			>
				<div>
					<h3 id="auto-backups-label" class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.autoTitle) }}
					</h3>
					<p id="auto-backups-description" class="m-0 mt-1">
						{{ formatMessage(messages.autoDescription) }}
					</p>
				</div>
				<span v-tooltip="manageTooltip" class="inline-flex shrink-0">
					<Toggle
						id="auto-backups"
						v-model="autoEnabled"
						:disabled="!canManageBackups"
						aria-labelledby="auto-backups-label"
						aria-describedby="auto-backups-description"
					/>
				</span>
			</div>

			<div class="flex flex-col gap-6" :class="{ 'opacity-50': !autoEnabled }">
				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.intervalTitle) }}
					</h3>
					<SegmentedControl
						v-model="interval"
						v-tooltip="manageTooltip"
						:items="intervalOptions"
						:format-label="formatInterval"
						:aria-label="formatMessage(messages.intervalTitle)"
						:disabled="!canManageBackups || !autoEnabled"
						equal-width
					/>
				</div>

				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.manualSlotsTitle) }}
					</h3>
					<Slider
						v-model="manualSlots"
						:min="0"
						:max="slotLimit"
						:step="1"
						:snap-points="manualSlotSnapPoints"
						:snap-range="1"
						:disabled="!canManageBackups || !autoEnabled"
						:aria-label="formatMessage(messages.manualSlotsTitle)"
					/>
					<p class="m-0">
						{{ formatMessage(messages.manualSlotsDescription, { limit: slotLimit }) }}
					</p>
				</div>
			</div>
		</div>

		<SaveBanner
			:is-visible="hasUnsavedChanges || isSaving"
			:server-id="serverId"
			:is-updating="isSaving"
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

import SegmentedControl from '#ui/components/base/SegmentedControl.vue'
import Slider from '#ui/components/base/Slider.vue'
import Toggle from '#ui/components/base/Toggle.vue'
import BackupRotationModal from '#ui/components/servers/backups/BackupRotationModal.vue'
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
const MAX_BACKUP_SLOTS = 15

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
const { backups, options, invalidate } = useServerBackupsQueue(
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

const slotLimit = computed(() => Math.min(options.value?.limit ?? 0, MAX_BACKUP_SLOTS))

const manualSlotSnapPoints = computed(() =>
	Array.from({ length: Math.floor(slotLimit.value / 5) }, (_, index) => (index + 1) * 5),
)

const pendingPatch = computed<Archon.BackupsQueue.v1.PatchWorldBackupOptions | null>(() => {
	if (!options.value) return null
	const limit = slotLimit.value
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
	manualSlots.value = Math.min(options.value.split_manual, slotLimit.value)
}

watch(
	options,
	(saved, previous) => {
		if (saved && (!previous || !hasUnsavedChanges.value)) syncForm()
	},
	{ immediate: true },
)

watch(autoEnabled, (enabled) => {
	const limit = slotLimit.value
	if (enabled && manualSlots.value >= limit) {
		manualSlots.value = Math.max(limit - DEFAULT_AUTO_SLOTS, 0)
	}
})

const rotationModal = ref<InstanceType<typeof BackupRotationModal>>()

const isSaving = ref(false)

const { mutateAsync: patchOptions } = useMutation({
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

	isSaving.value = true
	try {
		const result = await patchOptions({ acknowledgeRotation })
		if (!result.applied) {
			const rotatedIds = result.backup_ids ?? []
			rotationModal.value?.show(
				backups.value.filter((backup) => rotatedIds.includes(backup.id)),
				rotatedIds.length,
			)
			return false
		}

		await invalidate()
		syncForm()
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.saveErrorTitle),
			text: error instanceof Error ? error.message : String(error),
		})
		return false
	} finally {
		isSaving.value = false
	}
}
</script>
