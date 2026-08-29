<template>
	<span class="flex min-w-0 items-center gap-1.5 text-contrast">
		<span class="truncate">{{ label }}</span>
		<SpinnerIcon v-if="loading" class="size-4 shrink-0 animate-spin" aria-hidden="true" />
		<span v-else class="shrink-0">({{ formatNumber(count) }})</span>
	</span>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { useFormatNumber } from '@modrinth/ui'

defineProps<{
	label: string
	count: number
	loading?: boolean
}>()

const formatNumber = useFormatNumber()
</script>
