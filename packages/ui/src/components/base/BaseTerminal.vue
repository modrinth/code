<template>
	<div
		class="flex w-full flex-col bg-surface-2 overflow-hidden rounded-[20px] border border-solid border-surface-4"
		:style="
			fullscreen && snappedHeight
				? { maxHeight: snappedHeight + 'px' }
				: !fullscreen && componentHeight
					? { minHeight: componentHeight + 'px' }
					: {}
		"
		:class="{ 'h-full': fullscreen }"
	>
		<div ref="wrapperRef" class="relative min-h-0 flex-1 overflow-hidden pb-2 pt-1">
			<div ref="containerRef" class="size-full" />
			<div v-if="!isAtBottom" class="absolute bottom-4 right-4 z-10">
				<ButtonStyled circular type="highlight" size="large">
					<button class="!shadow-2xl" aria-label="Scroll to bottom" @click="scrollToBottom">
						<ChevronDownIcon />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div
			v-if="showInput"
			ref="inputRef"
			class="border-t border-solid border-b-0 border-x-0 border-surface-4 bg-surface-3 p-4"
		>
			<StyledInput
				v-model="commandInput"
				:icon="TerminalSquareIcon"
				:placeholder="disableInput ? 'Server is not running' : 'Send a command'"
				:disabled="disableInput"
				wrapper-class="w-full"
				input-class="!h-10"
				@keydown.enter="submitCommand"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, TerminalSquareIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { useTerminal } from '#ui/composables/terminal'

const props = withDefaults(
	defineProps<{
		scrollback?: number
		showInput?: boolean
		disableInput?: boolean
		fullscreen?: boolean
	}>(),
	{
		scrollback: Infinity,
		showInput: false,
		disableInput: false,
		fullscreen: false,
	},
)

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
}>()

const containerRef = ref<HTMLElement | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLElement | null>(null)
const commandInput = ref('')
const componentHeight = ref(0)

const snappedHeight = ref<number | null>(null)

const {
	terminal,
	searchAddon,
	isAtBottom,
	write,
	writeln,
	clear,
	reset,
	fit: rawFit,
	scrollToBottom,
} = useTerminal({
	container: containerRef,
	scrollback: props.scrollback,
	onReady: (term) => {
		nextTick(() => {
			updateComponentHeight()
			snapToRows()
		})
		emit('ready', term)
	},
	onResize: () => {
		updateComponentHeight()
	},
})

function getWrapperMargins() {
	if (!wrapperRef.value) return 0
	const style = getComputedStyle(wrapperRef.value)
	return parseFloat(style.marginTop) + parseFloat(style.marginBottom)
}

function snapToRows() {
	if (!props.fullscreen) {
		snappedHeight.value = null
		return
	}
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) {
		snappedHeight.value = null
		return
	}
	const inputH = inputRef.value?.offsetHeight ?? 0
	const borderW = 2
	snappedHeight.value = screen.offsetHeight + getWrapperMargins() + inputH + borderW
}

let resizeDebounce: ReturnType<typeof setTimeout> | null = null

function handleWindowResize() {
	if (!props.fullscreen) return
	if (resizeDebounce) clearTimeout(resizeDebounce)
	snappedHeight.value = null
	resizeDebounce = setTimeout(() => {
		rawFit()
		nextTick(() => snapToRows())
	}, 50)
}

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', handleWindowResize)
	if (resizeDebounce) clearTimeout(resizeDebounce)
})

function fit() {
	rawFit()
	snapToRows()
}

watch(
	() => props.fullscreen,
	() => {
		if (props.fullscreen) {
			nextTick(() => {
				rawFit()
				nextTick(() => snapToRows())
			})
		} else {
			snappedHeight.value = null
			componentHeight.value = 0
		}
	},
)

function updateComponentHeight() {
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) return
	const screenH = screen.offsetHeight
	const inputH = inputRef.value?.offsetHeight ?? 0
	const borderW = 2
	componentHeight.value = screenH + getWrapperMargins() + inputH + borderW
}

const submitCommand = () => {
	const cmd = commandInput.value.trim()
	if (!cmd) return
	emit('command', cmd)
	commandInput.value = ''
}

defineExpose({
	write,
	writeln,
	clear,
	reset,
	fit,
	scrollToBottom,
	terminal,
	searchAddon,
	isAtBottom,
	commandInput,
})
</script>

<style>
.xterm {
	height: 100% !important;
}

.xterm-viewport {
	background-color: var(--surface-2) !important;
}

.xterm .xterm-screen {
	width: 100%;
	margin-left: 8px;
	margin-right: auto;
}

.xterm .xterm-rows {
	position: relative;
	z-index: 7;
}

.xterm .xterm-decoration-container {
	overflow: visible !important;
}

.xterm .xterm-decoration-container > div {
	box-sizing: content-box !important;
	margin-left: -12px !important;
	padding-left: 12px !important;
	padding-right: 12px !important;
}

.xterm-scrollable-element > .scrollbar.vertical {
	width: 8px !important;
}

.xterm-scrollable-element > .scrollbar.vertical > div {
	width: 6px !important;
	border-radius: 8px !important;
	contain: layout style !important;
}
</style>
