<script setup lang="ts">
import { computed } from 'vue'

import { TagItem } from '../../base'

const { region } = defineProps<{
	region: string
}>()

const regionNames: Record<string, string> = {
	us_east: 'US East',
	us_west: 'US West',
	europe: 'Europe',
	asia: 'Asia',
	australia: 'Australia',
	south_america: 'South America',
	middle_east: 'Middle East',
	russia: 'Russia',
}

const regionName = computed(() => regionNames[region] ?? region)
</script>
<template>
	<TagItem v-tooltip="`Server hosted in ${regionName}`">{{ regionName }}</TagItem>
</template>
