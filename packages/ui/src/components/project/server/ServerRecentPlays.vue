<script setup lang="ts">
import { PlayIcon } from '@modrinth/assets'

import { formatNumber } from '../../../../../utils'
import { useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()

defineProps<{
	recentPlays: number
	hideLabel?: boolean
}>()
</script>
<template>
	<StatItem
		v-tooltip="`${formatNumber(recentPlays, true)} recent plays from Modrinth in the past month`"
		class="smart-clickable:allow-pointer-events"
	>
		<PlayIcon />
		{{
			hideLabel
				? formatNumber(recentPlays, true)
				: formatMessage(commonMessages.projectRecentPlays, {
						count: formatNumber(recentPlays, true),
					})
		}}
	</StatItem>
</template>
