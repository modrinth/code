<template>
	<NewModal
		ref="modal"
		:scrollable="true"
		max-content-height="72vh"
		:on-hide="onModalHide"
		:closable="true"
		:close-on-click-outside="false"
		:width="resolvedMaxWidth"
		:disable-close="resolveCtxFn(currentStage.disableClose, context)"
	>
		<template #title>
			<div
				v-if="breadcrumbs && !resolveCtxFn(currentStage.nonProgressStage, context)"
				class="relative w-full"
			>
				<div
					class="pointer-events-none absolute left-0 top-0 bottom-0 w-8 bg-gradient-to-r from-bg-raised to-transparent z-10 transition-opacity duration-200"
					:class="showLeftShadow ? 'opacity-100' : 'opacity-0'"
				/>
				<div
					ref="breadcrumbScroller"
					class="flex w-full overflow-x-auto overflow-y-hidden scrollbar-hide pr-6"
					@wheel.prevent="onBreadcrumbWheel"
					@scroll="updateScrollShadows"
				>
					<template v-for="(stage, index) in breadcrumbStages" :key="stage.id">
						<div
							:ref="(el) => setBreadcrumbRef(stage.id, el as HTMLElement | null)"
							class="flex w-max items-center"
						>
							<button
								class="bg-transparent active:scale-95 font-bold text-secondary p-0 w-max py-3 px-1"
								:class="{
									'!text-contrast font-bold': resolveCtxFn(currentStage.id, context) === stage.id,
									'font-bold': resolveCtxFn(currentStage.id, context) !== stage.id,
									'opacity-50 cursor-not-allowed': cannotNavigateToStage(index),
								}"
								:disabled="cannotNavigateToStage(index)"
								@click="setStage(stage.id)"
							>
								{{ resolveCtxFn(stage.title, context) }}
							</button>
							<ChevronRightIcon
								v-if="index < breadcrumbStages.length - 1"
								class="h-5 w-5 text-secondary"
								stroke-width="3"
							/>
						</div>
					</template>
				</div>
				<div
					class="pointer-events-none absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-bg-raised to-transparent z-10 transition-opacity duration-200"
					:class="showRightShadow ? 'opacity-100' : 'opacity-0'"
				/>
			</div>
			<span v-else class="text-lg font-bold text-contrast sm:text-xl">{{ resolvedTitle }}</span>
		</template>

		<progress
			v-if="nonProgressStage !== true"
			:value="progressValue"
			max="100"
			class="w-full h-1 appearance-none border-none absolute top-0 left-0"
		></progress>

		<component :is="currentStage?.stageContent" />

		<template #actions>
			<div
				class="flex flex-col justify-end gap-2 sm:flex-row"
				:class="leftButtonConfig || rightButtonConfig ? 'mt-4' : ''"
			>
				<ButtonStyled v-if="leftButtonConfig" type="outlined">
					<button
						class="!border-surface-5"
						:disabled="leftButtonConfig.disabled"
						@click="leftButtonConfig.onClick"
					>
						<component :is="leftButtonConfig.icon" />
						{{ leftButtonConfig.label }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="rightButtonConfig" :color="rightButtonConfig.color">
					<button :disabled="rightButtonConfig.disabled" @click="rightButtonConfig.onClick">
						<component
							:is="rightButtonConfig.icon"
							v-if="rightButtonConfig.iconPosition === 'before'"
							:class="rightButtonConfig.iconClass"
						/>
						{{ rightButtonConfig.label }}
						<component
							:is="rightButtonConfig.icon"
							v-if="rightButtonConfig.iconPosition === 'after'"
							:class="rightButtonConfig.iconClass"
						/>
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

export interface StageButtonConfig {
	label?: string
	icon?: Component | null
	iconPosition?: 'before' | 'after'
	color?: InstanceType<typeof ButtonStyled>['$props']['color']
	disabled?: boolean
	iconClass?: string | null
	onClick?: () => void
}

export type MaybeCtxFn<T, R> = R | ((ctx: T) => R)

export interface StageConfigInput<T> {
	id: string
	stageContent: Component
	title: MaybeCtxFn<T, string>
	skip?: MaybeCtxFn<T, boolean>
	hideStageInBreadcrumb?: MaybeCtxFn<T, boolean>
	nonProgressStage?: MaybeCtxFn<T, boolean>
	cannotNavigateForward?: MaybeCtxFn<T, boolean>
	disableClose?: MaybeCtxFn<T, boolean>
	leftButtonConfig: MaybeCtxFn<T, StageButtonConfig | null>
	rightButtonConfig: MaybeCtxFn<T, StageButtonConfig | null>
	/** Max width for the modal content and header defined in px (e.g., '460px', '600px'). Defaults to '460px'. */
	maxWidth?: MaybeCtxFn<T, string>
}

export function resolveCtxFn<T, R>(value: MaybeCtxFn<T, R>, ctx: T): R {
	return typeof value === 'function' ? (value as (ctx: T) => R)(ctx) : value
}
</script>

<script setup lang="ts" generic="T">
const props = defineProps<{
	stages: StageConfigInput<T>[]
	context: T
	breadcrumbs?: boolean
	fitContent?: boolean
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const currentStageIndex = ref<number>(0)

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

const setStage = (indexOrId: number | string) => {
	let index: number = 0
	if (typeof indexOrId === 'number') {
		index = indexOrId
		if (index < 0 || index >= props.stages.length) return
	} else {
		index = props.stages.findIndex((stage) => stage.id === indexOrId)
		if (index === -1) return
	}
	while (index < props.stages.length) {
		const skip = props.stages[index]?.skip
		if (!skip || !resolveCtxFn(skip, props.context)) break
		index++
	}
	if (index < props.stages.length) {
		currentStageIndex.value = index
	}
}

const nextStage = () => {
	if (currentStageIndex.value === -1) return
	if (currentStageIndex.value >= props.stages.length - 1) return
	let nextIndex = currentStageIndex.value + 1
	while (nextIndex < props.stages.length) {
		const skip = props.stages[nextIndex]?.skip
		if (!skip || !resolveCtxFn(skip, props.context)) break
		nextIndex++
	}
	if (nextIndex < props.stages.length) {
		currentStageIndex.value = nextIndex
	}
}

const prevStage = () => {
	if (currentStageIndex.value <= 0) return
	let prevIndex = currentStageIndex.value - 1
	while (prevIndex >= 0) {
		const skip = props.stages[prevIndex]?.skip
		if (!skip || !resolveCtxFn(skip, props.context)) break
		prevIndex--
	}
	if (prevIndex >= 0) {
		currentStageIndex.value = prevIndex
	}
}

const currentStage = computed(() => props.stages[currentStageIndex.value])

const resolvedTitle = computed(() => {
	const stage = currentStage.value
	if (!stage) return ''
	return resolveCtxFn(stage.title, props.context)
})

const leftButtonConfig = computed(() => {
	const stage = currentStage.value
	if (!stage) return null
	return resolveCtxFn(stage.leftButtonConfig, props.context)
})

const rightButtonConfig = computed(() => {
	const stage = currentStage.value
	if (!stage) return null
	return resolveCtxFn(stage.rightButtonConfig, props.context)
})

const nonProgressStage = computed(() => {
	const stage = currentStage.value
	if (!stage) return false
	return resolveCtxFn(stage.nonProgressStage, props.context)
})

const resolvedMaxWidth = computed(() => {
	const stage = currentStage.value
	if (!stage?.maxWidth) return '560px'
	return resolveCtxFn(stage.maxWidth, props.context)
})

const progressValue = computed(() => {
	const isProgressStage = (stage: StageConfigInput<T>) => {
		if (resolveCtxFn(stage.nonProgressStage, props.context)) return false
		const skip = stage.skip ? resolveCtxFn(stage.skip, props.context) : false
		return !skip
	}

	const completedCount = props.stages
		.slice(0, currentStageIndex.value + 1)
		.filter(isProgressStage).length
	const totalCount = props.stages.filter(isProgressStage).length

	return totalCount > 0 ? (completedCount / totalCount) * 100 : 0
})

const breadcrumbScroller = ref<HTMLElement | null>(null)
const breadcrumbRefs = ref<Map<string, HTMLElement>>(new Map())
const showLeftShadow = ref(false)
const showRightShadow = ref(false)

function setBreadcrumbRef(stageId: string, el: HTMLElement | null) {
	if (el) breadcrumbRefs.value.set(stageId, el)
	else breadcrumbRefs.value.delete(stageId)
}

function scrollToCurrentBreadcrumb() {
	const stage = currentStage.value
	if (!stage || !breadcrumbScroller.value) return

	const el = breadcrumbRefs.value.get(stage.id)
	if (!el) return

	nextTick(() => {
		breadcrumbScroller.value?.scrollTo({
			left: el.offsetLeft - 50,
			behavior: 'smooth',
		})
	})
}

function updateScrollShadows() {
	const el = breadcrumbScroller.value
	if (!el) {
		showLeftShadow.value = false
		showRightShadow.value = false
		return
	}

	showLeftShadow.value = el.scrollLeft > 0
	showRightShadow.value = el.scrollLeft < el.scrollWidth - el.clientWidth - 1
}

function onBreadcrumbWheel(e: WheelEvent) {
	if (!breadcrumbScroller.value) return

	const el = breadcrumbScroller.value
	const canScrollHorizontally = el.scrollWidth > el.clientWidth

	if (canScrollHorizontally) {
		// Support both horizontal and vertical scroll input
		const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY
		el.scrollLeft += delta
	}
}

// Stages that are not skipped (visible in breadcrumbs)
const breadcrumbStages = computed(() => {
	return props.stages.filter((stage) => {
		const visibleStep =
			!resolveCtxFn(stage.skip, props.context) &&
			!resolveCtxFn(stage.nonProgressStage, props.context) &&
			!resolveCtxFn(stage.hideStageInBreadcrumb, props.context)
		return visibleStep
	})
})

// Check if navigation to a breadcrumb stage is allowed
// Navigation backwards is always allowed, but forward navigation requires all intermediate stages to allow it
function cannotNavigateToStage(breadcrumbIndex: number): boolean {
	const targetStage = breadcrumbStages.value[breadcrumbIndex]
	if (!targetStage) return false

	const targetStageIndex = props.stages.findIndex((s) => s.id === targetStage.id)
	if (targetStageIndex === -1) return false

	// Always allow navigating to current or previous stages
	if (targetStageIndex <= currentStageIndex.value) return false

	// For forward navigation, check all stages between current and target
	for (let i = currentStageIndex.value; i < targetStageIndex; i++) {
		const stage = props.stages[i]
		if (stage.skip && resolveCtxFn(stage.skip, props.context)) continue
		if (resolveCtxFn(stage.cannotNavigateForward, props.context)) {
			return true
		}
	}

	return false
}

watch([breadcrumbStages, currentStageIndex], () => nextTick(() => updateScrollShadows()), {
	immediate: true,
})

watch(currentStageIndex, () => {
	scrollToCurrentBreadcrumb()
})

const emit = defineEmits<{
	(e: 'refresh-data' | 'hide'): void
}>()

function onModalHide() {
	emit('hide')
}

defineExpose({
	show,
	hide,
	setStage,
	nextStage,
	prevStage,
	currentStageIndex,
})
</script>

<style scoped>
progress {
	@apply bg-surface-3;
	background-color: var(--surface-3, rgb(30, 30, 30));
}

progress::-webkit-progress-bar {
	@apply bg-surface-3;
}

progress::-webkit-progress-value {
	@apply bg-contrast;
}

progress::-moz-progress-bar {
	@apply bg-contrast;
}

.scrollbar-hide {
	-ms-overflow-style: none;
	scrollbar-width: none;
}

.scrollbar-hide::-webkit-scrollbar {
	display: none;
}
</style>
