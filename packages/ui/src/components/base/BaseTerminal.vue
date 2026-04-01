<template>
	<div
		class="flex size-full flex-col bg-surface-2 overflow-hidden rounded-[4px] border border-solid border-surface-4"
	>
		<div class="relative min-h-0 pb-1 flex-1 overflow-hidden">
			<div ref="containerRef" class="size-full pl-2" />
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
import { ChevronDownIcon, TerminalSquareIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { useTerminal } from '#ui/composables/terminal'

const props = withDefaults(
	defineProps<{
		scrollback?: number
		showInput?: boolean
	}>(),
	{
		scrollback: 10000,
		showInput: false,
	},
)

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
}>()

const containerRef = ref<HTMLElement | null>(null)
const commandInput = ref('')

const { terminal, searchAddon, isAtBottom, write, writeln, clear, reset, fit, scrollToBottom } =
	useTerminal({
		container: containerRef,
		scrollback: props.scrollback,
		onReady: (term) => emit('ready', term),
	})

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

.xterm .xterm-scrollable-element {
	height: 100% !important;
}

.xterm .xterm-screen {
	min-height: 100% !important;
	margin-left: auto !important;
	margin-right: auto !important;
}
</style>
