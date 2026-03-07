<script setup lang="ts">
import { computed } from 'vue'

import { useVIntl } from '../../../composables'
import { SERVER_REGIONS } from '../../../utils'
import { TagItem } from '../../base'

const { region } = defineProps<{
	region: string
}>()

const { formatMessage } = useVIntl()

const regionName = computed(() => {
	const name = SERVER_REGIONS[region]
	if (name) return formatMessage(name)

	return region
})
</script>
<template>
	<TagItem v-tooltip="`Server hosted in ${regionName}`">{{ regionName }}</TagItem>
</template>
