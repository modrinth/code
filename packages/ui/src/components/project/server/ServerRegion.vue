<script setup lang="ts">
import { computed } from 'vue'

import { defineMessage, useVIntl } from '../../../composables'
import { SERVER_REGIONS } from '../../../utils'
import { regionOverrides } from '../../../utils/regions'
import { TagItem } from '../../base'

const { region, flagOnly = false } = defineProps<{
	region: string
	flagOnly?: boolean
}>()

const { formatMessage } = useVIntl()

const tooltip = defineMessage({
	id: 'project.server.region.tooltip',
	defaultMessage: 'Server hosted in {regionName}',
})

const regionName = computed(() => {
	const hostingRegion = regionOverrides[region as keyof typeof regionOverrides]
	if (hostingRegion) return formatMessage(hostingRegion.name)
	const name = SERVER_REGIONS[region]
	if (name) return formatMessage(name)

	return region
})
const regionFlag = computed(() => regionOverrides[region as keyof typeof regionOverrides]?.flag)
</script>
<template>
	<img
		v-if="flagOnly && regionFlag"
		v-tooltip="regionName"
		:src="regionFlag"
		:alt="regionName"
		class="h-4 w-6 rounded-sm object-cover"
	/>
	<TagItem v-else v-tooltip="formatMessage(tooltip, { regionName })">{{ regionName }}</TagItem>
</template>
