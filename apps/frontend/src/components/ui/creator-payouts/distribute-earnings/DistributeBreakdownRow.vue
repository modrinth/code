<template>
	<div class="mb-4 flex items-center justify-between gap-6 last:mb-0">
		<span
			class="font-base text-base"
			:class="strong ? 'font-medium text-contrast' : 'text-primary'"
		>
			{{ label }}
		</span>
		<span
			class="font-base text-right text-base"
			:class="[
				value === emptyValue
					? 'text-primary opacity-60'
					: strong
						? 'text-lg font-medium text-contrast'
						: 'text-primary',
				negative ? 'text-red' : '',
			]"
		>
			{{ value }}
		</span>
	</div>
</template>

<script setup lang="ts">
defineProps<{
	label: string
	value: string
	negative?: boolean
	strong?: boolean
}>()

const emptyValue = '—'
</script>
