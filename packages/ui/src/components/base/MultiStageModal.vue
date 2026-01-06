<template>
	<NewModal
		ref="modal"
		:scrollable="true"
		max-content-height="72vh"
		:on-hide="onModalHide"
		:closable="true"
		:close-on-click-outside="false"
	>
		<template #title>
			<div class="grow w-min flex flex-wrap items-center gap-1 text-secondary">
				<template v-if="breadcrumbs && !resolveCtxFn(currentStage.nonProgressStage, context)">
					<template v-for="(stage, index) in breadcrumbStages" :key="stage.id">
						<div class="flex">
							<button
								class="bg-transparent active:scale-95 font-bold text-secondary p-0"
								:class="{
									'!text-contrast font-bold': resolveCtxFn(currentStage.id, context) === stage.id,
									'font-bold': resolveCtxFn(currentStage.id, context) !== stage.id,
								}"
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
				</template>
				<span v-else class="text-lg font-bold text-contrast sm:text-xl">{{ resolvedTitle }}</span>
			</div>
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
import { computed, ref, useTemplateRef } from 'vue'

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
	nonProgressStage?: MaybeCtxFn<T, boolean>
	hideBreadcrumbTitle?: MaybeCtxFn<T, boolean>
	leftButtonConfig: MaybeCtxFn<T, StageButtonConfig | null>
	rightButtonConfig: MaybeCtxFn<T, StageButtonConfig | null>
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

// Stages that are not skipped (visible in breadcrumbs)
const breadcrumbStages = computed(() => {
	return props.stages.filter((stage) => {
		const skip =
			resolveCtxFn(stage.skip, props.context) ||
			resolveCtxFn(stage.nonProgressStage, props.context) ||
			resolveCtxFn(stage.hideBreadcrumbTitle, props.context)
		return !skip
	})
})

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
</style>
