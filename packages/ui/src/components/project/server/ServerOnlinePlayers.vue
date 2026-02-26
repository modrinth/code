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
