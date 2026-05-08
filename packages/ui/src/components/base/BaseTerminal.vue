<template>
	<div
		class="flex h-full w-full flex-col bg-surface-2 overflow-hidden rounded-[20px] border border-solid border-surface-4"
	>
		<div ref="wrapperRef" class="relative min-h-0 flex-1 overflow-hidden pb-2 pt-1">
			<div ref="containerRef" class="size-full" />
			<Transition name="terminal-loading-fade">
				<div
					v-if="loading"
					class="pointer-events-none absolute inset-0 z-20 animate-bpulse bg-surface-3"
					aria-hidden="true"
				/>
			</Transition>
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
		emptyStateType?: 'server' | 'instance'
		loading?: boolean
	}>(),
	{
		scrollback: Infinity,
		showInput: false,
		disableInput: false,
		fullscreen: false,
		emptyStateType: undefined,
		loading: false,
	},
)

const FROG = [
	'\x1B[32m     _    _ \x1B[37m',
	'\x1B[32m    (o)--(o)      \x1B[37m',
	'\x1B[32m   /.______.\\\x1B[37m',
	'\x1B[32m   \\________/     \x1B[37m',
	'\x1B[32m  ./        \\.    \x1B[37m',
	'\x1B[32m ( .        , )\x1B[37m',
	'\x1B[32m  \\ \\_\\\\ //_/ /\x1B[37m',
	'\x1B[32m   ~~  ~~  ~~\x1B[37m',
]

const EMPTY_STATE_BUBBLES: Record<string, string[]> = {
	server: [
		'   __________________________________________________',
		' /  Welcome to your \x1B[32mModrinth Server\x1B[37m!                  \\',
		'|   Press the green start button to start your server! |',
		' \\____________________________________________________/',
	],
	instance: [
		'   _____________________________________________________________',
		' /  Start your instance in the top right to start               \\',
		'|   receiving live logs!                                        |',
		' \\_____________________________________________________________/',
	],
}

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
}>()

const containerRef = ref<HTMLElement | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLElement | null>(null)
const commandInput = ref('')

const snappedHeight = ref<number | null>(null)

const showingEmptyState = ref(false)

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
			snapToRows()
		})
		emit('ready', term)
	},
})

function writeEmptyState() {
	if (!terminal.value || !props.emptyStateType) return
	terminal.value.reset()
	const bubble = EMPTY_STATE_BUBBLES[props.emptyStateType]
	if (bubble) {
		for (const line of [...bubble, ...FROG]) {
			terminal.value.writeln(line)
		}
	}
	showingEmptyState.value = true
}

function clearEmptyState() {
	if (!showingEmptyState.value) return
	terminal.value?.reset()
	showingEmptyState.value = false
}

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

function handleDocumentPointerDown(event: PointerEvent) {
	if (!terminal.value?.hasSelection()) return
	const target = event.target as Node | null
	if (target && containerRef.value?.contains(target)) return
	terminal.value.clearSelection()
}

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
	document.addEventListener('pointerdown', handleDocumentPointerDown)
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', handleWindowResize)
	document.removeEventListener('pointerdown', handleDocumentPointerDown)
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
		}
	},
)

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
	showingEmptyState,
	writeEmptyState,
	clearEmptyState,
})
</script>

<style>
@keyframes bpulse {
	50% {
		filter: brightness(75%);
	}
}
.animate-bpulse {
	animation: bpulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

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

.terminal-loading-fade-enter-active,
.terminal-loading-fade-leave-active {
	transition: opacity 250ms ease-in-out;
}

.terminal-loading-fade-enter-from,
.terminal-loading-fade-leave-to {
	opacity: 0;
}
</style>
