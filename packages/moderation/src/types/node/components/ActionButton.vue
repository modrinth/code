<template>
	<Button
		type="outlined"
		:disabled="disabled"
		:aria-pressed="modelValue"
		:aria-label="icon ? label : undefined"
		:aria-keyshortcuts="keybind"
		:data-review-keybind="keybind"
		:data-tone="tone"
		class="action-toggle !gap-1.5 !rounded-lg !px-2.5 !font-medium [&>svg]:!text-inherit"
		:class="{
			'!brightness-100': modelValue,
			'enabled:hover:!brightness-125 !text-primary': !modelValue,
			'action-toggle-selected !text-contrast': modelValue,
		}"
		size="sm"
		@click="emit('update:modelValue', !modelValue)"
	>
		<span v-if="reReview" class="flex shrink-0 items-center text-orange">
			<TagCategoryRefreshCcwIcon class="size-4" aria-hidden="true" />
		</span>
		<component :is="icon" v-if="icon" aria-hidden="true" />
		<template v-else>{{ label }}</template>
		<kbd
			v-if="keybind && showKeybindHint !== false"
			class="action-toggle-keybind ml-1 flex h-5 min-w-5 shrink-0 items-center justify-center gap-0.5 rounded px-1 font-sans text-[11px] font-medium leading-none"
			aria-hidden="true"
		>
			<ArrowBigUpIcon v-if="keybind.startsWith('Shift+')" class="!size-3" />
			{{ keybind.replace('Shift+', '').replaceAll('+', ' ') }}
		</kbd>
	</Button>
</template>

<script lang="ts" setup>
import { ArrowBigUpIcon, TagCategoryRefreshCcwIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

const props = defineProps<{
	modelValue: boolean
	reReview?: boolean
	label?: string
	icon?: Component
	disabled?: boolean
	needsAttention?: boolean
	fixActionable?: boolean
	keybind?: string
	showKeybindHint?: boolean
}>()

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()

const tone = computed(() => {
	if (props.needsAttention) return 'orange'
	return props.fixActionable ? 'blue' : 'orange'
})
</script>

<style scoped>
.action-toggle {
	--action-color: var(--color-orange);
	--action-highlight: var(--color-orange-highlight);
}

.action-toggle[data-tone='blue'] {
	--action-color: var(--color-blue);
	--action-highlight: var(--color-blue-highlight);
}

.action-toggle-selected {
	background: var(--action-highlight);
	box-shadow: inset 0 0 0 1px var(--action-color);
}

.action-toggle-keybind {
	color: var(--color-primary);
	background: var(--surface-3);
	border: 1px solid var(--surface-5);
	border-bottom-width: 2px;
	box-shadow: 0 1px 0 var(--surface-1);
}
</style>
