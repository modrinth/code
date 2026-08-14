<template>
	<div
		:class="[
			'relative grid grid-cols-[1.5rem_minmax(0,1fr)_auto] gap-x-2 rounded-2xl border border-solid p-4 text-contrast',
			centerContent ? 'items-center' : 'items-start',
			progress != null ? 'overflow-hidden pb-5' : '',
			typeClasses[type],
		]"
	>
		<slot name="icon" :icon-class="['h-6 w-6 flex-none', iconClasses[type]]">
			<component :is="getSeverityIcon(type)" :class="['h-6 w-6 flex-none', iconClasses[type]]" />
		</slot>
		<div
			class="col-start-2 min-w-0"
			:class="
				inlineActions && !showActionsUnderneath && $slots.actions
					? ['flex flex-wrap gap-x-4 gap-y-3', centerContent ? 'items-center' : 'items-start']
					: 'flex flex-1 flex-col gap-2'
			"
		>
			<div
				class="flex min-w-0 flex-1 flex-col gap-2"
				:class="
					inlineActions && !showActionsUnderneath && $slots.actions
						? 'admonition-inline-content'
						: ''
				"
			>
				<div
					v-if="header || $slots.header || normalizedTimestamp"
					class="flex flex-wrap items-center gap-2 text-lg font-semibold leading-6"
				>
					<slot name="header">{{ header }}</slot>
					<span
						v-if="normalizedTimestamp"
						v-tooltip="timestampTooltip"
						class="flex items-center gap-1.5 text-base font-medium leading-normal text-secondary"
					>
						<ClockIcon class="size-4" />
						{{ relativeTimeLabel }}
					</span>
				</div>
				<div v-if="!!$slots.default || body" class="font-normal text-contrast/85 leading-tight">
					<slot>{{ body }}</slot>
				</div>
			</div>
			<div
				v-if="inlineActions && !showActionsUnderneath && $slots.actions"
				class="ml-auto flex shrink-0 items-center justify-end self-center"
			>
				<slot name="actions" />
			</div>
			<div v-else-if="showActionsUnderneath || $slots.actions" class="mt-2">
				<slot name="actions" />
			</div>
		</div>
		<div
			v-if="$slots['top-right-actions'] || dismissible"
			class="col-start-3 row-start-1 flex shrink-0 items-center gap-2 self-start"
		>
			<slot name="top-right-actions" />
			<IconButton
				v-if="dismissible"
				type="quiet"
				:color="
					buttonColors[type] && buttonColors[type] !== 'standard'
						? buttonColors[type] === 'medal-promo'
							? 'medal_promotion'
							: buttonColors[type]
						: undefined
				"
				label="Dismiss"
				native-type="button"
				@click="$emit('dismiss')"
			>
				<XIcon />
			</IconButton>
		</div>
		<div
			v-if="progress != null"
			class="absolute inset-x-0 bottom-0 h-1 overflow-hidden"
			:class="progressTrackClasses[type]"
			role="progressbar"
			:aria-valuenow="waiting ? undefined : Math.round(normalizedProgress * 100)"
			aria-valuemin="0"
			aria-valuemax="100"
		>
			<div
				class="h-full rounded-r-full transition-[width] duration-200 ease-in-out"
				:class="[
					progressFillClasses[progressColor ?? type],
					{ 'admonition-progress--waiting': waiting },
				]"
				:style="waiting ? undefined : { width: `${normalizedProgress * 100}%` }"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ClockIcon, XIcon } from '@modrinth/assets'
import { useNow } from '@vueuse/core'
import { computed } from 'vue'

import { IconButton } from '#ui/components/base/buttons'

import { useFormatDateTime, useRelativeTime } from '../../composables'
import { getSeverityIcon } from '../../utils'

const props = withDefaults(
	defineProps<{
		type?: 'info' | 'warning' | 'critical' | 'success' | 'moderation' | 'circle-warning' | 'neutral'
		header?: string
		body?: string
		inlineActions?: boolean
		showActionsUnderneath?: boolean
		dismissible?: boolean
		progress?: number
		progressColor?: 'info' | 'warning' | 'critical' | 'success' | 'blue' | 'green' | 'red'
		waiting?: boolean
		/** Accepts a Date, an ISO string, or a millisecond Unix timestamp. */
		timestamp?: Date | string | number
		centerContent?: boolean
	}>(),
	{
		type: 'info',
		header: '',
		body: '',
		inlineActions: false,
		showActionsUnderneath: false,
		dismissible: false,
		progress: undefined,
		progressColor: undefined,
		waiting: false,
		timestamp: undefined,
		centerContent: false,
	},
)

defineEmits<{
	dismiss: []
}>()

const relativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	dateStyle: 'long',
	timeStyle: 'short',
})
const now = useNow({ interval: 1000 })

const normalizedProgress = computed(() => Math.min(Math.max(props.progress ?? 0, 0), 1))

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

const timestampTooltip = computed(() => {
	const t = normalizedTimestamp.value
	return t ? formatDateTime(t) : ''
})

const typeClasses = {
	info: 'border-brand-blue bg-bg-blue',
	warning: 'border-brand-orange bg-bg-orange',
	'circle-warning': 'border-brand-orange bg-bg-orange',
	critical: 'border-brand-red bg-bg-red',
	success: 'border-brand-green bg-bg-green',
	moderation: 'border-brand-orange bg-bg-orange',
	neutral: 'border-surface-4 bg-surface-3',
}

const iconClasses = {
	info: 'text-brand-blue',
	warning: 'text-brand-orange',
	'circle-warning': 'text-brand-orange',
	critical: 'text-brand-red',
	success: 'text-brand-green',
	moderation: 'text-brand-orange',
	neutral: 'text-secondary',
}

const buttonColors = {
	info: 'blue',
	warning: 'orange',
	'circle-warning': 'orange',
	critical: 'red',
	success: 'green',
	moderation: 'orange',
	neutral: 'standard',
} as const

const progressTrackClasses = {
	info: 'bg-brand-blue/20',
	warning: 'bg-brand-orange/20',
	'circle-warning': 'bg-brand-orange/20',
	critical: 'bg-brand-red/20',
	success: 'bg-brand-green/20',
	moderation: 'bg-brand-orange/20',
	neutral: 'bg-surface-4',
}

const progressFillClasses = {
	info: 'bg-brand-blue',
	warning: 'bg-brand-orange',
	'circle-warning': 'bg-brand-orange',
	critical: 'bg-brand-red',
	success: 'bg-brand-green',
	blue: 'bg-brand-blue',
	green: 'bg-brand-green',
	red: 'bg-brand-red',
	neutral: 'bg-surface-5',
}
</script>

<style scoped>
.admonition-inline-content {
	min-width: min(100%, 16rem);
}

.admonition-progress--waiting {
	animation: admonition-progress-waiting 1s linear infinite;
	position: relative;
	width: 20%;
}

@keyframes admonition-progress-waiting {
	0% {
		left: -20%;
	}

	100% {
		left: 100%;
	}
}
</style>
