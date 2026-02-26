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
}>()
</script>
<template>
	<StatItem
		v-tooltip="`${formatNumber(online, true)} players online`"
		class="smart-clickable:allow-pointer-events"
	>
		<OnlineIndicatorIcon />
		{{
			hideLabel
				? formatNumber(online, false)
				: formatMessage(commonMessages.projectOnlinePlayerCount, {
						count: formatNumber(online, false),
					})
		}}
	</StatItem>
</template>
