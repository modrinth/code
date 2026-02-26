<script setup lang="ts">
import { OnlineIndicatorIcon } from '@modrinth/assets'

import { formatNumber } from '../../../../../utils'
import { useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()

defineProps<{
	online: number
	hideLabel?: boolean
	statusOnline?: boolean
}>()
</script>
<template>
	<StatItem
		v-tooltip="`${formatNumber(online, true)} players online`"
		class="smart-clickable:allow-pointer-events"
	>
		<OnlineIndicatorIcon
			:class="statusOnline && 'pulse-glow'"
			:style="{
				'--_color-inner': statusOnline ? 'var(--color-brand)' : 'var(--color-red)',
				'--_color-outer': statusOnline
					? 'var(--color-green-highlight)'
					: 'var(--color-red-highlight)',
			}"
		/>
		{{
			hideLabel
				? formatNumber(online, false)
				: formatMessage(commonMessages.projectOnlinePlayerCount, {
						count: formatNumber(online, false),
					})
		}}
	</StatItem>
</template>

<style scoped>
.pulse-glow {
	animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
	0%,
	100% {
		filter: brightness(1.15);
	}
	50% {
		filter: brightness(0.95);
	}
}
</style>
