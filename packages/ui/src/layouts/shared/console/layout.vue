<template>
	<div
		class="flex min-h-0 flex-1 flex-col"
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
					:share-disabled="resolvedShareDisabled || !hasLogs"
					:share-disabled-tooltip="!hasLogs ? 'There are no logs to share.' : undefined"
					@clear="handleClear"
					@share="handleShare"
					@expand="enterFullscreen"
				/>
			</div>
		</template>

		<BaseTerminal
			ref="terminalRef"
			class="min-h-0 flex-1"
			:class="{ 'my-auto': isFullscreen }"
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
import { injectPageContext } from '#ui/providers/page-context'

import ConsoleActionButtons from './components/ConsoleActionButtons.vue'
import ConsoleFilterPills from './components/ConsoleFilterPills.vue'
import {
	colorize,
	computeHighlightColors,
	LogHighlightAddon,
	rewriteTerminal,
	useConsoleFilters,
} from './composables'
import { injectConsoleManager } from './providers'
import type { LogLevel, LogLine } from './types'

const ctx = injectConsoleManager()
const modalBehavior = injectModalBehavior()
const pageContext = injectPageContext()

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const searchQuery = ref('')
const isFullscreen = ref(false)
const { activeFilters, toggleFilter, buildFilterPredicate } = useConsoleFilters()
const hasLogs = computed(() => ctx.logLines.value.length > 0)

function buildCombinedPredicate(): ((line: LogLine) => boolean) | null {
	const levelPred = buildFilterPredicate()
	const query = searchQuery.value.trim().toLowerCase()
	if (!levelPred && !query) return null
	return (line: LogLine) => {
		if (levelPred && !levelPred(line)) return false
		if (query && !line.text.toLowerCase().includes(query)) return false
		return true
	}
}

onBeforeUnmount(() => {
	if (isFullscreen.value) {
		document.body.style.overflow = ''
		modalBehavior?.onHide?.()
	}
	themeObserver?.disconnect()
	themeObserver = null
})

let lastWrittenIndex = 0
let searchDebounce: ReturnType<typeof setTimeout> | null = null
let highlightAddon: LogHighlightAddon | null = null
let themeObserver: MutationObserver | null = null

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

function handleTerminalReady(terminal: Terminal) {
	const addon = new LogHighlightAddon(computeHighlightColors())
	terminal.loadAddon(addon)
	highlightAddon = addon

	themeObserver = new MutationObserver(() => {
		addon.updateColors(computeHighlightColors())
	})
	themeObserver.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['data-theme', 'class'],
	})

	writeAllLines()
}

function handleFilterToggle(value: LogLevel | 'all') {
	toggleFilter(value)
	rewriteFiltered()
}

function activeSearchQuery(): string {
	return searchQuery.value.trim().toLowerCase()
}

function rewriteFiltered() {
	const term = terminalRef.value?.terminal
	if (!term) return
	const predicate = buildCombinedPredicate()
	const lines = ctx.logLines.value
	const filtered = predicate ? lines.filter(predicate) : lines
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
	highlightAddon?.reapply(filtered.map((l) => l.level))
	lastWrittenIndex = lines.length
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
	const predicate = buildCombinedPredicate()
	const lines = ctx.logLines.value
	const filtered = predicate ? lines.filter(predicate) : lines
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
	highlightAddon?.reapply(filtered.map((l) => l.level))
	lastWrittenIndex = lines.length
}

watch(ctx.logLines, (lines, oldLines) => {
	const term = terminalRef.value?.terminal
	if (!term) return

	if (lines !== oldLines || lines.length < lastWrittenIndex) {
		rewriteFiltered()
		return
	}

	const predicate = buildCombinedPredicate()
	const query = activeSearchQuery()
	for (let i = lastWrittenIndex; i < lines.length; i++) {
		if (!predicate || predicate(lines[i])) {
			terminalRef.value?.writeln(colorize(lines[i], query))
			highlightAddon?.pushLevel(lines[i].level)
		}
	}
	lastWrittenIndex = lines.length
})

watch(searchQuery, () => {
	if (searchDebounce) clearTimeout(searchDebounce)
	searchDebounce = setTimeout(() => {
		rewriteFiltered()
	}, 200)
})

function handleCommand(cmd: string) {
	ctx.sendCommand?.(cmd)
}

function handleClear() {
	terminalRef.value?.reset()
	highlightAddon?.reapply([])
	lastWrittenIndex = 0
	ctx.onClear?.()
}

async function handleShare() {
	const predicate = buildCombinedPredicate()
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
			pageContext.openExternalUrl(data.url)
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
