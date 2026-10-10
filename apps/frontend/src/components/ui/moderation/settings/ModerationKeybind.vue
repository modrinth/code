<template>
	<div>
		<span class="flex flex-row items-center gap-2 text-sm text-secondary">
			<BoxIcon v-if="props.scope === 'project'" v-tooltip="formatMessage(messages.projectScope)" />
			<GlobeIcon v-if="props.scope === 'global'" v-tooltip="formatMessage(messages.globalScope)" />
			<ShieldCheckIcon
				v-if="props.scope === 'tech-review'"
				v-tooltip="formatMessage(messages.techScope)"
			/>
			{{ props.title }}
			<IconButton
				type="quiet"
				size="xs"
				class="!size-6"
				:label="formatMessage(messages.reset)"
				:disabled="!hasChanged"
				@click="resetToDefault"
			>
				<RotateCounterClockwiseIcon />
			</IconButton>
		</span>
		<div class="flex flex-row items-center gap-2">
			<kbd
				v-if="definitions.length === 0"
				ref="keybinding"
				class="cursor-pointer border-2 !text-lg font-bold text-secondary"
				:class="{ editing: editing === 0 }"
				@click="startEditing(0)"
			>
				{{ formatMessage(messages.notBound) }}
			</kbd>
			<kbd
				v-for="(definition, index) in definitions"
				v-else
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
import { BoxIcon, GlobeIcon, RotateCounterClockwiseIcon, ShieldCheckIcon } from '@modrinth/assets'
import { formatKeybind, type KeybindDefinition, toKeybindDefinition } from '@modrinth/moderation'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { onMounted, onUnmounted } from 'vue'

const props = defineProps<{
	title: string
	scope: string
	definitions: KeybindDefinition[]
	default: KeybindDefinition[]
	onChange: (definitions: KeybindDefinition[]) => void
}>()

const keybinding = useTemplateRef('keybinding')
const definitions = ref(JSON.parse(JSON.stringify(props.definitions)))
const editing = ref(-1)
const hasChanged = computed(
	() => JSON.stringify(definitions.value) !== JSON.stringify(props.default),
)
const isMac = ref(false)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	projectScope: {
		id: 'moderation.keybinds.scope-project',
		defaultMessage: 'Can be used without the checklist open if setting enabled.',
	},
	globalScope: {
		id: 'moderation.keybinds.scope-global',
		defaultMessage: 'Can be used anywhere on the website.',
	},
	techScope: {
		id: 'moderation.keybinds.scope-tech',
		defaultMessage: 'Used within the tech review pages.',
	},
	reset: {
		id: 'moderation.keybinds.reset-default',
		defaultMessage: 'Reset to default',
	},
	notBound: {
		id: 'moderation.keybinds.not-bound',
		defaultMessage: 'Not bound',
	},
})

function startEditing(index: number) {
	if (editing.value === index) {
		stopEditing()
	} else {
		stopEditing()
		editing.value = index
		window.addEventListener('keydown', handleKeybinds, true)
		window.addEventListener('click', handleMouse)
	}
}

function stopEditing() {
	editing.value = -1
	window.removeEventListener('keydown', handleKeybinds, true)
	window.removeEventListener('click', handleMouse)
}

function resetToDefault() {
	stopEditing()
	definitions.value = JSON.parse(JSON.stringify(props.default))
	props.onChange(definitions.value)
}

function handleMouse(event: MouseEvent) {
	if (keybinding.value && event.target && event.target instanceof Node && editing.value != -1) {
		const editingRef = Array.isArray(keybinding.value)
			? keybinding.value[editing.value]
			: keybinding.value
		if (editingRef === event.target || editingRef.contains(event.target)) {
			return
		}
	}

	stopEditing()
}

function handleKeybinds(event: KeyboardEvent) {
	event.preventDefault()
	event.stopImmediatePropagation()
	if (event.repeat || event.isComposing || ['Control', 'Meta', 'Alt', 'Shift'].includes(event.key))
		return
	if (event.key === 'Escape') {
		definitions.value.splice(editing.value, 1)
	} else if (definitions.value && definitions.value.length > 0) {
		definitions.value[editing.value] = toKeybindDefinition(event)
	} else {
		definitions.value.push(toKeybindDefinition(event))
	}
	props.onChange(definitions.value)
	stopEditing()
}

function toDisplay(definition: KeybindDefinition): string {
	return formatKeybind(definition, isMac.value)
}

onMounted(() => {
	isMac.value = navigator.platform.toUpperCase().includes('MAC')
})
onUnmounted(stopEditing)

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
