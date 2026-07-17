<template>
	<div>
		<span class="flex flex-row text-sm text-secondary items-center gap-2">
			<GlobeIcon v-if="props.global" v-tooltip="'Can be used without the checklist open if setting enabled.'"/>
			{{ props.title }}
			<ButtonStyled size="small" circular type="transparent">
				<Button :disabled="!hasChanged" @click="resetToDefault">
					<RotateCounterClockwiseIcon/>
				</Button>
			</ButtonStyled>
		</span>
		<div class="flex flex-row items-center gap-2">
			<kbd
				v-if="definitions.length === 0"
				ref="keybinding"
				class="cursor-pointer border-2 !text-lg font-bold text-secondary"
				:class="{editing: editing === 0}"
				@click="startEditing(0)"
			>
				Not Bound
			</kbd>
			<kbd
				v-else
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
	</div>
</template>

<script setup lang="ts">
import {
	type KeybindDefinition,
	toKeybindDefinition
} from '@modrinth/moderation'
import {onUnmounted} from 'vue'
import {Button, ButtonStyled} from "@modrinth/ui";
import {RotateCounterClockwiseIcon, GlobeIcon} from "@modrinth/assets";

const props = defineProps<{
	title: string,
	global: boolean,
	definitions: KeybindDefinition[],
	default: KeybindDefinition[],
	onChange: (definitions: KeybindDefinition[]) => void
}>()

const keybinding = useTemplateRef('keybinding')
const definitions = ref(JSON.parse(JSON.stringify(props.definitions)))
const editing = ref(-1)
const hasChanged = computed(() => JSON.stringify(definitions.value) !== JSON.stringify(props.default))
const isMac = ref(false)

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
	editing.value = -1
	window.removeEventListener('keyup', handleKeybinds)
	window.removeEventListener('click', handleMouse)
}

function resetToDefault() {
	definitions.value = JSON.parse(JSON.stringify(props.default))
	props.onChange(definitions.value)
}

function handleMouse(event: MouseEvent) {
	if (keybinding.value && event.target && event.target instanceof Node && editing.value != -1) {
		const editingRef = Array.isArray(keybinding.value) ? keybinding.value[editing.value] : keybinding.value
		if (editingRef === event.target || editingRef.contains(event.target)) {
			return
		}
	}

	stopEditing()
}

function handleKeybinds(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		definitions.value.splice(editing.value, 1)
	} else if (definitions.value && definitions.value.length > 0) {
		definitions.value[editing.value] = toKeybindDefinition(event)
	} else {
		definitions.value.push(toKeybindDefinition(event))
	}
	props.onChange(definitions.value)
	stopEditing()

	event.preventDefault()
	event.stopPropagation()
}

function toDisplay(definition: KeybindDefinition): string {
	const keys = []

	if (definition.ctrl || definition.meta) {
		keys.push(isMac.value ? 'CMD' : 'CTRL')
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

	keys.push(mainKey)

	return keys.join(' + ')
}

onUnmounted(() => {
	stopEditing()
	isMac.value = navigator.platform.toUpperCase().includes('MAC')
})

defineExpose({
	setDefinitions(newDefinitions: KeybindDefinition[]) {
		definitions.value = JSON.parse(JSON.stringify(newDefinitions))
	},
})
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
