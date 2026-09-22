<template>
	<Button
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		:aria-label="icon ? label : undefined"
		:aria-keyshortcuts="keybind"
		:data-review-keybind="keybind"
		size="sm"
		@click="emit('update:modelValue', !modelValue)"
	>
		<component :is="icon" v-if="icon" />
		<template v-else>{{ label }}</template>
		<kbd
			v-if="keybind && showKeybindHint !== false"
			class="relative bottom-px ml-auto flex h-[18px] min-w-[18px] shrink-0 items-center justify-center rounded border border-solid border-surface-4 bg-surface-3 px-0.5 font-sans text-[10px] font-medium text-primary"
			aria-hidden="true"
		>
			<ArrowBigUpIcon v-if="keybind.startsWith('Shift+')" class="!size-3" />
			{{ keybind.replace('Shift+', '').replaceAll('+', ' ') }}
		</kbd>
	</Button>
</template>

<script lang="ts" setup>
import { ArrowBigUpIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

const props = defineProps<{
	modelValue: boolean
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

const color = computed(() => {
	if (!props.modelValue) return 'standard'
	if (props.needsAttention) return 'orange'
	return props.fixActionable ? 'blue' : 'brand'
})
</script>
