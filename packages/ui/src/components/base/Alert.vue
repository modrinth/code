<template>
	<div class="flex items-center gap-1.5 text-sm" :class="colorClass">
		<SpinnerIcon v-if="type === 'checking'" class="size-4 flex-none animate-spin" />
		<component :is="icon" v-else class="size-4 flex-none" />
		<slot />
	</div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, InfoIcon, IssuesIcon, SpinnerIcon, XCircleIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = defineProps<{
	type: 'warning' | 'error' | 'success' | 'info' | 'checking'
}>()

const colorClass = computed(() => {
	switch (props.type) {
		case 'warning':
			return 'text-brand-orange'
		case 'error':
			return 'text-brand-red'
		case 'success':
			return 'text-brand-green'
		case 'checking':
			return 'text-secondary'
		default:
			return 'text-brand-blue'
	}
})

const icon = computed(() => {
	switch (props.type) {
		case 'warning':
			return IssuesIcon
		case 'error':
			return XCircleIcon
		case 'success':
			return CheckCircleIcon
		default:
			return InfoIcon
	}
})
</script>
