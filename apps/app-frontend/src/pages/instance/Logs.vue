<template>
	<div class="flex flex-col gap-4 h-full">
		<ConsolePageLayout />
	</div>
</template>

<script setup>
import { ConsolePageLayout, injectNotificationManager, provideConsoleManager } from '@modrinth/ui'
import { computed, onUnmounted, ref, shallowRef, triggerRef, watch, watchEffect } from 'vue'
import { useRoute } from 'vue-router'

import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { log_listener, process_listener } from '@/helpers/events.js'
import { get_output_by_filename } from '@/helpers/logs.js'

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
} = useInstanceConsole(profilePathId.value)

await hydrate()

const allLogs = await getHistoricalLogs(props.instance.path)
const logs = ref([
	{ name: 'Live Log', live: true },
	...allLogs
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
])

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

provideConsoleManager({
	logLines,
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

if (filteredLogs.value.length > 1 && !props.playing) {
	selectedLogIndex.value = 1
} else {
	selectedLogIndex.value = 0
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
		logs.value = [
			{ name: 'Live Log', live: true },
			...freshLogs
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
		if (filteredLogs.value.length > 1) {
			selectedLogIndex.value = 1
		}
	}
})

onUnmounted(() => {
	unlistenLog()
	unlistenProcesses()
})
</script>
