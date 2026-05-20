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
import { computed, onUnmounted, ref, shallowRef, triggerRef, watch, watchEffect } from 'vue'
import { useRoute } from 'vue-router'

import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { log_listener, process_listener } from '@/helpers/events.js'
import { delete_logs_by_filename, get_output_by_filename } from '@/helpers/logs.js'

const client = injectModrinthClient()
const { handleError } = injectNotificationManager()
const route = useRoute()

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
	},
	options: {
		type: Object,
		default() {
			return {}
		},
	},
	offline: {
		type: Boolean,
		default() {
			return false
		},
	},
	playing: {
		type: Boolean,
		default() {
			return false
		},
	},
	installed: {
		type: Boolean,
		default() {
			return false
		},
	},
})

const profilePathId = computed(() => route.params.id)
const {
	liveConsole,
	historicalConsole,
	hydrate,
	getHistoricalLogs,
	getHistoricalContent,
	invalidate,
	clearLive,
} = useInstanceConsole(profilePathId.value)

await hydrate()

function buildLogList(rawLogs) {
	return [
		{ name: 'Live Log', live: true },
		...rawLogs
			.filter(
				(log) =>
					log.filename !== 'latest_stdout.log' &&
					log.filename !== 'latest_stdout' &&
					log.filename !== 'launcher_log.txt' &&
					log.stdout !== '' &&
					(log.filename.includes('.log') || log.filename.endsWith('.txt')),
			)
			.map((log) => ({
				...log,
				name: log.filename || 'Unknown',
			})),
	]
}

const logs = ref(buildLogList([]))

void getHistoricalLogs(props.instance.path)
	.then((allLogs) => {
		logs.value = buildLogList(allLogs)
	})
	.catch(handleError)

const selectedLogIndex = ref(0)
const isLive = computed(() => selectedLogIndex.value === 0)

const filteredLogs = computed(() =>
	props.playing ? logs.value.filter((l) => l.live || l.name !== 'latest.log') : logs.value,
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
	return log.filename === 'latest.log' && props.playing
})

async function deleteSelectedLog() {
	const log = selectedLog.value
	if (!log || log.live) return
	await delete_logs_by_filename(props.instance.path, log.log_type, log.filename)
	invalidate()
	const freshLogs = await getHistoricalLogs(props.instance.path)
	logs.value = buildLogList(freshLogs)
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
	shareDisabled: computed(() => props.offline),
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

	const output = await get_output_by_filename(
		props.instance.path,
		log.log_type,
		log.filename,
	).catch(handleError)
	if (output) {
		historicalConsole.clear()
		historicalConsole.addLegacyLog(output)
	}
})

selectedLogIndex.value = 0

if (!props.playing) {
	void analyseForCrash()
}

const unlistenLog = await log_listener((payload) => {
	if (payload.profile_path_id !== profilePathId.value) return

	if (payload.type === 'log4j') {
		liveConsole.addLog4jEvent(payload)
	} else if (payload.type === 'legacy') {
		liveConsole.addLegacyLog(payload.message)
	}
})

const unlistenProcesses = await process_listener(async (e) => {
	if (e.profile_path_id !== profilePathId.value) return
	if (e.event === 'launched') {
		liveConsole.clear()
		invalidate()
		selectedLogIndex.value = 0
	}
	if (e.event === 'finished') {
		invalidate()
		const freshLogs = await getHistoricalLogs(props.instance.path)
		logs.value = buildLogList(freshLogs)
		void analyseForCrash()
	}
})

onUnmounted(() => {
	unlistenLog()
	unlistenProcesses()
})
</script>
