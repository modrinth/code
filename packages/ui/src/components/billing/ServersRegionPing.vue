<script setup lang="ts">
import { NoSignalIcon, SignalIcon, SpinnerIcon } from '@modrinth/assets'
import { getPingLevel } from '@modrinth/utils'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'

const { formatMessage } = useVIntl()

const props = defineProps<{
	/** Latency in ms, `-1` if pinging failed, or `undefined` while pinging. */
	ping?: number
}>()

const pingLevel = computed(() => getPingLevel(props.ping ?? 0))

const messages = defineMessages({
	testing: {
		id: 'servers.region.ping.testing',
		defaultMessage: 'Testing connection...',
	},
	failed: {
		id: 'servers.region.ping.failed',
		defaultMessage: 'Failed to ping',
	},
	latency: {
		id: 'servers.region.ping.latency',
		defaultMessage: '{ping}ms',
	},
})
</script>

<template>
	<span class="flex gap-1 items-center tabular-nums">
		<template v-if="ping === undefined">
			<SpinnerIcon class="animate-spin shrink-0" />
			{{ formatMessage(messages.testing) }}
		</template>
		<template v-else-if="ping <= 0">
			<NoSignalIcon aria-hidden="true" stroke-width="3px" class="shrink-0" />
			{{ formatMessage(messages.failed) }}
		</template>
		<template v-else>
			<SignalIcon
				aria-hidden="true"
				:style="`--_signal-${pingLevel}: ${pingLevel <= 2 ? 'var(--color-red)' : pingLevel <= 4 ? 'var(--color-orange)' : 'var(--color-green)'}`"
				stroke-width="3px"
				class="shrink-0"
			/>
			{{ formatMessage(messages.latency, { ping }) }}
		</template>
	</span>
</template>
