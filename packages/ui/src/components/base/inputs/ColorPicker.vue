<template>
	<button
		ref="triggerRef"
		v-tooltip="label"
		type="button"
		class="relative flex shrink-0 cursor-pointer items-center justify-center rounded-xl transition-[filter,box-shadow] hover:brightness-125 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:brightness-100"
		:class="triggerSizeClass"
		:disabled="disabled"
		:aria-label="label"
		:aria-expanded="isOpen"
		:aria-controls="panelId"
		aria-haspopup="dialog"
		@click="toggle"
	>
		<span
			class="size-full rounded-xl border border-solid border-surface-5"
			:style="{ backgroundColor: model }"
		/>
	</button>

	<Teleport v-if="isClient" to="body">
		<Transition name="floating-expand">
			<div
				v-if="isOpen"
				:id="panelId"
				ref="panelRef"
				class="fixed isolate z-[9999] w-56 rounded-[14px] bg-surface-3 p-3 shadow-lg ring-1 ring-surface-5"
				:style="panelStyle"
				role="dialog"
				:aria-label="label"
				tabindex="-1"
				@keydown="handlePanelKeydown"
			>
				<div
					ref="svRef"
					class="relative h-36 w-full touch-none select-none rounded-lg outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
					role="group"
					:aria-label="`${label} saturation and brightness`"
					tabindex="0"
					:style="svBackgroundStyle"
					@pointerdown="onSvPointerDown"
					@pointermove="onSvPointerMove"
					@keydown="onSvKeydown"
				>
					<span
						class="pointer-events-none absolute size-[1.125rem] -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgb(0_0_0_/_40%)]"
						:style="svThumbStyle"
					/>
				</div>

				<input
					type="range"
					min="0"
					max="360"
					step="1"
					class="color-picker-hue-slider relative z-10 h-5 w-full"
					:aria-label="`${label} hue`"
					:value="hue"
					@input="onHueInput"
				/>

				<InputFrame appearance="surface" size="standard" class="w-full">
					<template #leading>
						<span class="text-secondary" aria-hidden="true">#</span>
					</template>
					<input
						v-model="hexInput"
						class="w-full min-w-0 appearance-none border-0 bg-transparent p-0 text-base font-medium uppercase shadow-none outline-none focus:ring-0"
						:aria-label="`${label} hex value`"
						maxlength="6"
						spellcheck="false"
						@keydown.enter="commitHexInput"
						@blur="commitHexInput"
					/>
				</InputFrame>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import { colord } from 'colord'
import { computed, nextTick, onMounted, ref, useId, watch } from 'vue'

import {
	type AnchoredTeleportPlacement,
	useAnchoredTeleport,
} from '../../../utils/use-anchored-teleport'
import InputFrame from './InputFrame.vue'

const model = defineModel<string>({ required: true })

const props = withDefaults(
	defineProps<{
		label: string
		disabled?: boolean
		/** Size of the swatch trigger button. */
		size?: 'sm' | 'md'
		placement?: AnchoredTeleportPlacement
	}>(),
	{
		disabled: false,
		size: 'md',
		placement: 'bottom-start',
	},
)

const emit = defineEmits<{
	/** Fired when the picker is opened and an edit session begins. */
	focus: []
	/** Fired when the picker closes and the edit session is committed. */
	change: [value: string]
}>()

const panelId = `color-picker-${useId()}`
const triggerRef = ref<HTMLButtonElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)
const svRef = ref<HTMLElement | null>(null)
const resolvedPlacement = computed(() => props.placement)

const { isOpen, panelStyle, open, close } = useAnchoredTeleport(
	triggerRef,
	panelRef,
	resolvedPlacement,
)

const hue = ref(0)
const saturation = ref(0)
const brightness = ref(100)
const hexInput = ref('')

let suppressModelSync = false

function clamp(value: number, min: number, max: number) {
	return Math.min(max, Math.max(min, value))
}

function hexToHsv(hex: string) {
	if (!/^#[0-9a-f]{6}$/i.test(hex)) return undefined
	const { h, s, v } = colord(hex).toHsv()
	return { h, s, v }
}

function hsvToHex(h: number, s: number, v: number) {
	return colord({ h, s, v }).toHex()
}

function syncFromModel(value: string) {
	const hsv = hexToHsv(value)
	if (!hsv) return
	hue.value = hsv.h
	saturation.value = hsv.s
	brightness.value = hsv.v
	hexInput.value = value.replace('#', '').toUpperCase()
}

watch(
	model,
	(value) => {
		if (suppressModelSync) return
		syncFromModel(value)
	},
	{ immediate: true },
)

function applyHsv() {
	const hex = hsvToHex(hue.value, saturation.value, brightness.value)
	hexInput.value = hex.replace('#', '').toUpperCase()
	suppressModelSync = true
	model.value = hex
	nextTick(() => {
		suppressModelSync = false
	})
}

function updateSvFromPointer(event: PointerEvent) {
	const rect = svRef.value?.getBoundingClientRect()
	if (!rect) return
	const x = clamp(event.clientX - rect.left, 0, rect.width)
	const y = clamp(event.clientY - rect.top, 0, rect.height)
	saturation.value = (x / rect.width) * 100
	brightness.value = 100 - (y / rect.height) * 100
	applyHsv()
}

function onSvPointerDown(event: PointerEvent) {
	;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
	updateSvFromPointer(event)
}

function onSvPointerMove(event: PointerEvent) {
	if (event.buttons !== 1) return
	updateSvFromPointer(event)
}

function onSvKeydown(event: KeyboardEvent) {
	const step = event.shiftKey ? 10 : 2
	switch (event.key) {
		case 'ArrowLeft':
			saturation.value = clamp(saturation.value - step, 0, 100)
			break
		case 'ArrowRight':
			saturation.value = clamp(saturation.value + step, 0, 100)
			break
		case 'ArrowUp':
			brightness.value = clamp(brightness.value + step, 0, 100)
			break
		case 'ArrowDown':
			brightness.value = clamp(brightness.value - step, 0, 100)
			break
		default:
			return
	}
	event.preventDefault()
	applyHsv()
}

function onHueInput(event: Event) {
	hue.value = Number((event.target as HTMLInputElement).value)
	applyHsv()
}

function commitHexInput() {
	const normalized = hexInput.value.trim().replace('#', '')
	if (/^[0-9a-f]{6}$/i.test(normalized)) {
		syncFromModel(`#${normalized.toLowerCase()}`)
		model.value = `#${normalized.toLowerCase()}`
	} else {
		hexInput.value = model.value.replace('#', '').toUpperCase()
	}
}

const svBackgroundStyle = computed(() => ({
	backgroundImage:
		'linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, transparent)',
	backgroundColor: `hsl(${hue.value}, 100%, 50%)`,
}))

const svThumbStyle = computed(() => ({
	left: `${saturation.value}%`,
	top: `${100 - brightness.value}%`,
}))

const triggerSizeClass = computed(() => (props.size === 'sm' ? 'size-8' : 'size-9'))

function handlePanelKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape') return
	event.preventDefault()
	closePicker(true)
}

async function openPicker() {
	if (props.disabled || isOpen.value) return
	await open()
	emit('focus')
}

function closePicker(restoreFocus = false) {
	if (!isOpen.value) return
	close(restoreFocus)
}

function toggle() {
	if (isOpen.value) closePicker()
	else openPicker()
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('change', model.value)
})

const isClient = ref(false)
onMounted(() => {
	isClient.value = true
})

defineExpose({ open: openPicker, close: closePicker })
</script>

<style scoped>
.color-picker-hue-slider {
	display: block;
	margin: 0;
	padding: 0;
	cursor: pointer;
	background: transparent;
	outline: none;
	appearance: none;

	&::-webkit-slider-runnable-track {
		height: 1.125rem;
		border-radius: 999px;
		background: linear-gradient(
			to right,
			#f00 0%,
			#ff0 17%,
			#0f0 33%,
			#0ff 50%,
			#00f 67%,
			#f0f 83%,
			#f00 100%
		);
	}

	&::-webkit-slider-thumb {
		width: 1.125rem;
		height: 1.125rem;
		margin-top: 0;
		border: 2px solid #fff;
		border-radius: 999px;
		background: transparent;
		box-shadow: 0 0 0 1px rgb(0 0 0 / 40%);
		appearance: none;
	}

	&:focus-visible::-webkit-slider-thumb {
		box-shadow:
			0 0 0 1px rgb(0 0 0 / 40%),
			0 0 0 0.25rem var(--color-brand-shadow);
	}

	&::-moz-range-track {
		height: 1.125rem;
		border-radius: 999px;
		background: linear-gradient(
			to right,
			#f00 0%,
			#ff0 17%,
			#0f0 33%,
			#0ff 50%,
			#00f 67%,
			#f0f 83%,
			#f00 100%
		);
	}

	&::-moz-range-thumb {
		width: 1.125rem;
		height: 1.125rem;
		border: 2px solid #fff;
		border-radius: 999px;
		background: transparent;
		box-shadow: 0 0 0 1px rgb(0 0 0 / 40%);
	}
}
</style>
