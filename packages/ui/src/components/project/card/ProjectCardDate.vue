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

const updatedTooltip = computed(() =>
	dayjs(props.dateUpdated).format('MMMM D, YYYY [at] h:mm:ss A'),
)
</script>

<template>
	<div
		v-tooltip="updatedTooltip"
		class="flex items-center gap-2 smart-clickable:allow-pointer-events"
	>
		<HistoryIcon class="size-5 shrink-0" />
		{{
			capitalizeString(
				formatMessage(commonMessages.projectUpdated, { date: formatRelativeTime(dateUpdated) }),
			)
		}}
	</div>
</template>
