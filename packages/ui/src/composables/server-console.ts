import { createGlobalState } from '@vueuse/core'
import { type Ref, shallowRef } from 'vue'

const maxLines = 10000
const batchTimeout = 300
const initialBatchSize = 256

export const useModrinthServersConsole = createGlobalState(() => {
	const output: Ref<string[]> = shallowRef<string[]>([])
	const searchQuery: Ref<string> = shallowRef('')
	const filteredOutput: Ref<string[]> = shallowRef([])
	let searchRegex: RegExp | null = null

	let lineBuffer: string[] = []
	let batchTimer: NodeJS.Timeout | null = null
	let isProcessingInitialBatch = false

	let refilterTimer: NodeJS.Timeout | null = null
	const refilterTimeout = 100

	const updateFilter = () => {
		if (!searchQuery.value) {
			filteredOutput.value = []
			return
		}

		if (!searchRegex) {
			searchRegex = new RegExp(searchQuery.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i')
		}

		filteredOutput.value = output.value.filter((line) => searchRegex?.test(line) ?? false)
	}

	const scheduleRefilter = () => {
		if (refilterTimer) clearTimeout(refilterTimer)
		refilterTimer = setTimeout(updateFilter, refilterTimeout)
	}

	const flushBuffer = () => {
		if (lineBuffer.length === 0) return

		const processedLines = lineBuffer.flatMap((line) => line.split('\n').filter(Boolean))

		if (isProcessingInitialBatch && processedLines.length >= initialBatchSize) {
			isProcessingInitialBatch = false
			output.value = processedLines.slice(-maxLines)
		} else {
			const newOutput = [...output.value, ...processedLines]
			output.value = newOutput.slice(-maxLines)
		}

		lineBuffer = []
		batchTimer = null

		if (searchQuery.value) {
			scheduleRefilter()
		}
	}

	const addLine = (line: string): void => {
		lineBuffer.push(line)

		if (!batchTimer) {
			batchTimer = setTimeout(flushBuffer, batchTimeout)
		}
	}

	const addLines = (lines: string[]): void => {
		if (output.value.length === 0 && lines.length >= initialBatchSize) {
			isProcessingInitialBatch = true
			lineBuffer = lines
			flushBuffer()
			return
		}

		lineBuffer.push(...lines)

		if (!batchTimer) {
			batchTimer = setTimeout(flushBuffer, batchTimeout)
		}
	}

	const setSearchQuery = (query: string): void => {
		searchQuery.value = query
		searchRegex = null
		updateFilter()
	}

	const clear = (): void => {
		output.value = []
		filteredOutput.value = []
		searchQuery.value = ''
		lineBuffer = []
		isProcessingInitialBatch = false
		if (batchTimer) {
			clearTimeout(batchTimer)
			batchTimer = null
		}
		if (refilterTimer) {
			clearTimeout(refilterTimer)
			refilterTimer = null
		}
		searchRegex = null
	}

	const findLineIndex = (line: string): number => {
		return output.value.findIndex((l) => l === line)
	}

	return {
		output,
		searchQuery,
		filteredOutput,
		addLine,
		addLines,
		setSearchQuery,
		clear,
		findLineIndex,
	}
})
