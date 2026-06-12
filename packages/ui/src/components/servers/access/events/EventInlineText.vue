<template>
	<span
		ref="textRef"
		v-tooltip="truncatedTooltip(textRef, tooltipText)"
		class="max-w-full min-w-0 whitespace-normal break-words @[800px]:truncate @[800px]:whitespace-nowrap"
	>
		<slot>{{ text }}</slot>
	</span>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { truncatedTooltip } from '#ui/utils/truncate'

const props = defineProps<{
	text: string | number
	tooltip?: string
}>()

const textRef = ref<HTMLElement | null>(null)
const tooltipText = computed(() => props.tooltip ?? String(props.text))
</script>
