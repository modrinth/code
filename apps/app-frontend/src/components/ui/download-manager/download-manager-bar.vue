<script setup lang="ts">
import { CheckIcon, CircleAlertIcon, DownloadIcon, LoaderSpinnerIcon, PauseIcon } from '@modrinth/assets'
import { defineMessages, truncatedTooltip, useFormatNumber, useVIntl } from '@modrinth/ui'
import {
	refDebounced,
	usePreferredReducedMotion,
	useResizeObserver,
	useWindowSize,
} from '@vueuse/core'
import { Motion } from 'motion-v'
import { computed, nextTick, onMounted, onScopeDispose, ref, useTemplateRef, watch } from 'vue'

import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{
	task: DownloadManagerJob | null
	activeCount: number
	progress: number
	hasAttention: boolean
	rate: string
	completing: boolean
	animated: boolean
	expanded: boolean
	panelId: string
}>()
defineEmits<{ toggle: []; close: [] }>()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const reducedMotion = usePreferredReducedMotion()
const { width: windowWidth } = useWindowSize()
const trigger = useTemplateRef('trigger')
const titleRef = useTemplateRef('title')
const titleMeasure = useTemplateRef('title-measure')
const speedMeasure = useTemplateRef('speed-measure')
const countMeasure = useTemplateRef('count-measure')
const measured = ref(false)
const titleWidth = ref(0)
const speedWidth = ref(0)
const countWidth = ref(20)
const rootFontSize = ref(16)
const measuredTitle = ref('')
const lastRate = ref('')
const displayedCount = ref(props.activeCount)
const countPulse = ref(false)
const compressed = ref(false)
let compressAfterCollapse = false
let compressionTimer: ReturnType<typeof setTimeout> | undefined
let pulseTimer: ReturnType<typeof setTimeout> | undefined

const messages = defineMessages({
	show: { id: 'app.download-manager.show', defaultMessage: 'Show installation tasks' },
	hide: { id: 'app.download-manager.hide', defaultMessage: 'Hide installation tasks' },
	attention: { id: 'app.download-manager.attention', defaultMessage: 'Needs attention' },
})

const animate = computed(() => props.animated && reducedMotion.value !== 'reduce')
const idle = computed(() => !props.task && !props.activeCount && !props.hasAttention)
const idleSize = computed(() => rootFontSize.value * 2 + 2)
const controlsPadding = computed(() => (idle.value ? (idleSize.value - 18) / 2 : 12))
const countLabel = computed(() => formatNumber(displayedCount.value))
const rateAvailable = refDebounced(
	computed(() => !!props.rate),
	500,
)
const hasRate = computed(() => rateAvailable.value && !props.completing && !!props.activeCount)
const taskFailed = computed(
	() => props.task?.status === 'failed' || props.task?.status === 'interrupted',
)
const icon = computed(() =>
	props.completing
		? 'complete'
		: taskFailed.value
			? 'failed'
			: props.task?.paused
				? 'paused'
				: 'running',
)
const controlsWidth = computed(
	() => 40 + (props.activeCount ? countWidth.value + 4 : 0) + (props.hasAttention ? 20 : 0),
)
const rateSlotWidth = computed(() => (hasRate.value ? speedWidth.value + 8 : 0))
const summaryWidth = computed(() => titleWidth.value + 47 + rateSlotWidth.value)
const expandedWidth = computed(() =>
	Math.min(
		controlsWidth.value + summaryWidth.value + 2,
		rootFontSize.value * 24,
		windowWidth.value * 0.4,
	),
)
const titleLayoutWidth = computed(() =>
	Math.max(0, expandedWidth.value - controlsWidth.value - 49 - rateSlotWidth.value),
)
const targetWidth = computed(() =>
	idle.value ? idleSize.value : props.task ? expandedWidth.value : controlsWidth.value + 2,
)
const resizeTransition = computed(() =>
	animate.value ? { type: 'spring' as const, stiffness: 260, damping: 32 } : { duration: 0 },
)
const contentTransition = computed(() => ({ duration: animate.value ? 0.18 : 0 }))

watch(idle, (isIdle) => {
	clearTimeout(compressionTimer)
	compressed.value = false
	compressAfterCollapse = isIdle && animate.value
})

function finishResize() {
	if (!compressAfterCollapse || !idle.value || !animate.value) return
	compressAfterCollapse = false
	compressed.value = true
	compressionTimer = setTimeout(() => {
		compressed.value = false
	}, 110)
}

function measure() {
	if (!titleMeasure.value || !speedMeasure.value || !countMeasure.value) return
	titleWidth.value = Math.ceil(titleMeasure.value.getBoundingClientRect().width)
	speedWidth.value = Math.ceil(speedMeasure.value.getBoundingClientRect().width)
	countWidth.value = Math.max(20, Math.ceil(countMeasure.value.getBoundingClientRect().width) + 10)
	rootFontSize.value = Number.parseFloat(getComputedStyle(document.documentElement).fontSize) || 16
	measured.value = true
}

watch(
	() => props.task?.title,
	async (title) => {
		if (title != null) measuredTitle.value = title
		await nextTick()
		measure()
	},
	{ immediate: true },
)
watch(
	() => props.rate,
	(rate) => {
		if (rate) lastRate.value = rate
	},
	{ immediate: true },
)
watch(
	() => props.activeCount,
	(count, previous) => {
		if (count) displayedCount.value = count
		clearTimeout(pulseTimer)
		countPulse.value = animate.value && count > previous
		if (countPulse.value) {
			pulseTimer = setTimeout(() => {
				countPulse.value = false
			}, 90)
		}
	},
)

useResizeObserver([titleMeasure, speedMeasure, countMeasure], measure)
onMounted(measure)
onScopeDispose(() => {
	clearTimeout(pulseTimer)
	clearTimeout(compressionTimer)
})
defineExpose({ focus: () => trigger.value?.focus() })
</script>

<template>
	<Motion
		as="div"
		class="relative flex origin-right shrink-0 text-sm font-medium leading-5 text-primary"
		:class="{ 'download-bar-static': !animate }"
		:initial="animate ? { width: controlsWidth + 2, opacity: 0 } : false"
		:animate="{
			width: targetWidth,
			height: idle ? idleSize : 32,
			borderRadius: idle ? idleSize / 2 : 12,
			scaleX: compressed && animate ? 0.9 : 1,
			opacity: measured ? 1 : 0,
		}"
		:transition="resizeTransition"
		@animation-complete="finishResize"
	>
		<span
			class="pointer-events-none invisible absolute left-0 top-0 h-0 w-full overflow-hidden whitespace-nowrap"
			aria-hidden="true"
		>
			<span ref="title-measure" class="inline-block">{{ measuredTitle }}</span>
			<span ref="speed-measure" class="inline-block w-[8ch] tabular-nums" />
			<span ref="count-measure" class="inline-block text-xs tabular-nums">{{ countLabel }}</span>
		</span>
		<button
			ref="trigger"
			type="button"
			class="relative isolate flex h-full w-full min-w-0 items-center overflow-hidden rounded-[inherit] border border-solid bg-transparent p-0 text-sm font-medium leading-5 text-primary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand"
			:class="
				completing
					? 'border-brand'
					: taskFailed
						? 'border-orange'
						: 'border-surface-5 hover:bg-surface-4'
			"
			:aria-label="formatMessage(expanded ? messages.hide : messages.show)"
			:aria-expanded="expanded"
			:aria-controls="expanded ? panelId : undefined"
			aria-haspopup="dialog"
			@click="$emit('toggle')"
			@keydown.esc.stop.prevent="$emit('close')"
		>
			<Motion
				as="span"
				class="pointer-events-none absolute inset-0 -z-10 origin-left transition-colors duration-300 motion-reduce:transition-none"
				:initial="false"
				:animate="{
					scaleX: completing || taskFailed ? 1 : Math.max(0, Math.min(1, progress)),
					opacity: activeCount || completing || taskFailed ? 1 : 0,
				}"
				:transition="{ duration: animate ? 0.3 : 0, ease: 'easeOut' }"
				:style="{
					backgroundColor: completing
						? 'var(--color-green-highlight)'
						: taskFailed
							? 'var(--color-orange-highlight)'
							: 'var(--surface-4)',
				}"
				aria-hidden="true"
			/>
			<span class="relative h-full min-w-0 flex-1 overflow-hidden">
				<span
					class="download-bar-summary absolute inset-0 flex items-center border-0 border-r border-solid px-3"
					:class="completing ? 'border-brand' : taskFailed ? 'border-orange' : 'border-surface-5'"
					:style="{ opacity: task ? 1 : 0 }"
					:aria-hidden="!task"
				>
					<span class="relative flex size-4 shrink-0 items-center justify-center">
						<Transition name="download-bar-icon" :css="animate">
							<CheckIcon
								v-if="icon === 'complete'"
								key="complete"
								class="absolute size-4 text-brand"
								aria-hidden="true"
							/>
							<CircleAlertIcon
								v-else-if="icon === 'failed'"
								key="failed"
								class="absolute size-4 text-orange"
								aria-hidden="true"
							/>
							<LoaderSpinnerIcon
								v-else-if="icon === 'running'"
								key="running"
								class="absolute size-5 motion-safe:animate-spin"
								aria-hidden="true"
							/>
							<PauseIcon
								v-else-if="icon === 'paused'"
								key="paused"
								class="absolute size-4"
								aria-hidden="true"
							/>
						</Transition>
					</span>
					<span
						class="relative ml-1.5 h-5 shrink-0 overflow-hidden text-left"
						:style="{ width: `${titleLayoutWidth}px` }"
					>
						<Transition name="download-bar-title" :css="animate" :appear="animate">
							<span
								v-if="task"
								:key="`${task.id}:${task.title}`"
								ref="title"
								v-tooltip="truncatedTooltip(titleRef, task.title)"
								class="block truncate text-contrast"
							>
								{{ task.title }}
							</span>
						</Transition>
					</span>
					<Motion
						as="span"
						class="shrink-0 overflow-hidden whitespace-nowrap text-right tabular-nums"
						:initial="false"
						:animate="{
							width: hasRate ? speedWidth : 0,
							marginLeft: hasRate ? 8 : 0,
							opacity: hasRate ? 1 : 0,
						}"
						:transition="resizeTransition"
						:aria-hidden="!hasRate"
					>
						<span class="inline-block w-[8ch]">{{ lastRate }}</span>
					</Motion>
				</span>
			</span>
			<Motion
				as="span"
				class="flex h-full shrink-0 items-center justify-center"
				:initial="false"
				:animate="{ paddingLeft: controlsPadding, paddingRight: controlsPadding }"
				:transition="resizeTransition"
			>
				<DownloadIcon
					class="size-4 shrink-0"
					:class="{ 'text-contrast': completing || taskFailed }"
					aria-hidden="true"
				/>
				<Motion
					as="span"
					class="relative h-5 shrink-0"
					:initial="false"
					:animate="{
						width: activeCount ? countWidth : 0,
						marginLeft: activeCount ? 4 : 0,
						opacity: activeCount ? 1 : 0,
					}"
					:transition="contentTransition"
					:aria-hidden="!activeCount"
				>
					<Motion
						as="span"
						class="absolute inset-y-0 left-0 grid h-5 place-items-center overflow-hidden rounded-full border border-solid border-brand text-xs font-medium leading-4 text-brand tabular-nums"
						:style="{
							width: `${countWidth}px`,
							backgroundColor: 'var(--color-green-highlight)',
						}"
						:initial="false"
						:animate="{ scale: countPulse ? 1.08 : 1 }"
						:transition="resizeTransition"
					>
						<Transition name="download-bar-count" :css="animate">
							<span :key="countLabel" class="col-start-1 row-start-1">{{ countLabel }}</span>
						</Transition>
					</Motion>
				</Motion>
				<Motion
					as="span"
					class="relative h-4 shrink-0"
					:initial="false"
					:animate="{
						width: hasAttention ? 16 : 0,
						marginLeft: hasAttention ? 4 : 0,
						opacity: hasAttention ? 1 : 0,
					}"
					:transition="contentTransition"
					:aria-hidden="!hasAttention"
				>
					<CircleAlertIcon
						v-tooltip="formatMessage(messages.attention)"
						class="absolute left-0 top-0 size-4 text-orange"
						:aria-label="formatMessage(messages.attention)"
					/>
				</Motion>
			</Motion>
		</button>
	</Motion>
</template>

<style scoped>
.download-bar-summary {
	transition:
		opacity 160ms ease-out,
		border-color 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

.download-bar-static .download-bar-summary {
	transition: none;
}

.download-bar-title-enter-active,
.download-bar-title-leave-active,
.download-bar-icon-enter-active,
.download-bar-icon-leave-active,
.download-bar-count-enter-active,
.download-bar-count-leave-active {
	transition:
		transform 160ms ease-out,
		opacity 160ms ease-out;
}

.download-bar-title-leave-active {
	position: absolute;
	inset: 0;
}

.download-bar-title-enter-from,
.download-bar-count-enter-from {
	transform: translateY(6px);
	opacity: 0;
}

.download-bar-title-leave-to,
.download-bar-count-leave-to {
	transform: translateY(-6px);
	opacity: 0;
}

.download-bar-icon-enter-from,
.download-bar-icon-leave-to {
	transform: scale(0.8);
	opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
	.download-bar-summary {
		transition: none;
	}
}
</style>
