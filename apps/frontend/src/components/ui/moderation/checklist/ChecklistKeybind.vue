<template>
	<div class="flex flex-row items-center gap-2">
		<kbd
			v-for="(definition, index) in definitions"
			:key="`keybind-${index}`"
			ref="keybinding"
			class="cursor-pointer border-2 !text-lg font-bold"
			:class="{
				editing: editing === index,
			}"
			@click="startEditing(index)"
		>
			{{ toDisplay(definition) }}
		</kbd>
	</div>
</template>

<script setup lang="ts">
import { type KeybindDefinition, toKeybindDefinition } from '@modrinth/moderation'
import { onUnmounted } from 'vue'

const props = defineProps<{
	definitions: KeybindDefinition[]
	onChange: (definitions: KeybindDefinition[]) => void
}>()

const keybinding = useTemplateRef('keybinding')
const definitions = ref(structuredClone(props.definitions))
const editing = ref(-1)

function startEditing(index: number) {
	if (editing.value === index) {
		stopEditing()
	} else {
		editing.value = index
		window.addEventListener('keyup', handleKeybinds)
		window.addEventListener('click', handleMouse)
	}
}

function stopEditing() {
	console.log('stop editing')

	editing.value = -1
	window.removeEventListener('keyup', handleKeybinds)
	window.removeEventListener('click', handleMouse)
}

function handleMouse(event: MouseEvent) {
	if (keybinding.value && event.target && editing.value != -1) {
		const editingRef = keybinding.value[editing.value]
		if (editingRef === event.target || editingRef.contains(event.target)) {
			return
		}
	}

	stopEditing()
}

function handleKeybinds(event: KeyboardEvent) {
	definitions.value[editing.value] = toKeybindDefinition(event)
	props.onChange(definitions.value)
	stopEditing()

	event.preventDefault()
	event.stopPropagation()
}

function toDisplay(definition: KeybindDefinition): string {
	const keys = []

	if (definition.ctrl || definition.meta) {
		keys.push(isMac() ? 'CMD' : 'CTRL')
	}
	if (definition.shift) keys.push('SHIFT')
	if (definition.alt) keys.push('ALT')

	const mainKey = definition.key
		.toUpperCase()
		.replace('ARROWLEFT', '←')
		.replace('ARROWRIGHT', '→')
		.replace('ARROWUP', '↑')
		.replace('ARROWDOWN', '↓')
		.replace('ENTER', '↵')
		.replace('ESCAPE', 'ESC')

	keys.push(mainKey)

	return keys.join(' + ')
}

function isMac() {
	return navigator.platform.toUpperCase().includes('MAC')
}

onUnmounted(stopEditing)
</script>

<style scoped lang="scss">
.editing {
	animation: blink 1s step-end infinite;
}

@keyframes blink {
	0%,
	100% {
		border-color: var(--color-red);
		box-shadow: 0 0 10px 1px var(--color-red);
	}

	50% {
		border-color: transparent;
		box-shadow: none;
	}
}
</style>
