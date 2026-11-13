<template>
	<TagItem v-tooltip="consumerTooltip" tabindex="0">
		<CircleAlertIcon v-if="numConsumers === 0" class="text-orange" aria-hidden="true" />
		<LoaderCircleIcon v-else class="animate-spin" aria-hidden="true" />
		{{ scanned?.toLocaleString('en-US') ?? '—' }}/{{ total.toLocaleString('en-US') }} scanned
	</TagItem>
</template>

<script setup lang="ts">
import { CircleAlertIcon, LoaderCircleIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'

const props = defineProps<{ scanned?: number; total: number; numConsumers?: number }>()

const consumerTooltip = computed(() => {
	const count = props.numConsumers
	if (count === undefined) return 'Consumer count unavailable'
	return `${count.toLocaleString('en-US')} ${count === 1 ? 'consumer' : 'consumers'}`
})
</script>
