<template>
	<div class="grid grid-cols-2 items-center gap-3">
		<span class="flex min-w-0 items-center gap-1.5 text-sm text-secondary [&>svg]:shrink-0">
			<BoxIcon
				v-if="props.scope === 'project'"
				v-tooltip="formatMessage(messages.projectScope)"
				class="size-4"
			/>
			<GlobeIcon
				v-if="props.scope === 'global'"
				v-tooltip="formatMessage(messages.globalScope)"
				class="size-4"
			/>
			<ShieldCheckIcon
				v-if="props.scope === 'tech-review'"
				v-tooltip="formatMessage(messages.techScope)"
				class="size-4"
			/>
			<span class="min-w-0 flex-1 text-sm font-semibold">{{ props.title }}</span>
			<IconButton
				type="quiet"
				size="xs"
				class="!size-6 shrink-0"
				:label="formatMessage(messages.reset)"
				:disabled="!hasChanged"
				@click="resetToDefault"
			>
				<RotateCounterClockwiseIcon />
			</IconButton>
		</span>
		<div class="flex min-w-0 flex-wrap items-center gap-1.5">
			<span
				v-for="(definition, index) in definitions.length ? definitions : [undefined]"
				:key="index"
				ref="keybinding"
				v-tooltip="editing === -1 && conflictMessage ? conflictMessage : undefined"
				class="flex min-w-0 flex-1"
			>
				<Button
					:type="editing === index || conflictMessage ? 'outlined' : 'base'"
					:color="editing === index ? 'brand' : conflictMessage ? 'orange' : undefined"
					size="md"
					:aria-label="
						formatMessage(editing === index ? messages.recording : messages.change, {
							setting: title,
							binding: definition ? toDisplay(definition) : formatMessage(messages.notBound),
						})
					"
					:aria-describedby="editing === index || conflictMessage ? statusId : undefined"
					class="w-full overflow-hidden"
					@click="startEditing(index, $event)"
				>
					<TriangleAlertIcon v-if="editing !== index && conflictMessage" aria-hidden="true" />
					<span class="min-w-0 truncate">{{
						editing === index
							? formatMessage(messages.listen)
							: definition
								? toDisplay(definition)
								: formatMessage(messages.notBound)
					}}</span>
				</Button>
			</span>
		</div>
		<span :id="statusId" class="visually-hidden" role="status" aria-live="polite">{{
			editing !== -1 ? formatMessage(messages.recording, { setting: title }) : conflictMessage
		}}</span>
	</div>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	GlobeIcon,
	RotateCounterClockwiseIcon,
	ShieldCheckIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { formatKeybind, type KeybindDefinition, toKeybindDefinition } from '@modrinth/moderation'
import { Button, defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { computed, onMounted, onUnmounted, ref, useId, useTemplateRef } from 'vue'

const props = defineProps<{
	title: string
	scope: string
	conflictMessage?: string
	definitions: KeybindDefinition[]
	default: KeybindDefinition[]
	onChange: (definitions: KeybindDefinition[]) => void
}>()

const keybinding = useTemplateRef('keybinding')
const definitions = ref<KeybindDefinition[]>(JSON.parse(JSON.stringify(props.definitions)))
const editing = ref(-1)
const statusId = useId()
let recordingWindow: Window | null = null
const hasChanged = computed(
	() => JSON.stringify(definitions.value) !== JSON.stringify(props.default),
)
const isMac = ref(false)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	listen: { id: 'moderation.keybinds.listen', defaultMessage: 'Press a key…' },
	change: {
		id: 'moderation.keybinds.change',
		defaultMessage: '{setting}: {binding}. Activate to change the binding.',
	},
	recording: {
		id: 'moderation.keybinds.recording',
		defaultMessage: '{setting}: listening for a shortcut. Press Escape to clear this binding.',
	},
	projectScope: {
		id: 'moderation.keybinds.scope-project-page',
		defaultMessage: 'Used on project pages when project page shortcuts are enabled.',
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

function startEditing(index: number, event: MouseEvent) {
	if (editing.value === index) {
		stopEditing()
	} else {
		stopEditing()
		editing.value = index
		recordingWindow = (event.currentTarget as HTMLElement).ownerDocument.defaultView
		recordingWindow?.addEventListener('keydown', handleKeybinds, true)
		recordingWindow?.addEventListener('click', handleMouse)
		recordingWindow?.addEventListener('blur', stopEditing)
	}
}

function stopEditing() {
	editing.value = -1
	recordingWindow?.removeEventListener('keydown', handleKeybinds, true)
	recordingWindow?.removeEventListener('click', handleMouse)
	recordingWindow?.removeEventListener('blur', stopEditing)
	recordingWindow = null
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
