<template>
	<NewModal
		ref="modal"
		:scrollable="true"
		max-content-height="72vh"
		:on-hide="onModalHide"
		:closable="true"
	>
		<template #title>
			<div class="flex flex-wrap items-center gap-1 text-secondary">
				<span class="text-lg font-bold text-contrast sm:text-xl">{{ currentStage.title }}</span>
			</div>
		</template>

		<component :is="currentStage.stageContent" />

		<template #actions>
			<div class="mt-4 flex flex-col justify-end gap-2 sm:flex-row">
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

<script setup lang="ts">
import { ButtonStyled, NewModal } from '@modrinth/ui';
import type { Component } from 'vue';
import { computed, ref, useTemplateRef } from 'vue';

const props = defineProps<{
	stages: ModalStage[]
}>()

export interface ButtonConfig {
	icon?: Component | null
	label?: string
	onClick?: () => void
	disabled?: boolean
	color?: InstanceType<typeof ButtonStyled>['$props']['color']
	iconClass?: string | null
	iconPosition?: 'after' | 'before'
}

export interface ModalStage {
	title: string
	stageContent: Component
	leftButtonConfig: ButtonConfig | null
	rightButtonConfig: ButtonConfig | null
}

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const currentStageIndex = ref<number>(0)

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

const nextStage = () => {
	if (currentStageIndex.value === -1) return
	if (currentStageIndex.value >= props.stages.length - 1) return
	currentStageIndex.value = currentStageIndex.value + 1
}

const prevStage = () => {
	if (currentStageIndex.value <= 0) return
	currentStageIndex.value = currentStageIndex.value - 1
}

const currentStage = computed(() => props.stages[currentStageIndex.value])

const leftButtonConfig = computed(() => {
	console.log(currentStage.value?.leftButtonConfig)
	return currentStage.value?.leftButtonConfig
})

const rightButtonConfig = computed(() => {
	console.log(currentStage.value?.rightButtonConfig)
	return currentStage.value?.rightButtonConfig
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
	nextStage,
	prevStage,
})
</script>
