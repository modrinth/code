<template>
	<div
		class="flex min-h-0 flex-col"
		:class="isFullscreen ? 'fixed inset-0 z-50 bg-surface-1 p-6 pt-12' : 'gap-4'"
	>
		<template v-if="!isFullscreen">
			<div class="flex items-center gap-2">
				<StyledInput
					v-model="searchQuery"
					:icon="SearchIcon"
					placeholder="Search logs"
					wrapper-class="flex-1"
					input-class="!h-10"
					clearable
				/>
				<DropdownSelect
					v-if="ctx.logSources?.value && ctx.activeLogSourceIndex"
					:model-value="ctx.logSources.value[ctx.activeLogSourceIndex.value]?.name"
					:options="ctx.logSources.value.map((s) => s.name)"
					name="log-source"
					class="w-[220px]"
					@update:model-value="handleLogSourceChange"
				/>
			</div>

			<div class="flex items-center justify-between">
				<ConsoleFilterPills v-model="activeFilters" @toggle="handleFilterToggle" />
				<ConsoleActionButtons
					:share-disabled="resolvedShareDisabled"
					@clear="handleClear"
					@share="handleShare"
					@expand="enterFullscreen"
				/>
			</div>
		</template>

		<BaseTerminal
			ref="terminalRef"
			class="min-h-0"
			:class="{ 'flex-1 my-auto': isFullscreen }"
			:show-input="resolvedShowInput"
			:fullscreen="isFullscreen"
			@command="handleCommand"
			@ready="handleTerminalReady"
			@exit-fullscreen="exitFullscreen"
		/>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { computed, isRef, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import BaseTerminal from '#ui/components/base/BaseTerminal.vue'
import DropdownSelect from '#ui/components/base/DropdownSelect.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { injectModalBehavior } from '#ui/providers/modal-behavior'

import ConsoleActionButtons from './components/ConsoleActionButtons.vue'
import ConsoleFilterPills from './components/ConsoleFilterPills.vue'
import { colorize, rewriteTerminal, useConsoleFilters } from './composables'
import { injectConsoleManager } from './providers'
import type { LogLevel } from './types'

const ctx = injectConsoleManager()
const modalBehavior = injectModalBehavior()

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const searchQuery = ref('')
const isFullscreen = ref(false)
const { activeFilters, toggleFilter, buildFilterPredicate } = useConsoleFilters()

onBeforeUnmount(() => {
	if (isFullscreen.value) {
		document.body.style.overflow = ''
		modalBehavior?.onHide?.()
	}
})

let lastWrittenIndex = 0
let searchDebounce: ReturnType<typeof setTimeout> | null = null

const resolvedShowInput = computed(() => {
	const v = ctx.showCommandInput
	if (v === undefined) return false
	if (typeof v === 'boolean') return v
	return isRef(v) ? v.value : v
})

const resolvedShareDisabled = computed(() => {
	const v = ctx.shareDisabled
	if (!v) return false
	return isRef(v) ? v.value : v
})

function handleTerminalReady(_terminal: Terminal) {
	writeAllLines()
}

function handleFilterToggle(value: LogLevel | 'all') {
	toggleFilter(value)
	rewriteFiltered()
}

function rewriteFiltered() {
	const term = terminalRef.value?.terminal
	if (!term) return
	const predicate = buildFilterPredicate()
	rewriteTerminal(term, ctx.logLines.value, predicate)
	lastWrittenIndex = ctx.logLines.value.length
}

function enterFullscreen() {
	isFullscreen.value = true
	document.body.style.overflow = 'hidden'
	modalBehavior?.onShow?.()
	nextTick(() => {
		terminalRef.value?.fit()
	})
}

function exitFullscreen() {
	isFullscreen.value = false
	document.body.style.overflow = ''
	modalBehavior?.onHide?.()
	nextTick(() => {
		terminalRef.value?.fit()
	})
}

function writeAllLines() {
	const term = terminalRef.value?.terminal
	if (!term) return
	const predicate = buildFilterPredicate()
	rewriteTerminal(term, ctx.logLines.value, predicate)
	lastWrittenIndex = ctx.logLines.value.length
}

watch(
	() => ctx.logLines.value,
	(lines, oldLines) => {
		const term = terminalRef.value?.terminal
		if (!term) return

		if (lines !== oldLines || lines.length < lastWrittenIndex) {
			rewriteFiltered()
			return
		}

		const predicate = buildFilterPredicate()
		for (let i = lastWrittenIndex; i < lines.length; i++) {
			if (!predicate || predicate(lines[i])) {
				terminalRef.value?.writeln(colorize(lines[i]))
			}
		}
		lastWrittenIndex = lines.length
	},
)

watch(searchQuery, (query) => {
	if (searchDebounce) clearTimeout(searchDebounce)
	searchDebounce = setTimeout(() => {
		const addon = terminalRef.value?.searchAddon
		if (!addon) return
		if (query) {
			addon.findNext(query, {
				decorations: { activeMatchColorOverviewRuler: '#ffffff', matchOverviewRuler: '#888888' },
			})
		} else {
			addon.clearDecorations()
		}
	}, 200)
})

function handleCommand(cmd: string) {
	ctx.sendCommand?.(cmd)
}

function handleClear() {
	terminalRef.value?.reset()
	lastWrittenIndex = 0
	ctx.onClear?.()
}

async function handleShare() {
	const predicate = buildFilterPredicate()
	const lines = predicate ? ctx.logLines.value.filter(predicate) : ctx.logLines.value
	const content = lines.map((l) => l.text).join('\n')

	try {
		const res = await fetch('https://api.mclo.gs/1/log', {
			method: 'POST',
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			body: new URLSearchParams({ content }),
		})
		const data = await res.json()
		if (data.url) {
			window.open(data.url, '_blank')
		}
	} catch (err) {
		console.error('Failed to share logs:', err)
	}
}

function handleLogSourceChange(name: string) {
	if (!ctx.logSources?.value || !ctx.activeLogSourceIndex) return
	const idx = ctx.logSources.value.findIndex((s) => s.name === name)
	if (idx >= 0) {
		ctx.activeLogSourceIndex.value = idx
	}
}
</script>
