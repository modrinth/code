<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ScaleIcon } from '@modrinth/assets'
import { sortByIndex } from '@modrinth/utils'
import { computed } from 'vue'

import { TagItem } from '#ui/components'

import { MODERATION_DB_BADGE } from './external-project-utils'

const props = defineProps<{
	files?: Labrinth.Attribution.Internal.AttributionGroup['files']
}>()

const MODERATION_STATUS_PRIORITY: Labrinth.ExternalProjects.Internal.ExternalLicenseStatus[] = [
	'permanent-no',
	'no',
	'yes',
	'with-attribution',
	'with-attribution-and-source',
	'unidentified',
]

function statusLabel(status: Labrinth.ExternalProjects.Internal.ExternalLicenseStatus): string {
	return MODERATION_DB_BADGE[status]?.label ?? status
}

const moderationStatuses = computed<{
	primary: Labrinth.ExternalProjects.Internal.ExternalLicenseStatus
	others?: Labrinth.ExternalProjects.Internal.ExternalLicenseStatus[]
}>(() => {
	const statuses = (props.files ?? []).map((file) => {
		const status = file.moderation_external_license?.status
		if (!status) {
			return 'unidentified'
		} else {
			return status
		}
	})
	const sorted = sortByIndex(MODERATION_STATUS_PRIORITY, [...new Set(statuses)])
	const primary = sorted[0] ?? 'unidentified'
	return {
		primary,
		...(sorted.length > 1 ? { others: sorted.slice(1) } : {}),
	}
})

const otherStatusesTooltip = computed(() => {
	const others = moderationStatuses.value.others
	if (!others?.length) {
		return undefined
	}
	return others.map((status) => statusLabel(status)).join(', ')
})
</script>

<template>
	<TagItem
		v-tooltip="`Other statuses: ${otherStatusesTooltip}`"
		:style="{ color: MODERATION_DB_BADGE[moderationStatuses.primary]?.color }"
	>
		<ScaleIcon class="size-4 shrink-0" />
		{{ statusLabel(moderationStatuses.primary) }}
		<span class="text-primary">
			{{
				moderationStatuses.others?.length > 0 ? `+ ${moderationStatuses.others?.length} more` : ''
			}}
		</span>
	</TagItem>
</template>
