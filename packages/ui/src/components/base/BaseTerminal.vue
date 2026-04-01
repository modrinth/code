<template>
	<div
		class="flex w-full flex-col bg-surface-2 overflow-hidden rounded-[20px] border border-solid border-surface-4"
		:style="!fullscreen && componentHeight ? { height: componentHeight + 'px' } : {}"
		:class="{ 'h-full': fullscreen }"
	>
		<div class="relative min-h-0 flex-1 overflow-hidden">
			<div ref="containerRef" class="size-full pl-2" />
			<div v-if="fullscreen" class="absolute top-4 right-4 z-10">
				<ButtonStyled circular type="highlight">
					<button class="!shadow-none" aria-label="Exit fullscreen" @click="emit('exit-fullscreen')">
						<XIcon />
					</button>
				</ButtonStyled>
			</div>
			<div v-if="!isAtBottom" class="absolute bottom-4 right-4">
				<ButtonStyled circular type="highlight">
					<button class="!shadow-none" aria-label="Scroll to bottom" @click="scrollToBottom">
						<ChevronDownIcon />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div
			v-if="showInput"
			ref="inputRef"
			class="border-t border-solid border-b-0 border-x-0 border-surface-5 bg-surface-3 p-4"
		>
			<StyledInput
				v-model="commandInput"
				:icon="TerminalSquareIcon"
				placeholder="Send a command"
				wrapper-class="w-full"
				input-class="!h-10"
				@keydown.enter="submitCommand"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, TerminalSquareIcon, XIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { useTerminal } from '#ui/composables/terminal'

const props = withDefaults(
	defineProps<{
		scrollback?: number
		showInput?: boolean
		fullscreen?: boolean
	}>(),
	{
		scrollback: 10000,
		showInput: false,
		fullscreen: false,
	},
)

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
	'exit-fullscreen': []
}>()

const containerRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLElement | null>(null)
const commandInput = ref('')
const componentHeight = ref(0)

const { terminal, searchAddon, isAtBottom, write, writeln, clear, reset, fit, scrollToBottom } =
	useTerminal({
		container: containerRef,
		scrollback: props.scrollback,
		onReady: (term) => {
			nextTick(() => {
				updateComponentHeight()
			})
			emit('ready', term)
		},
	})

function updateComponentHeight() {
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) return
	const screenH = screen.offsetHeight
	const inputH = inputRef.value?.offsetHeight ?? 0
	const borderW = 2
	componentHeight.value = screenH + inputH + borderW
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


.xterm .xterm-screen {
	margin-left: auto !important;
	margin-right: auto !important;
}
</style>
