<script setup lang="ts">
import { PlayIcon } from '@modrinth/assets'

import { formatNumber } from '../../../../../utils'
import { useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()

defineProps<{
	recentPlays: number
}>()
</script>
<template>
	<StatItem>
		<PlayIcon />
		{{
			formatMessage(commonMessages.projectRecentPlays, { count: formatNumber(recentPlays, true) })
		}}
	</StatItem>
</template>
