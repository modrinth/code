<template>
	<NewModal
		ref="createProjectModal"
		:scrollable="true"
		max-content-height="72vh"
		:on-hide="onModalHide"
	>
		<!-- :closable="currentStage !== 'add-files'"
		:hide-header="currentStage === 'add-files'"
		:merge-header="currentStage === 'add-files'" -->
		<template #title>
			<div class="flex flex-wrap items-center gap-1 text-secondary">
				<span class="text-lg font-bold text-contrast sm:text-xl">Modal title</span>
			</div>
		</template>
		<div class="mx-auto w-full max-w-[496px] sm:mx-0 sm:min-w-[496px]">
			<template v-if="currentStage === 'add-files'">
				<AddFileStage></AddFileStage>
			</template>
		</div>
		<template #actions>
			<div class="mt-4 flex flex-col justify-end gap-2 sm:flex-row">
				<ButtonStyled type="outlined">
					<button
						class="!border-surface-5"
						:disabled="leftButtonConfig.disabled"
						@click="leftButtonConfig.handler"
					>
						<component :is="leftButtonConfig.icon" />
						{{ leftButtonConfig.label }}
					</button>
				</ButtonStyled>
				<!-- <ButtonStyled :color="rightButtonConfig.color">
					<button :disabled="rightButtonConfig.disabled" @click="rightButtonConfig.handler">
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
				</ButtonStyled> -->
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal } from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import AddFileStage from './stages/AddFileStage.vue'

const createProjectModal = useTemplateRef<InstanceType<typeof NewModal>>('createProjectModal')
const isSubmitting = ref(false)

type Stage = 'add-files' | 'add-details' | 'add-mc-versions' | 'add-changelog' | 'add-dependencies'

const currentStage = ref<Stage>('add-files')

function show(preferred?: Stage) {
	if (preferred) {
		// setStage(preferred, true)
		createProjectModal.value?.show()
		return
	}

	createProjectModal.value?.show()
}

defineExpose({
	show,
})

// const { formatMessage } = useVIntl()
// const { addNotification } = injectNotificationManager()

// const withdrawContext = createWithdrawContext(
// 	props.balance,
// 	props.preloadedPaymentData || undefined,
// )
// provideWithdrawContext(withdrawContext)

// const {
// 	currentStage,
// 	previousStep,
// 	nextStep,
// 	canProceed,
// 	setStage,
// 	withdrawData,
// 	resetData,
// 	stages,
// 	submitWithdrawal,
// 	restoreStateFromStorage,
// 	clearSavedState,
// } = withdrawContext

const leftButtonConfig = computed(() => {
	return {
		icon: XIcon,
		label: 'next',
		handler: () => createProjectModal.value?.hide(),
		disabled: isSubmitting.value,
	}
})

// const rightButtonConfig = computed(() => {
// 	return {
// 		icon: RightArrowIcon,
// 		label: 'Continue',
// 		handler: () => setStage(nextStep.value),
// 		disabled: !canProceed.value,
// 		color: 'standard' as const,
// 		iconPosition: 'after' as const,
// 	}
// })

const emit = defineEmits<{
	(e: 'refresh-data' | 'hide'): void
}>()

function onModalHide() {
	emit('hide')
}

// function handleClose() {
// 	createProjectModal.value?.hide()
// 	emit('refresh-data')
// }

// const messages = defineMessages({})
</script>
