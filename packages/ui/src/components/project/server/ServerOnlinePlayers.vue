<script setup lang="ts">
import { OnlineIndicatorIcon } from '@modrinth/assets'

import { useCompactNumber, useFormatNumber, useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()
const { formatCompactNumber, formatCompactNumberPlural } = useCompactNumber()
const formatNumber = useFormatNumber()

defineProps<{
	online: number
	hideLabel?: boolean
	statusOnline?: boolean
}>()
</script>
<template>
	<StatItem
		v-tooltip="
			formatMessage(commonMessages.projectOnlinePlayerCountTooltip, {
				count: formatCompactNumber(online),
				countPlural: formatCompactNumberPlural(online),
			})
		"
		class="smart-clickable:allow-pointer-events"
	>
		<OnlineIndicatorIcon
			:style="{
				'--_color-inner': statusOnline ? 'var(--color-brand)' : 'var(--color-red)',
				'--_color-outer': statusOnline
					? 'var(--color-green-highlight)'
					: 'var(--color-red-highlight)',
			}"
		/>
		{{
			hideLabel
				? formatNumber(online)
				: formatMessage(commonMessages.projectOnlinePlayerCount, { count: online })
		}}
	</StatItem>
</template>
