<script setup lang="ts">
import { TriangleAlertIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onBeforeUnmount, ref, useId } from 'vue'

import {
	activateKeybindRecording,
	deactivateKeybindRecording,
	formatMinecraftKeybind,
	minecraftKeyTokenFromKeyboardEvent,
	minecraftMouseTokenFromButton,
} from './game-keybinds'

const props = withDefaults(
	defineProps<{
		modelValue: string
		settingLabel: string
		conflicts?: string[]
		mixed?: boolean
		disabled?: boolean
	}>(),
	{
		conflicts: () => [],
		mixed: false,
		disabled: false,
	},
)

const emit = defineEmits<{
	'update:model-value': [value: string]
}>()

const { formatMessage, locale } = useVIntl()
const statusId = useId()
const recording = ref(false)
const statusMessage = ref('')
const controlElement = ref<HTMLElement | null>(null)
let suppressNextClick = false

const messages = defineMessages({
	listen: {
		id: 'app.settings.game-options.keybind.listen',
		defaultMessage: 'Press a key…',
	},
	choose: {
		id: 'app.settings.game-options.keybind.choose',
		defaultMessage: 'Choose a key',
	},
	mixed: {
		id: 'app.settings.game-options.keybind.mixed',
		defaultMessage: 'Different across instances',
	},
	change: {
		id: 'app.settings.game-options.keybind.change',
		defaultMessage: '{setting}: {binding}. Activate to change the binding.',
	},
	listeningStatus: {
		id: 'app.settings.game-options.keybind.listening-status',
		defaultMessage: 'Listening for a key or mouse button. Press Escape to clear this binding.',
	},
	unsupportedStatus: {
		id: 'app.settings.game-options.keybind.unsupported-status',
		defaultMessage: 'That input cannot be used by Minecraft. Press another key.',
	},
	cancelledStatus: {
		id: 'app.settings.game-options.keybind.cancelled-status',
		defaultMessage: 'Key binding change cancelled.',
	},
	assignedStatus: {
		id: 'app.settings.game-options.keybind.assigned-status',
		defaultMessage: '{setting} is now bound to {binding}.',
	},
	conflict: {
		id: 'app.settings.game-options.keybind.conflict',
		defaultMessage: 'This key is also used for {settings}.',
	},
})

const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.platform)
const bindingLabel = computed(() => {
	if (props.mixed) return formatMessage(messages.mixed)
	if (!props.modelValue) return formatMessage(messages.choose)
	return formatMinecraftKeybind(formatMessage, props.modelValue, isMac)
})
const visibleLabel = computed(() =>
	recording.value ? formatMessage(messages.listen) : bindingLabel.value,
)
const conflictSettings = computed(() =>
	new Intl.ListFormat(locale.value, { style: 'long', type: 'conjunction' }).format(props.conflicts),
)
const conflictMessage = computed(() =>
	props.conflicts.length
		? formatMessage(messages.conflict, { settings: conflictSettings.value })
		: '',
)
const accessibleLabel = computed(() => {
	if (recording.value) {
		return `${props.settingLabel}. ${formatMessage(messages.listeningStatus)}`
	}
	const change = formatMessage(messages.change, {
		setting: props.settingLabel,
		binding: bindingLabel.value,
	})
	return conflictMessage.value ? `${change} ${conflictMessage.value}` : change
})

function startRecording() {
	if (props.disabled || recording.value) return
	activateKeybindRecording(cancelRecording)
	recording.value = true
	statusMessage.value = formatMessage(messages.listeningStatus)
	window.addEventListener('keydown', handleKeydown, true)
	window.addEventListener('pointerdown', handlePointerDown, true)
}

function stopRecording() {
	recording.value = false
	window.removeEventListener('keydown', handleKeydown, true)
	window.removeEventListener('pointerdown', handlePointerDown, true)
	deactivateKeybindRecording(cancelRecording)
}

function cancelRecording() {
	if (!recording.value) return
	stopRecording()
	statusMessage.value = formatMessage(messages.cancelledStatus)
}

function assign(value: string) {
	if (!recording.value) return
	stopRecording()
	emit('update:model-value', value)
	statusMessage.value = formatMessage(messages.assignedStatus, {
		setting: props.settingLabel,
		binding: formatMinecraftKeybind(formatMessage, value, isMac),
	})
}

function handleClick(event: MouseEvent) {
	if (suppressNextClick) {
		suppressNextClick = false
		event.preventDefault()
		event.stopPropagation()
		return
	}
	startRecording()
}

function handleKeydown(event: KeyboardEvent) {
	if (!recording.value) return

	event.preventDefault()
	event.stopPropagation()
	if (event.code === 'Escape') {
		assign('key.keyboard.unknown')
		return
	}
	if (event.repeat) return

	const token = event.isComposing ? null : minecraftKeyTokenFromKeyboardEvent(event)
	if (!token) {
		statusMessage.value = formatMessage(messages.unsupportedStatus)
		return
	}
	assign(token)
}

function handlePointerDown(event: PointerEvent) {
	if (!recording.value) return
	if (event.pointerType !== 'mouse') return
	const otherKeybindControl =
		event.target instanceof Element
			? event.target.closest<HTMLElement>('[data-game-keybind-control]')
			: null
	if (otherKeybindControl && otherKeybindControl !== controlElement.value) return

	event.preventDefault()
	event.stopPropagation()
	const token = minecraftMouseTokenFromButton(event.button)
	if (!token) {
		statusMessage.value = formatMessage(messages.unsupportedStatus)
		return
	}
	suppressNextClick = event.button === 0 && otherKeybindControl === controlElement.value
	assign(token)
}

onBeforeUnmount(cancelRecording)
</script>

<template>
	<div class="flex w-full min-w-0 items-center gap-1.5">
		<span
			ref="controlElement"
			v-tooltip="!recording && conflictMessage ? conflictMessage : undefined"
			data-game-keybind-control
			class="flex min-w-0 flex-1"
		>
			<Button
				type="base"
				size="lg"
				:disabled="disabled"
				:aria-label="accessibleLabel"
				:aria-describedby="recording ? statusId : undefined"
				class="keybind-button !w-full overflow-hidden"
				:class="{
					'!bg-brand-highlight !text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]': recording,
					'keybind-button--conflict': !recording && conflicts.length,
					'!bg-transparent !text-secondary': !recording && !conflicts.length,
				}"
				@click="handleClick"
				@contextmenu.prevent.stop
				@auxclick.prevent.stop
			>
				<span class="flex min-w-0 flex-row items-center justify-center gap-2">
					<TriangleAlertIcon
						v-if="!recording && conflicts.length"
						class="keybind-conflict-icon order-first size-4 shrink-0"
						aria-hidden="true"
					/>
					<span class="order-last min-w-0 truncate">{{ visibleLabel }}</span>
				</span>
			</Button>
		</span>

		<span :id="statusId" class="visually-hidden" role="status" aria-live="polite">
			{{ statusMessage }}
		</span>
	</div>
</template>

<style scoped>
.keybind-button.keybind-button--conflict {
	background-color: var(--color-orange-highlight) !important;
	color: var(--color-orange) !important;
	box-shadow: inset 0 0 0 1px var(--color-orange) !important;
}

.keybind-button.keybind-button--conflict .keybind-conflict-icon {
	color: var(--color-orange) !important;
}
</style>
