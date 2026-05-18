<template>
	<div
		v-if="uptimeSeconds || uptimeSeconds !== 0"
		v-tooltip="`Online for ${verboseUptime}`"
		class="server-action-buttons-anim flex min-w-0 flex-row items-center gap-2"
		data-pyro-uptime
	>
		<Separator v-if="!noSeparator" />

		<div class="flex gap-1.5">
			<time class="truncate text-sm font-medium" :aria-label="verboseUptime">
				{{ formattedUptime }}
			</time>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import Separator from './Separator.vue'

const props = defineProps<{
	uptimeSeconds: number
	noSeparator?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	verboseDays: {
		id: 'servers.uptime.verbose.days',
		defaultMessage: '{count, plural, one {# day} other {# days}}',
	},
	verboseHours: {
		id: 'servers.uptime.verbose.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	verboseMinutes: {
		id: 'servers.uptime.verbose.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
	verboseSeconds: {
		id: 'servers.uptime.verbose.seconds',
		defaultMessage: '{count, plural, one {# second} other {# seconds}}',
	},
})

const formattedUptime = computed(() => {
	const days = Math.floor(props.uptimeSeconds / (24 * 3600))
	const hours = Math.floor((props.uptimeSeconds % (24 * 3600)) / 3600)
	const minutes = Math.floor((props.uptimeSeconds % 3600) / 60)
	const seconds = props.uptimeSeconds % 60

	let formatted = ''
	if (days > 0) {
		formatted += `${days}d `
	}
	if (hours > 0 || days > 0) {
		formatted += `${hours}h `
	}
	formatted += `${minutes}m ${seconds}s`

	return formatted.trim()
})

const verboseUptime = computed(() => {
	const days = Math.floor(props.uptimeSeconds / (24 * 3600))
	const hours = Math.floor((props.uptimeSeconds % (24 * 3600)) / 3600)
	const minutes = Math.floor((props.uptimeSeconds % 3600) / 60)
	const seconds = props.uptimeSeconds % 60

	let verbose = ''
	if (days > 0) {
		verbose += `${formatMessage(messages.verboseDays, { count: days })} `
	}
	if (hours > 0) {
		verbose += `${formatMessage(messages.verboseHours, { count: hours })} `
	}
	if (minutes > 0) {
		verbose += `${formatMessage(messages.verboseMinutes, { count: minutes })} `
	}
	verbose += formatMessage(messages.verboseSeconds, { count: seconds })

	return verbose.trim()
})
</script>
