<script setup lang="ts">
import { computed } from 'vue'

import { defineMessage, useVIntl } from '../../../composables'

const { region } = defineProps<{
	region: string
}>()

const { formatMessage } = useVIntl()

const alt = defineMessage({
	id: 'project.server.region.alt',
	defaultMessage: 'Region: {regionCode}',
})

const regionLower = computed(() => region.toLowerCase())
</script>
<template>
	<img
		v-tooltip="`Server hosted in ${region}`"
		:src="`https://flagcdn.com/${regionLower}.svg`"
		:alt="formatMessage(alt, { regionCode: regionLower })"
		class="h-4 aspect-[3/2] border-[1px] border-surface-5 border-solid shrink-0 rounded-[3px] object-cover smart-clickable:allow-pointer-events"
	/>
</template>
