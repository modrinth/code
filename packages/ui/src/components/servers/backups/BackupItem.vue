<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { CalendarClockIcon, SaveIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { useFormatDateTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { truncatedTooltip } from '../../../utils'

defineProps<{
	backup: Archon.BackupsQueue.v1.BackupQueueBackup
}>()

const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const nameRef = ref<HTMLElement | null>(null)

const messages = defineMessages({
	auto: {
		id: 'servers.backups.item.auto',
		defaultMessage: 'Auto',
	},
})
</script>

<template>
	<div class="flex min-w-0 items-center gap-4 rounded-[20px] bg-surface-2 p-4">
		<div
			class="flex size-10 shrink-0 items-center justify-center rounded-2xl border border-solid border-surface-5 bg-surface-4"
		>
			<component
				:is="backup.automated ? CalendarClockIcon : SaveIcon"
				class="size-6 text-secondary"
				aria-hidden="true"
			/>
		</div>

		<div class="flex min-w-0 flex-col gap-1.5">
			<div class="flex min-w-0 items-center gap-2">
				<span
					ref="nameRef"
					v-tooltip="truncatedTooltip(nameRef, backup.name)"
					class="min-w-0 truncate font-semibold text-contrast"
				>
					{{ backup.name }}
				</span>
				<span
					v-if="backup.automated"
					class="inline-flex h-7 shrink-0 items-center rounded-full border border-solid border-blue bg-highlight-blue px-2.5 text-sm font-semibold text-blue"
				>
					{{ formatMessage(messages.auto) }}
				</span>
			</div>
			<span class="text-sm font-medium text-secondary">
				{{ formatDateTime(backup.created_at) }}
			</span>
		</div>
	</div>
</template>
