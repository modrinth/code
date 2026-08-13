<template>
	<div class="flex flex-col gap-4 h-full">
		<ConsolePageLayout />
	</div>
</template>

<script setup>
import {
	ConsolePageLayout,
	injectModrinthClient,
	injectNotificationManager,
	provideConsoleManager,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, shallowRef, triggerRef, watch, watchEffect } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { delete_logs_by_filename, get_output_by_filename } from '@/helpers/logs.js'

import { injectInstancePage } from '../instance-context'
import { instanceKeys } from '../query-options'

const client = injectModrinthClient()
const { handleError } = injectNotificationManager()
const instancePage = injectInstancePage()
const instanceId = instancePage.instanceId
const {
	liveConsole,
	historicalConsole,
	hydrate,
	getHistoricalLogs,
	getHistoricalContent,
	invalidate,
	clearLive,
} = useInstanceConsole(instanceId.value)

const consoleHydrationQuery = useQuery({
	queryKey: computed(() => instanceKeys.console(instanceId.value)),
	queryFn: async () => {
		await hydrate()
		return true
	},
	staleTime: 0,
	refetchOnMount: 'always',
})

await consoleHydrationQuery.suspense()

function buildLogList(rawLogs) {
	return [
		{ name: 'Live Log', live: true },
		...rawLogs
			.filter(
				(log) =>
					log.filename !== 'latest_stdout.log' &&
					log.filename !== 'latest_stdout' &&
					log.filename !== 'launcher_log.txt' &&
					(log.output == null || log.output !== '') &&
					(log.filename.includes('.log') || log.filename.endsWith('.txt')),
			)
			.map((log) => ({
				...log,
				name: log.filename || 'Unknown',
			})),
	]
}

const logs = ref(buildLogList([]))
const historicalLogsQuery = useQuery({
	queryKey: computed(() => instanceKeys.logs(instanceId.value)),
	queryFn: getHistoricalLogs,
	staleTime: 0,
})
watch(
	historicalLogsQuery.data,
	(allLogs) => {
		if (allLogs) logs.value = buildLogList(allLogs)
	},
	{ immediate: true },
)
watch(historicalLogsQuery.error, (error) => {
	if (error) handleError(error)
})

const selectedLogIndex = ref(0)
const isLive = computed(() => selectedLogIndex.value === 0)

const filteredLogs = computed(() =>
	instancePage.playing.value
		? logs.value.filter((l) => l.live || l.name !== 'latest.log')
		: logs.value,
)

const logSources = computed(() =>
	filteredLogs.value.map((l, i) => ({
		id: String(i),
		name: l?.name ?? `Log ${i}`,
		live: l?.live ?? false,
	})),
)

const activeConsole = computed(() => (isLive.value ? liveConsole : historicalConsole))

const logLines = shallowRef(activeConsole.value.output.value)
watchEffect(() => {
	logLines.value = activeConsole.value.output.value
	triggerRef(logLines)
})

const crashAnalysis = ref(null)

async function analyseForCrash() {
	const lines = liveConsole.output.value
	if (lines.length === 0) return

	const content = lines.map((l) => l.text).join('\n')
	try {
		const data = await client.mclogs.insights_v1.analyse(content)
		if (data.analysis?.problems?.length > 0) {
			crashAnalysis.value = data
		}
	} catch {
		// Crash analysis is best-effort
	}
}

const selectedLog = computed(() => filteredLogs.value[selectedLogIndex.value])

const deleteDisabled = computed(() => {
	const log = selectedLog.value
	if (!log || log.live) return true
	return log.filename === 'latest.log' && instancePage.playing.value
})

async function deleteSelectedLog() {
	const log = selectedLog.value
	if (!log || log.live) return
	await delete_logs_by_filename(instanceId.value, log.log_type, log.filename)
	invalidate()
	const { data } = await historicalLogsQuery.refetch()
	if (data) logs.value = buildLogList(data)
	selectedLogIndex.value = 0
}

provideConsoleManager({
	logLines,
	logSources,
	activeLogSourceIndex: selectedLogIndex,
	showCommandInput: false,
	loading: ref(false),
	onClear: () => {
		if (!isLive.value) return
		void clearLive()
	},
	onDelete: deleteSelectedLog,
	deleteDisabled,
	deleteDisabledTooltip: 'Cannot delete latest.log while the instance is running',
	shareDisabled: instancePage.offline,
	emptyStateType: 'instance',
	crashAnalysis,
	onDismissCrash: () => {
		crashAnalysis.value = null
	},
})

watch(selectedLogIndex, async (newIndex) => {
	if (newIndex === 0) return
	const log = filteredLogs.value[newIndex]
	if (!log) return

	const cached = getHistoricalContent(log.filename)
	if (cached) {
		historicalConsole.clear()
		historicalConsole.addLegacyLog(cached)
		return
	}

	const output = await get_output_by_filename(instanceId.value, log.log_type, log.filename).catch(
		handleError,
	)
	if (output) {
		historicalConsole.clear()
		historicalConsole.addLegacyLog(output)
	}
})

selectedLogIndex.value = 0

if (!instancePage.playing.value) {
	void analyseForCrash()
}

useAppEvent('log', (payload) => {
	if (payload.instance_id !== instanceId.value) return

	if (payload.type === 'log4j') {
		liveConsole.addLog4jEvent(payload)
	} else if (payload.type === 'legacy') {
		liveConsole.addLegacyLog(payload.message)
	}
})

useAppEvent('process', async (e) => {
	if (e.instance_id !== instanceId.value) return
	if (e.event === 'launched') {
		liveConsole.clear()
		invalidate()
		void historicalLogsQuery.refetch()
		selectedLogIndex.value = 0
	}
	if (e.event === 'finished') {
		invalidate()
		const { data } = await historicalLogsQuery.refetch()
		if (data) logs.value = buildLogList(data)
		void analyseForCrash()
	}
})
</script>
