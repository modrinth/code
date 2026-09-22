<template>
	<kbd
		v-for="(shortcut, index) in shortcuts"
		:key="index"
		:aria-label="shortcut.label"
		class="flex h-[18px] min-w-[18px] shrink-0 items-center justify-center rounded border border-solid border-surface-4 bg-surface-3 px-0.5 font-sans text-[10px] font-medium text-primary"
		v-bind="$attrs"
	>
		<ArrowBigUpIcon v-if="shortcut.shift" class="size-3" aria-hidden="true" />
		{{ shortcut.text }}
	</kbd>
</template>

<script setup lang="ts">
import { ArrowBigUpIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { computed } from 'vue'

import { useModerationKeybinds, useModerationSettings } from '~/composables/moderation'

defineOptions({ inheritAttrs: false })
const props = defineProps<{ keybind: string }>()
const keybinds = useModerationKeybinds()
const settings = useModerationSettings()
const shortcuts = computed(() => {
	if (!settings.value.get(moderationSettings.General.ShowShortcutKeybindHints)) return []
	const binding = [...keybinds.value].find(([id]) => id === props.keybind)?.[1]
	return (
		binding?.keybind.map((definition) => {
			const modifiers = []
			if (definition.ctrl || definition.meta) modifiers.push('Ctrl/Cmd')
			if (definition.alt) modifiers.push('Alt')
			const key = definition.key.toUpperCase()
			return {
				shift: definition.shift,
				label: [...modifiers, ...(definition.shift ? ['Shift'] : []), key].join('+'),
				text: [...modifiers, key].join(' '),
			}
		}) ?? []
	)
})
</script>
