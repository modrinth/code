<script setup lang="ts" generic="T">
import { Button } from '#ui/components/base/buttons'

const props = withDefaults(
	defineProps<{
		items: T[]
		ariaLabel: string
		formatLabel?: (item: T) => string
		neutralItems?: T[]
		disabled?: boolean
		fullWidth?: boolean
	}>(),
	{
		formatLabel: (item: T) => String(item),
		neutralItems: () => [],
		disabled: false,
		fullWidth: false,
	},
)

const selected = defineModel<T>()

function isGreenSelection(item: T) {
	return selected.value === item && !props.neutralItems.includes(item)
}
</script>

<template>
	<div
		role="radiogroup"
		:aria-label="ariaLabel"
		class="h-9 items-center gap-0.5 rounded-xl border border-solid border-surface-5 p-px shadow-[0_1px_3px_rgba(0,0,0,0.15)]"
		:class="[fullWidth ? 'flex w-full' : 'inline-flex', { 'opacity-50': disabled }]"
	>
		<Button
			v-for="item in items"
			:key="String(item)"
			type="quiet"
			size="sm"
			role="radio"
			:aria-checked="selected === item"
			:data-tone="neutralItems.includes(item) ? 'neutral' : 'green'"
			:interaction="selected === item ? 'none' : 'surface'"
			class="segment !rounded-[10px] !px-3 !font-medium disabled:!opacity-100"
			:class="[isGreenSelection(item) ? '!text-green' : '!text-contrast', { 'flex-1': fullWidth }]"
			:disabled="disabled"
			@click="selected = item"
		>
			{{ formatLabel(item) }}
		</Button>
	</div>
</template>

<style scoped>
.segment[aria-checked='true'][data-tone='green'] {
	background-color: color-mix(in srgb, var(--color-green) 30%, var(--surface-3));
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-green) 60%, transparent);
}

.segment[aria-checked='true'][data-tone='neutral'] {
	background-color: var(--surface-4);
	box-shadow: inset 0 0 0 1px var(--surface-5);
}
</style>
