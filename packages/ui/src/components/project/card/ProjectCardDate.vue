<script setup lang="ts">
import { HistoryIcon } from '@modrinth/assets'
import { capitalizeString } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { useRelativeTime, useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'

const { formatMessage } = useVIntl()

const formatRelativeTime = useRelativeTime()

const props = defineProps<{
	dateUpdated: Date
}>()

const formattedUpdateDate = computed(() =>
	dayjs(props.dateUpdated).format('MMMM D, YYYY [at] h:mm A'),
)

const updatedTooltip = computed(() =>
	capitalizeString(
		formatMessage(commonMessages.projectUpdated, { date: formattedUpdateDate.value }),
	),
)
</script>

<template>
	<div
		v-tooltip="updatedTooltip"
		class="flex items-center gap-2 smart-clickable:allow-pointer-events"
	>
		<HistoryIcon class="size-5 shrink-0" />
		{{ capitalizeString(formatRelativeTime(dateUpdated)) }}
	</div>
</template>
