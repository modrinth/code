<template>
	<div class="flex flex-col gap-4 h-full">
		<ConsolePageLayout />
	</div>
</template>

<script setup>
import {
	ConsolePageLayout,
	createConsoleState,
	injectNotificationManager,
	provideConsoleManager,
} from '@modrinth/ui'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { log_listener, process_listener } from '@/helpers/events.js'
import { get_logs, get_output_by_filename } from '@/helpers/logs.js'

const { handleError } = injectNotificationManager()
const route = useRoute()
const liveConsole = createConsoleState()
const historicalConsole = createConsoleState()

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
	const log = filteredLogs.value[selectedLogIndex.value]
	const text = log?.stdout ?? ''
	if (!text || text === 'Loading...') return
	historicalConsole.clear()
	historicalConsole.addLegacyLog(text)
}

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

provideConsoleManager({
	logLines: computed(() => activeConsole.value.output.value),
	logSources,
	activeLogSourceIndex: selectedLogIndex,
	showCommandInput: false,
	loading: ref(false),
	onClear: () => {
		activeConsole.value.clear()
	},
	shareDisabled: computed(() => props.offline),
})

watch(selectedLogIndex, async (newIndex) => {
	if (newIndex === 0) return
	const log = filteredLogs.value[newIndex]
	if (!log) return
	log.stdout = 'Loading...'
	log.stdout = await get_output_by_filename(
		props.instance.path,
		log.log_type,
		log.filename,
	).catch(handleError)
	loadHistoricalLog()
})

if (filteredLogs.value.length > 1 && !props.playing) {
	selectedLogIndex.value = 1
} else {
	selectedLogIndex.value = 0
}

const profilePathId = computed(() => route.params.id)

const unlistenLog = await log_listener((payload) => {
	if (payload.profile_path_id !== profilePathId.value) return

	if (payload.type === 'log4j') {
		liveConsole.addLog4jEvent(payload)
	} else if (payload.type === 'legacy') {
		liveConsole.addLegacyLog(payload.message)
	}
})

const unlistenProcesses = await process_listener(async (e) => {
	if (e.event === 'launched') {
		liveConsole.clear()
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
