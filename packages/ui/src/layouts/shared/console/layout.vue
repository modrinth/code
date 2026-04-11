<template>
	<div
		class="flex min-h-0 flex-1 flex-col gap-4"
		:class="isFullscreen ? 'fixed inset-0 z-50 bg-surface-1 p-6 pt-9' : ''"
	>
		<div class="flex items-center gap-2">
			<StyledInput
				v-model="searchQuery"
				:icon="SearchIcon"
				placeholder="Search logs"
				wrapper-class="flex-1"
				input-class="!h-10"
				clearable
			/>
			<div v-if="ctx.logSources?.value && ctx.activeLogSourceIndex" class="w-[220px]">
				<Combobox
					:model-value="ctx.activeLogSourceIndex.value"
					:options="logSourceOptions"
					@update:model-value="(v) => (ctx.activeLogSourceIndex!.value = v)"
				/>
			</div>
		</div>

		<div class="flex items-center justify-between">
			<ConsoleFilterPills
				v-model="activeFilters"
				:present-levels="presentLevels"
				@toggle="handleFilterToggle"
			/>
			<ConsoleActionButtons
				:show-clear="isLiveSource"
				:share-disabled="resolvedShareDisabled || !hasLogs"
				:share-disabled-tooltip="!hasLogs ? 'There are no logs to share.' : undefined"
				:sharing="isSharing"
				:fullscreen="isFullscreen"
				@clear="handleClear"
				@share="handleShare"
				@toggle-fullscreen="toggleFullscreen"
			/>
		</div>

		<BaseTerminal
			ref="terminalRef"
			class="min-h-0 flex-1"
			:show-input="resolvedShowInput"
			:disable-input="resolvedDisableInput"
			:fullscreen="isFullscreen"
			@command="handleCommand"
			@ready="handleTerminalReady"
		/>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { computed, isRef, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import BaseTerminal from '#ui/components/base/BaseTerminal.vue'
import Combobox from '#ui/components/base/Combobox.vue'
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
import type { ConditionalLevel } from './composables/console-filtering'
import type { LogLevel, LogLine } from './types'

const ctx = injectConsoleManager()
const modalBehavior = injectModalBehavior()
const pageContext = injectPageContext()

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const searchQuery = ref('')
const isFullscreen = ref(false)
const isSharing = ref(false)
const { activeFilters, toggleFilter, buildFilterPredicate } = useConsoleFilters()
const hasLogs = computed(() => ctx.logLines.value.length > 0)
const presentLevels = computed(() => {
	const levels = new Set<ConditionalLevel>()
	for (const line of ctx.logLines.value) {
		if (line.level === 'debug') levels.add('debug')
		if (line.level === 'trace') levels.add('trace')
		if (levels.size === 2) break
	}
	return levels
})
const isLiveSource = computed(() => {
	const sources = ctx.logSources?.value
	const index = ctx.activeLogSourceIndex?.value
	if (!sources || index === undefined) return true
	return sources[index]?.live ?? true
})
const logSourceOptions = computed(() =>
	(ctx.logSources?.value ?? []).map((s, i) => ({ value: i, label: s.name })),
)

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

const resolvedDisableInput = computed(() => {
	const v = ctx.disableCommandInput
	if (!v) return false
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

function toggleFullscreen() {
	isFullscreen.value = !isFullscreen.value
	if (isFullscreen.value) {
		document.body.style.overflow = 'hidden'
		modalBehavior?.onShow?.()
	} else {
		document.body.style.overflow = ''
		modalBehavior?.onHide?.()
	}
	nextTick(() => {
		terminalRef.value?.fit()
	})
}

let showingEmptyState = false

function writeEmptyState() {
	const term = terminalRef.value?.terminal
	if (!term) return
	term.reset()
	term.write('\x1b[2m Start your instance to start receiving live logs.\x1b[0m')
	showingEmptyState = true
	lastWrittenIndex = 0
}

function writeAllLines() {
	const term = terminalRef.value?.terminal
	if (!term) return
	const lines = ctx.logLines.value
	if (lines.length === 0 && isLiveSource.value) {
		writeEmptyState()
		return
	}
	showingEmptyState = false
	const predicate = buildCombinedPredicate()
	const filtered = predicate ? lines.filter(predicate) : lines
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
	highlightAddon?.reapply(filtered.map((l) => l.level))
	lastWrittenIndex = lines.length
}

watch(ctx.logLines, (lines, oldLines) => {
	const term = terminalRef.value?.terminal
	if (!term) return

	if (lines.length === 0 && isLiveSource.value) {
		writeEmptyState()
		return
	}

	if (showingEmptyState || lines !== oldLines || lines.length < lastWrittenIndex) {
		showingEmptyState = false
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

	isSharing.value = true
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
	} finally {
		isSharing.value = false
	}
}
</script>
