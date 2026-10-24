<template>
	<kbd
		v-for="(shortcut, index) in shortcuts"
		:key="index"
		:aria-label="shortcut.label"
		class="flex min-w-[18px] shrink-0 items-center justify-center rounded font-sans font-medium text-primary"
		:class="{
			'h-auto border-none bg-transparent !p-0 !text-sm shadow-none': tooltipContent,
			'h-[18px] border border-solid border-surface-4 bg-surface-3 px-0.5 text-[10px]':
				!tooltipContent,
		}"
		v-bind="$attrs"
	>
		{{ shortcut.text }}
	</kbd>
</template>

<script setup lang="ts">
import { formatKeybind, moderationSettings } from '@modrinth/moderation'
import { computed, onMounted, ref } from 'vue'

import { useModerationKeybinds, useModerationSettings } from '~/composables/moderation'

defineOptions({ inheritAttrs: false })
const props = defineProps<{
	keybind: string
	tooltipContent?: boolean
}>()
const keybinds = useModerationKeybinds()
const settings = useModerationSettings()
const isMac = ref(false)
onMounted(() => {
	isMac.value = navigator.platform.toUpperCase().includes('MAC')
})
const shortcuts = computed(() => {
	if (
		!props.tooltipContent &&
		!settings.value.get(moderationSettings.General.ShowShortcutKeybindHints)
	)
		return []
	const binding = [...keybinds.value].find(([id]) => id === props.keybind)?.[1]
	return (
		binding?.keybind.map((definition) => {
			const label = formatKeybind(definition, isMac.value)
			return { label, text: label }
		}) ?? []
	)
})
</script>
