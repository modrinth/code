<template>
	<div class="flex min-h-0 flex-1 flex-col gap-4">
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
			<ButtonStyled circular :type="wrapEnabled ? 'highlight' : 'standard'" color="brand">
				<button aria-label="Toggle word wrap" @click="wrapEnabled = !wrapEnabled">
					<WrapTextIcon />
				</button>
			</ButtonStyled>
		</div>

		<div class="flex items-center justify-between">
			<ConsoleFilterPills v-model="activeFilters" @toggle="handleFilterToggle" />
			<ConsoleActionButtons
				:share-disabled="resolvedShareDisabled"
				@clear="handleClear"
				@copy="handleCopy"
				@share="handleShare"
			/>
		</div>

		<BaseTerminal
			ref="terminalRef"
			class="min-h-0 flex-1"
			:show-input="resolvedShowInput"
			@command="handleCommand"
			@ready="handleTerminalReady"
		/>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon, WrapTextIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { computed, isRef, ref, watch } from 'vue'

import BaseTerminal from '#ui/components/base/BaseTerminal.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import DropdownSelect from '#ui/components/base/DropdownSelect.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'

import ConsoleActionButtons from './components/ConsoleActionButtons.vue'
import ConsoleFilterPills from './components/ConsoleFilterPills.vue'
import { useConsoleFilters, rewriteTerminal } from './composables'
import { injectConsoleManager } from './providers'
import type { LogLevel } from './types'

const ctx = injectConsoleManager()

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const searchQuery = ref('')
const wrapEnabled = ref(false)
const { activeFilters, toggleFilter, buildFilterPredicate } = useConsoleFilters()

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
				terminalRef.value?.writeln(lines[i])
			}
		}
		lastWrittenIndex = lines.length
	},
	{ deep: true },
)

watch(searchQuery, (query) => {
	if (searchDebounce) clearTimeout(searchDebounce)
	searchDebounce = setTimeout(() => {
		const addon = terminalRef.value?.searchAddon
		if (!addon) return
		if (query) {
			addon.findNext(query, { decorations: { activeMatchColorOverviewRuler: '#ffffff' } })
		} else {
			addon.clearDecorations()
		}
	}, 200)
})

function handleCommand(cmd: string) {
	ctx.sendCommand?.(cmd)
}

function handleClear() {
	terminalRef.value?.clear()
	lastWrittenIndex = 0
	ctx.onClear?.()
}

async function handleCopy() {
	const predicate = buildFilterPredicate()
	const lines = predicate ? ctx.logLines.value.filter(predicate) : ctx.logLines.value
	await navigator.clipboard.writeText(lines.join('\n'))
}

async function handleShare() {
	const predicate = buildFilterPredicate()
	const lines = predicate ? ctx.logLines.value.filter(predicate) : ctx.logLines.value
	const content = lines.join('\n')

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
