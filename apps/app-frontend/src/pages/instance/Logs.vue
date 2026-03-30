<template>
	<div class="flex flex-col gap-4 h-full">
		<ConsolePageLayout />
	</div>
</template>

<script setup>
import { ConsolePageLayout, provideConsoleManager } from '@modrinth/ui'
import dayjs from 'dayjs'
import isToday from 'dayjs/plugin/isToday'
import isYesterday from 'dayjs/plugin/isYesterday'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { injectNotificationManager } from '@modrinth/ui'
import { process_listener } from '@/helpers/events.js'
import {
	delete_logs_by_filename,
	get_latest_log_cursor,
	get_logs,
	get_output_by_filename,
} from '@/helpers/logs.js'
import { get_by_profile_path } from '@/helpers/process.js'

dayjs.extend(isToday)
dayjs.extend(isYesterday)

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

const currentLiveLog = ref(null)
const currentLiveLogCursor = ref(0)
const emptyText = ['No live game detected.', 'Start your game to proceed.']

const logs = ref([])
await setLogs()

const selectedLogIndex = ref(0)
const interval = ref(null)

async function getLiveStdLog() {
	if (route.params.id) {
		const processes = await get_by_profile_path(route.params.id).catch(handleError)
		let returnValue
		if (processes.length === 0) {
			returnValue = emptyText.join('\n')
		} else {
			const logCursor = await get_latest_log_cursor(
				props.instance.path,
				currentLiveLogCursor.value,
			).catch(handleError)
			if (logCursor.new_file) {
				currentLiveLog.value = ''
			}
			currentLiveLog.value = currentLiveLog.value + logCursor.output
			currentLiveLogCursor.value = logCursor.cursor
			returnValue = currentLiveLog.value
		}
		return { name: 'Live Log', stdout: returnValue, live: true }
	}
	return null
}

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
	const [liveStd, allLogs] = await Promise.all([getLiveStdLog(), getLogs()])
	logs.value = [liveStd, ...allLogs]
}

const logLines = computed(() => {
	const log = logs.value[selectedLogIndex.value]
	return log?.stdout?.split('\n').filter(Boolean) ?? []
})

const logSources = computed(() =>
	logs.value.map((l, i) => ({
		id: String(i),
		name: l?.name ?? `Log ${i}`,
		live: l?.live ?? false,
	})),
)

provideConsoleManager({
	logLines,
	logSources,
	activeLogSourceIndex: selectedLogIndex,
	showCommandInput: false,
	loading: ref(false),
	onClear: () => {
		currentLiveLog.value = ''
	},
	shareDisabled: computed(() => props.offline),
})

watch(selectedLogIndex, async (newIndex) => {
	if (logs.value.length > 1 && newIndex !== 0) {
		logs.value[newIndex].stdout = 'Loading...'
		logs.value[newIndex].stdout = await get_output_by_filename(
			props.instance.path,
			logs.value[newIndex].log_type,
			logs.value[newIndex].filename,
		).catch(handleError)
	}
})

if (logs.value.length > 1 && !props.playing) {
	selectedLogIndex.value = 1
} else {
	selectedLogIndex.value = 0
}

const deleteLog = async () => {
	if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
		const deleteIndex = selectedLogIndex.value
		selectedLogIndex.value = deleteIndex - 1
		await delete_logs_by_filename(
			props.instance.path,
			logs.value[deleteIndex].log_type,
			logs.value[deleteIndex].filename,
		).catch(handleError)
		await setLogs()
	}
}

interval.value = setInterval(async () => {
	if (logs.value.length > 0 && selectedLogIndex.value === 0) {
		logs.value[0] = await getLiveStdLog()
	}
}, 250)

const unlistenProcesses = await process_listener(async (e) => {
	if (e.event === 'launched') {
		currentLiveLog.value = ''
		currentLiveLogCursor.value = 0
		selectedLogIndex.value = 0
	}
	if (e.event === 'finished') {
		currentLiveLog.value = ''
		currentLiveLogCursor.value = 0
		await setLogs()
		selectedLogIndex.value = 1
	}
})

onUnmounted(() => {
	clearInterval(interval.value)
	unlistenProcesses()
})
</script>
