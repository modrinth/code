<template>
	<div class="flex flex-col gap-4 h-full">
		<ConsolePageLayout />
	</div>
</template>

<script setup>
import {
	ConsolePageLayout,
	injectNotificationManager,
	provideConsoleManager,
	useModrinthServersConsole,
} from '@modrinth/ui'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { log_listener, process_listener } from '@/helpers/events.js'
import { get_logs, get_output_by_filename } from '@/helpers/logs.js'

const { handleError } = injectNotificationManager()
const route = useRoute()
const consoleLines = useModrinthServersConsole()

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

const logs = ref([])
await setLogs()

const selectedLogIndex = ref(0)

async function getLogs() {
	return (await get_logs(props.instance.path, true).catch(handleError))
		.filter(
			(log) =>
				log.filename !== 'latest_stdout.log' &&
				log.filename !== 'latest_stdout' &&
				log.stdout !== '' &&
				(log.filename.includes('.log') || log.filename.endsWith('.txt')),
		)
		.map((log) => {
			log.name = log.filename || 'Unknown'
			log.stdout = 'Loading...'
			return log
		})
}

async function setLogs() {
	const allLogs = await getLogs()
	logs.value = [{ name: 'Live Log', live: true }, ...allLogs]
}

const isLive = computed(() => selectedLogIndex.value === 0)

const loadHistoricalLog = () => {
	const log = logs.value[selectedLogIndex.value]
	const text = log?.stdout ?? ''
	if (!text || text === 'Loading...') return
	consoleLines.clear()
	consoleLines.addLegacyLog(text)
}

const logSources = computed(() =>
	logs.value.map((l, i) => ({
		id: String(i),
		name: l?.name ?? `Log ${i}`,
		live: l?.live ?? false,
	})),
)

provideConsoleManager({
	logLines: consoleLines.output,
	logSources,
	activeLogSourceIndex: selectedLogIndex,
	showCommandInput: false,
	loading: ref(false),
	onClear: () => {
		consoleLines.clear()
	},
	shareDisabled: computed(() => props.offline),
})

watch(selectedLogIndex, async (newIndex) => {
	if (newIndex === 0) {
		consoleLines.clear()
		return
	}
	if (logs.value.length > 1) {
		logs.value[newIndex].stdout = 'Loading...'
		logs.value[newIndex].stdout = await get_output_by_filename(
			props.instance.path,
			logs.value[newIndex].log_type,
			logs.value[newIndex].filename,
		).catch(handleError)
		loadHistoricalLog()
	}
})

if (logs.value.length > 1 && !props.playing) {
	selectedLogIndex.value = 1
} else {
	selectedLogIndex.value = 0
}

const profilePathId = computed(() => route.params.id)

const unlistenLog = await log_listener((payload) => {
	if (payload.profile_path_id !== profilePathId.value) return
	if (!isLive.value) return

	if (payload.type === 'log4j') {
		consoleLines.addLog4jEvent(payload)
	} else if (payload.type === 'legacy') {
		consoleLines.addLegacyLog(payload.message)
	}
})

const unlistenProcesses = await process_listener(async (e) => {
	if (e.event === 'launched') {
		consoleLines.clear()
		selectedLogIndex.value = 0
	}
	if (e.event === 'finished') {
		await setLogs()
		selectedLogIndex.value = 1
	}
})

onUnmounted(() => {
	unlistenLog()
	unlistenProcesses()
})
</script>
