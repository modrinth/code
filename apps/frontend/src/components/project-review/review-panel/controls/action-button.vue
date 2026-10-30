<template>
	<Button
		type="outlined"
		:disabled="disabled"
		:aria-pressed="mixed ? 'mixed' : modelValue"
		:aria-keyshortcuts="ariaKeybind"
		:data-review-toggle-index="toggleIndex"
		:style="
			color
				? {
						'--action-color': color,
						'--action-highlight': `color-mix(in srgb, ${color} 25%, transparent)`,
					}
				: undefined
		"
		class="action-toggle !h-auto min-h-8 max-w-full !gap-1.5 !whitespace-normal !rounded-lg !px-2.5 !font-medium [&>svg]:!text-inherit"
		:class="{
			'!brightness-100': modelValue,
			'!text-primary enabled:hover:!brightness-125': !modelValue,
			'action-toggle-selected !text-contrast': modelValue || mixed,
		}"
		size="sm"
		@click="emit('update:modelValue', !modelValue)"
	>
		<MinusIcon v-if="mixed" class="size-4" aria-hidden="true" />
		<span v-if="reReview" class="flex shrink-0 items-center text-orange">
			<TagCategoryRefreshCcwIcon class="size-4" aria-hidden="true" />
		</span>
		<span class="min-w-0 [overflow-wrap:anywhere]">{{ label }}</span>
		<kbd
			v-if="keybind && showKeybindHint !== false"
			class="action-toggle-keybind ml-1 flex h-5 min-w-5 shrink-0 items-center justify-center gap-0.5 rounded px-1 font-sans text-[11px] font-medium leading-none"
			aria-hidden="true"
		>
			{{ keybindLabel }}
		</kbd>
	</Button>
</template>

<script lang="ts" setup>
import { MinusIcon, TagCategoryRefreshCcwIcon } from '@modrinth/assets'
import { formatKeybind, type KeybindDefinition } from '@modrinth/moderation'
import { Button } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

const props = defineProps<{
	modelValue: boolean
	mixed?: boolean
	reReview?: boolean
	label: string
	color?: string
	disabled?: boolean
	keybind?: KeybindDefinition
	toggleIndex?: number
	showKeybindHint?: boolean
}>()

const isMac = ref(false)
onMounted(() => {
	isMac.value = navigator.platform.toUpperCase().includes('MAC')
})
const keybindLabel = computed(() =>
	props.keybind ? formatKeybind(props.keybind, isMac.value) : undefined,
)
const ariaKeybind = computed(() => {
	const definition = props.keybind
	if (!definition) return undefined
	const portableMod =
		definition.mod || (definition.ctrl && (definition.meta || definition.meta === undefined))
	return [
		(portableMod ? !isMac.value : definition.ctrl) ? 'Control' : undefined,
		(portableMod ? isMac.value : definition.meta) ? 'Meta' : undefined,
		definition.shift ? 'Shift' : undefined,
		definition.alt ? 'Alt' : undefined,
		definition.key === ' ' ? 'Space' : definition.key,
	]
		.filter(Boolean)
		.join('+')
})

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()
</script>

<style scoped>
.action-toggle {
	--action-color: var(--color-orange);
	--action-highlight: var(--color-orange-highlight);
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
