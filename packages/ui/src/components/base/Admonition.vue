<template>
	<div
		:class="[
			'flex items-start gap-3 rounded-2xl border border-solid p-4 text-contrast',
			typeClasses[type],
		]"
	>
		<slot name="icon" :icon-class="['h-6 w-6 flex-none', iconClasses[type]]">
			<component
				:is="getSeverityIcon(type)"
				:class="['h-6 w-6 flex-none', iconClasses[type]]"
			/>
		</slot>
		<div class="flex min-w-0 flex-1 flex-col gap-1">
			<div
				v-if="header || $slots.header || normalizedTimestamp"
				class="flex flex-wrap items-center gap-2 text-base font-semibold"
			>
				<slot name="header">{{ header }}</slot>
				<span
					v-if="normalizedTimestamp"
					class="flex items-center gap-1.5 font-medium text-secondary"
				>
					<ClockIcon class="size-4" />
					{{ relativeTimeLabel }}
				</span>
			</div>
			<div class="font-normal text-contrast/85">
				<slot>{{ body }}</slot>
			</div>
			<div v-if="$slots.progress" class="mt-2">
				<slot name="progress" />
			</div>
			<div v-if="showActionsUnderneath || $slots.actions" class="mt-2">
				<slot name="actions" />
			</div>
		</div>
		<div
			v-if="$slots['top-right-actions'] || dismissible"
			class="flex shrink-0 items-center gap-2 self-start"
		>
			<slot name="top-right-actions" />
			<button
				v-if="dismissible"
				type="button"
				aria-label="Dismiss"
				:class="['transition-opacity opacity-70 hover:opacity-100', iconClasses[type]]"
				@click="$emit('dismiss')"
			>
				<XIcon class="h-6 w-6" />
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ClockIcon, XIcon } from '@modrinth/assets'
import { useNow } from '@vueuse/core'
import { computed } from 'vue'

import { useRelativeTime } from '../../composables'
import { getSeverityIcon } from '../../utils'

const props = withDefaults(
	defineProps<{
		type?: 'info' | 'warning' | 'critical' | 'success'
		header?: string
		body?: string
		showActionsUnderneath?: boolean
		dismissible?: boolean
		/** Accepts a Date, an ISO string, or a millisecond Unix timestamp. */
		timestamp?: Date | string | number
	}>(),
	{
		type: 'info',
		header: '',
		body: '',
		showActionsUnderneath: false,
		dismissible: false,
		timestamp: undefined,
	},
)

defineEmits<{
	dismiss: []
}>()

const relativeTime = useRelativeTime()
const now = useNow({ interval: 1000 })

const normalizedTimestamp = computed(() => {
	const t = props.timestamp
	if (t == null) return null
	if (t instanceof Date) return t.toISOString()
	if (typeof t === 'number') return new Date(t).toISOString()
	return t
})

const relativeTimeLabel = computed(() => {
	void now.value
	const t = normalizedTimestamp.value
	return t ? relativeTime(t) : ''
})

const typeClasses = {
	info: 'border-brand-blue bg-bg-blue',
	warning: 'border-brand-orange bg-bg-orange',
	critical: 'border-brand-red bg-bg-red',
	success: 'border-brand-green bg-bg-green',
}

const iconClasses = {
	info: 'text-brand-blue',
	warning: 'text-brand-orange',
	critical: 'text-brand-red',
	success: 'text-brand-green',
}
</script>
