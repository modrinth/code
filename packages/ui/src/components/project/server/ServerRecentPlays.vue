<script setup lang="ts">
import { PlayIcon } from '@modrinth/assets'

import { useCompactNumber, useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()
const { formatCompactNumber, formatCompactNumberPlural } = useCompactNumber()

defineProps<{
	recentPlays: number
	hideLabel?: boolean
}>()
</script>
<template>
	<StatItem
		v-tooltip="
			formatMessage(commonMessages.projectRecentPlaysTooltip, {
				count: formatCompactNumber(recentPlays),
				countPlural: formatCompactNumberPlural(recentPlays),
			})
		"
		class="smart-clickable:allow-pointer-events"
	>
		<PlayIcon />
		{{
			hideLabel
				? formatCompactNumber(recentPlays)
				: formatMessage(commonMessages.projectRecentPlays, {
						count: formatCompactNumber(recentPlays),
						countPlural: formatCompactNumberPlural(recentPlays),
					})
		}}
	</StatItem>
</template>
