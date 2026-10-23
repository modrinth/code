<template>
	<div class="flex h-full min-h-0 flex-col">
		<div
			class="flex shrink-0 flex-wrap items-center gap-1 border-0 border-b border-solid border-surface-5 px-2 py-1"
		>
			<Button
				v-for="level in LEVELS"
				:key="level"
				type="quiet"
				size="xs"
				:aria-pressed="shownLevels.has(level)"
				:class="shownLevels.has(level) ? '!text-contrast' : '!text-secondary opacity-60'"
				@click="toggleLevel(level)"
			>
				{{ formatMessage(levelMessages[level]) }}
			</Button>
			<span class="ml-auto text-xs text-secondary tabular-nums">
				{{ formatMessage(messages.count, { shown: shown.length, total: entries.length }) }}
			</span>
			<Button type="quiet" size="xs" @click="clear">
				{{ formatMessage(commonMessages.clearButton) }}
			</Button>
		</div>
		<div ref="list" class="min-h-0 flex-1 overflow-auto font-mono text-xs" @scroll="onScroll">
			<p v-if="shown.length === 0" class="m-0 p-3 font-sans text-sm text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<div
				v-for="entry in shown"
				:key="entry.id"
				class="flex gap-2 border-0 border-b border-solid border-surface-4 px-2 py-0.5"
			>
				<span class="shrink-0 text-secondary tabular-nums">{{ formatTime(entry.time) }}</span>
				<span class="w-14 shrink-0 font-semibold uppercase" :class="levelColors[entry.level]">
					{{ entry.level }}
				</span>
				<span class="shrink-0 text-secondary">{{ entry.logger }}</span>
				<span class="min-w-0 whitespace-pre-wrap break-words text-primary">
					{{ entry.message }}
					<template v-if="entry.error">
						<br />
						<span class="text-red">{{ entry.error }}</span>
					</template>
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { LogLevel } from '#ui/helpers/toolbox/misc/logging'
import { commonMessages } from '#ui/utils/common-messages'

import { useToolboxLogs } from '../../composables/toolbox-logs'

const { formatMessage } = useVIntl()
const formatTime = useFormatDateTime({ timeStyle: 'medium' })

const messages = defineMessages({
	empty: { id: 'archive-viewer.logs.empty', defaultMessage: 'Nothing has been logged yet.' },
	count: { id: 'archive-viewer.logs.count', defaultMessage: '{shown} of {total}' },
	levelError: { id: 'archive-viewer.logs.level-error', defaultMessage: 'Errors' },
	levelWarning: { id: 'archive-viewer.logs.level-warning', defaultMessage: 'Warnings' },
	levelInfo: { id: 'archive-viewer.logs.level-info', defaultMessage: 'Info' },
	levelDebug: { id: 'archive-viewer.logs.level-debug', defaultMessage: 'Debug' },
})

const LEVELS = ['error', 'warning', 'info', 'debug'] as const
type FilterLevel = (typeof LEVELS)[number]

const levelMessages = {
	error: messages.levelError,
	warning: messages.levelWarning,
	info: messages.levelInfo,
	debug: messages.levelDebug,
} satisfies Record<FilterLevel, unknown>

const levelColors: Record<LogLevel, string> = {
	error: 'text-red',
	warning: 'text-orange',
	success: 'text-green',
	info: 'text-blue',
	neutral: 'text-secondary',
	debug: 'text-secondary',
}

/** Success and neutral entries are filtered along with info ones. */
function filterLevel(level: LogLevel): FilterLevel {
	return level === 'success' || level === 'neutral' ? 'info' : level
}

const { entries, clear } = useToolboxLogs()

/** Debug entries (e.g. every worker message) are hidden until asked for. */
const shownLevels = ref(new Set<FilterLevel>(['error', 'warning', 'info']))

function toggleLevel(level: FilterLevel) {
	const next = new Set(shownLevels.value)
	if (next.has(level)) next.delete(level)
	else next.add(level)
	shownLevels.value = next
}

const shown = computed(() =>
	entries.value.filter((entry) => shownLevels.value.has(filterLevel(entry.level))),
)

const list = ref<HTMLElement | null>(null)
/** Follows new entries while scrolled to the bottom. */
const following = ref(true)

function onScroll() {
	const element = list.value
	if (!element) return
	following.value = element.scrollHeight - element.scrollTop - element.clientHeight < 16
}

watch(
	() => shown.value.length,
	async () => {
		if (!following.value) return
		await nextTick()
		list.value?.scrollTo({ top: list.value.scrollHeight })
	},
	{ immediate: true },
)
</script>
