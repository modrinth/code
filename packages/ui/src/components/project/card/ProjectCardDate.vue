<script setup lang="ts">
import { CalendarIcon, HistoryIcon } from '@modrinth/assets'
import { capitalizeString } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { useRelativeTime, useVIntl } from '../../../composables'

const { formatMessage } = useVIntl()

const formatRelativeTime = useRelativeTime()

const props = defineProps<{
	date: Date
	type: 'updated' | 'published'
}>()

const formattedDate = computed(() => dayjs(props.date).format('MMMM D, YYYY [at] h:mm A'))

const types = {
	updated: {
		icon: HistoryIcon,
		tooltip: {
			id: 'project-card.date.updated.tooltip',
			defaultMessage: 'Updated {date}',
		},
	},
	published: {
		icon: CalendarIcon,
		tooltip: {
			id: 'project-card.date.published.tooltip',
			defaultMessage: 'Published {date}',
		},
	},
}

const tooltip = computed(() =>
	capitalizeString(formatMessage(types[props.type].tooltip, { date: formattedDate.value })),
)
</script>

<template>
	<div v-tooltip="tooltip" class="flex items-center gap-2 smart-clickable:allow-pointer-events">
		<component :is="types[props.type].icon" class="size-5 shrink-0" />
		{{ capitalizeString(formatRelativeTime(date)) }}
	</div>
</template>
