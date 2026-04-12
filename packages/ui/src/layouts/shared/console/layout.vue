<template>
	<div
		class="flex min-h-0 flex-1 flex-col gap-4"
		:class="isFullscreen ? `fixed inset-0 z-50 bg-surface-1 p-6 py-8 ${isApp ? 'pt-12' : ''}` : ''"
	>
		<CollapsibleAdmonition
			v-if="ctx.crashAnalysis?.value"
			type="critical"
			:header="crashHeader"
			:items="crashItems"
			dismissible
			@dismiss="ctx.onDismissCrash?.()"
		/>

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
				:has-logs="hasLogs"
				:share-disabled="resolvedShareDisabled"
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
			:empty-state-type="ctx.emptyStateType"
			@command="handleCommand"
			@ready="handleTerminalReady"
		/>
	</div>
	<ShareModal ref="shareModal" header="Share Logs" link :social-buttons="false" />
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { computed, isRef, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import BaseTerminal from '#ui/components/base/BaseTerminal.vue'
import type { CollapsibleAdmonitionItem } from '#ui/components/base/CollapsibleAdmonition.vue'
import CollapsibleAdmonition from '#ui/components/base/CollapsibleAdmonition.vue'
import Combobox from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import ShareModal from '#ui/components/modal/ShareModal.vue'
import { injectModrinthClient } from '#ui/providers'
import { injectModalBehavior } from '#ui/providers/modal-behavior'
import { injectNotificationManager } from '#ui/providers/web-notifications.ts'

import ConsoleActionButtons from './components/ConsoleActionButtons.vue'
import ConsoleFilterPills from './components/ConsoleFilterPills.vue'
import { colorize, rewriteTerminal, useConsoleFilters } from './composables'
import type { ConditionalLevel } from './composables/console-filtering'
import { injectConsoleManager } from './providers'
import type { LogLevel, LogLine } from './types'

const ctx = injectConsoleManager()
const client = injectModrinthClient()
const modalBehavior = injectModalBehavior()
const { addNotification } = injectNotificationManager()

const crashHeader = computed(() => {
	const problems = ctx.crashAnalysis?.value?.analysis.problems ?? []
	const count = problems.length
	return `${count} problem${count !== 1 ? 's' : ''} detected`
})

const crashItems = computed<CollapsibleAdmonitionItem[]>(() => {
	const problems = ctx.crashAnalysis?.value?.analysis.problems ?? []
	return problems.map((p) => ({
		title: p.message,
		descriptions: p.solutions.map((s) => s.message),
	}))
})

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const shareModal = ref<InstanceType<typeof ShareModal> | null>(null)
const searchQuery = ref('')
const isFullscreen = ref(false)
const isApp =
	typeof window !== 'undefined' && !!(window as Record<string, unknown>).__TAURI_INTERNALS__
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
})

let lastWrittenIndex = 0
let manuallyCleared = false
let searchDebounce: ReturnType<typeof setTimeout> | null = null

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

function handleTerminalReady(_terminal: Terminal) {
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
	const lines = ctx.logLines.value
	if (lines.length === 0 && isLiveSource.value) {
		writeEmptyState()
		return
	}
	terminalRef.value?.clearEmptyState()
	const predicate = buildCombinedPredicate()
	const filtered = predicate ? lines.filter(predicate) : lines
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
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

function writeEmptyState() {
	terminalRef.value?.writeEmptyState()
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
	terminalRef.value?.clearEmptyState()
	const predicate = buildCombinedPredicate()
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
	lastWrittenIndex = lines.length
}

watch(ctx.logLines, (lines, oldLines) => {
	const term = terminalRef.value?.terminal
	if (!term) return

	if (lines.length === 0 && isLiveSource.value) {
		if (!manuallyCleared) {
			writeEmptyState()
		}
		return
	}

	manuallyCleared = false

	if (
		terminalRef.value?.showingEmptyState ||
		lines !== oldLines ||
		lines.length < lastWrittenIndex
	) {
		terminalRef.value?.clearEmptyState()
		rewriteFiltered()
		return
	}

	const predicate = buildCombinedPredicate()
	const query = activeSearchQuery()
	const newLines: string[] = []
	for (let i = lastWrittenIndex; i < lines.length; i++) {
		if (!predicate || predicate(lines[i])) {
			newLines.push(colorize(lines[i], query))
		}
	}
	if (newLines.length > 0) {
		const buffer = term.buffer.active
		const onFreshLine = buffer.cursorX === 0
		const data = onFreshLine ? newLines.join('\r\n') : '\r\n' + newLines.join('\r\n')
		term.write(data)
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
	manuallyCleared = true
	terminalRef.value?.reset()
	lastWrittenIndex = 0
	ctx.onClear?.()
}

async function handleShare() {
	const predicate = buildCombinedPredicate()
	const lines = predicate ? ctx.logLines.value.filter(predicate) : ctx.logLines.value
	const content = lines.map((l) => l.text).join('\n')

	isSharing.value = true
	try {
		const data = await client.mclogs.logs_v1.create(content)
		if (data.url) {
			shareModal.value?.show(data.url)
		}
	} catch (err) {
		console.error('Failed to share logs:', err)
		addNotification({
			type: 'error',
			title: 'Failed to share logs',
			text: typeof err === 'string' ? err : 'Unknown error.',
		})
	} finally {
		isSharing.value = false
	}
}
</script>
