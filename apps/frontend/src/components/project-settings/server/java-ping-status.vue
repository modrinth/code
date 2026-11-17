<script setup lang="ts">
import { InfoIcon, RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import { IconButton, IntlFormatted, normalizeChildren, useVIntl } from '@modrinth/ui'

import { messages } from './messages'

defineProps<{
	javaAddress: string
	javaPingLoading: boolean
	javaPingResult: { online: boolean; latency: number | null } | null
}>()

const emit = defineEmits<{ refresh: [] }>()
const { formatMessage } = useVIntl()
</script>

<template>
	<div
		v-if="javaAddress"
		class="mt-2 flex gap-1.5"
		:class="{
			'items-center': javaPingResult?.online,
			'items-start': javaPingResult && !javaPingResult.online,
		}"
	>
		<IconButton
			v-if="(javaAddress && javaPingResult) || javaPingLoading"
			v-tooltip="formatMessage(messages.refreshPingTooltip)"
			class="!size-6"
			type="quiet"
			color="orange"
			size="xs"
			:label="formatMessage(messages.refreshPingTooltip)"
			:disabled="javaPingLoading"
			@click="emit('refresh')"
		>
			<SpinnerIcon v-if="javaPingLoading" class="animate-spin" />
			<RefreshCwIcon v-else />
		</IconButton>
		<div
			v-if="javaPingResult !== null && !javaPingLoading && javaPingResult.online"
			class="mt-0.5 flex items-center gap-1.5 text-green"
		>
			{{ formatMessage(messages.serverOnline) }}
			<template v-if="javaPingResult.latency">
				{{
					formatMessage(messages.latencyLabel, {
						latency: javaPingResult.latency,
					})
				}}
			</template>
		</div>
		<div v-else-if="javaPingResult !== null && !javaPingLoading" class="mt-0.5 text-orange">
			<IntlFormatted :message-id="messages.pingFailedMessage">
				<template #support-link="{ children }">
					<a
						class="inline underline"
						href="https://support.modrinth.com"
						target="_blank"
						rel="noopener noreferrer"
						><component :is="() => normalizeChildren(children)"
					/></a>
				</template>
			</IntlFormatted>
		</div>
	</div>
	<div v-else class="mt-2 text-sm">
		<IntlFormatted :message-id="messages.srvRecordsHint">
			<template #srv-tooltip="{ children }"
				><component :is="() => normalizeChildren(children)" /><InfoIcon
					v-tooltip="{
						content: formatMessage(messages.srvRecordsTooltip),
						popperClass: 'max-w-xs',
					}"
			/></template>
		</IntlFormatted>
	</div>
</template>
