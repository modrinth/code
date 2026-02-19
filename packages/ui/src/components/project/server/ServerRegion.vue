<script setup lang="ts">
import { defineMessage, useVIntl } from '../../../composables'

defineProps<{
	region: string
}>()

const { formatMessage } = useVIntl()

const alt = defineMessage({
	id: 'project.server.region.alt',
	defaultMessage: 'Region: {regionCode}',
})
</script>
<template>
	<img
		:src="`https://flagcdn.com/${region}.svg`"
		:alt="formatMessage(alt, { regionCode: region })"
		class="h-4 aspect-[3/2] border-[1px] border-surface-5 border-solid shrink-0 rounded-[3px] object-cover"
	/>
</template>
