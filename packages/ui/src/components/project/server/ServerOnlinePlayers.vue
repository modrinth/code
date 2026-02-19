<script setup lang="ts">
import { OnlineIndicatorIcon } from '@modrinth/assets'

import { formatNumber } from '../../../../../utils'
import { useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'
import { StatItem } from '../../base'

const { formatMessage } = useVIntl()

defineProps<{
	online: number
}>()
</script>
<template>
	<StatItem>
		<OnlineIndicatorIcon />
		{{
			formatMessage(commonMessages.projectOnlinePlayerCount, {
				count: formatNumber(online, false),
			})
		}}
	</StatItem>
</template>
