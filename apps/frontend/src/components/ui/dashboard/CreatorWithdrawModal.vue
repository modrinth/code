<template>
	<NewModal
		ref="withdrawModal"
		:closable="withdrawContext.currentStage.value !== 'completion'"
		@on-hide="onModalHide"
	>
		<template #title>
			<div v-if="shouldShowTitle" class="flex items-center gap-1 text-secondary">
				<template v-if="withdrawContext.currentStage.value === 'tax-form'">
					<span class="text-xl font-bold text-contrast">{{
						formatMessage(stageLabels['tax-form'])
					}}</span>
				</template>
				<template v-else-if="withdrawContext.currentStage.value === 'method-selection'">
					<span class="text-xl font-bold text-contrast">{{
						formatMessage(stageLabels['method-selection'])
					}}</span>
					<ChevronRightIcon class="size-5 text-secondary" stroke-width="3" />
					<span class="text-xl text-secondary">{{ formatMessage(messages.detailsLabel) }}</span>
				</template>
				<template v-else-if="isDetailsStage">
					<button
						class="active:scale-9 bg-transparent p-0 text-xl text-secondary"
						@click="goToBreadcrumbStage('method-selection')"
					>
						{{ formatMessage(stageLabels['method-selection']) }}
					</button>
					<ChevronRightIcon class="size-5 text-secondary" stroke-width="3" />
					<span class="text-xl font-bold text-contrast">{{
						formatMessage(messages.detailsLabel)
					}}</span>
				</template>
			</div>
		</template>
		<div class="relative min-w-[496px] max-w-[496px]">
			<div
				v-show="showTopFade"
				class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-10 bg-gradient-to-b from-bg-raised to-transparent transition-all duration-300"
			/>

			<div
				ref="scrollContainer"
				class="max-h-[70vh] overflow-y-auto px-1 pb-1"
				@scroll="checkScrollState"
			>
				<TaxFormStage
					v-if="withdrawContext.currentStage.value === 'tax-form'"
					:balance="balance"
					:on-show-tax-form="showTaxFormModal"
				/>
				<MethodSelectionStage
					v-else-if="withdrawContext.currentStage.value === 'method-selection'"
					:on-show-tax-form="showTaxFormModal"
				/>
				<TremendousDetailsStage
					v-else-if="withdrawContext.currentStage.value === 'tremendous-details'"
				/>
				<MuralpayKycStage v-else-if="withdrawContext.currentStage.value === 'muralpay-kyc'" />
				<MuralpayDetailsStage
					v-else-if="withdrawContext.currentStage.value === 'muralpay-details'"
				/>
				<CompletionStage v-else-if="withdrawContext.currentStage.value === 'completion'" />
				<div v-else>Something went wrong</div>
			</div>

			<div
				v-show="showBottomFade"
				class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-10 bg-gradient-to-t from-bg-raised to-transparent transition-all duration-300"
			/>
		</div>
		<div class="mt-4 flex justify-end gap-2">
			<ButtonStyled type="outlined">
				<button
					v-if="withdrawContext.previousStep.value"
					class="!border-surface-5"
					@click="withdrawContext.setStage(withdrawContext.previousStep.value, true)"
				>
					<LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
				</button>
				<button v-else class="!border-surface-5" @click="withdrawModal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled
				:color="
					withdrawContext.currentStage.value === 'tax-form' && needsTaxForm && remainingLimit <= 0
						? 'orange'
						: 'standard'
				"
			>
				<button
					v-if="
						withdrawContext.currentStage.value === 'tax-form' && needsTaxForm && remainingLimit > 0
					"
					@click="continueWithLimit"
				>
					{{ formatMessage(messages.continueWithLimit) }}
					<RightArrowIcon />
				</button>
				<button
					v-else-if="withdrawContext.currentStage.value === 'tax-form' && needsTaxForm"
					@click="showTaxFormModal"
				>
					<FileTextIcon />
					{{ formatMessage(messages.completeTaxForm) }}
				</button>
				<button
					v-else
					:disabled="!withdrawContext.canProceed.value"
					@click="withdrawContext.setStage(withdrawContext.nextStep.value)"
				>
					<template v-if="withdrawContext.currentStage.value === 'completion'">
						<CheckCircleIcon /> Complete
					</template>
					<template v-else>
						{{ formatMessage(commonMessages.nextButton) }}
						<RightArrowIcon />
					</template>
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
	<CreatorTaxFormModal
		ref="taxFormModal"
		close-button-text="Continue"
		@success="onTaxFormSuccess"
		@cancelled="onTaxFormCancelled"
	/>
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	ChevronRightIcon,
	FileTextIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, NewModal, useScrollIndicator } from '@modrinth/ui'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import {
	createWithdrawContext,
	provideWithdrawContext,
	type WithdrawStage,
} from '@/providers/creator-withdraw.ts'

import CreatorTaxFormModal from './CreatorTaxFormModal.vue'
import CompletionStage from './withdraw-stages/CompletionStage.vue'
import MethodSelectionStage from './withdraw-stages/MethodSelectionStage.vue'
import MuralpayDetailsStage from './withdraw-stages/MuralpayDetailsStage.vue'
import MuralpayKycStage from './withdraw-stages/MuralpayKycStage.vue'
import TaxFormStage from './withdraw-stages/TaxFormStage.vue'
import TremendousDetailsStage from './withdraw-stages/TremendousDetailsStage.vue'

type FormCompletionStatus = 'unknown' | 'unrequested' | 'unsigned' | 'tin-mismatch' | 'complete'

interface UserBalanceResponse {
	available: number
	withdrawn_lifetime: number
	withdrawn_ytd: number
	pending: number
	dates: Record<string, number>
	requested_form_type: string | null
	form_completion_status: FormCompletionStatus | null
}

const props = defineProps<{
	balance: UserBalanceResponse | null
}>()

const emit = defineEmits<{
	(e: 'refresh-data' | 'hide'): void
}>()

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()

const withdrawContext = createWithdrawContext(props.balance, undefined, flags.value.testTaxForm)
provideWithdrawContext(withdrawContext)

const needsTaxForm = computed(() => {
	if (!props.balance || withdrawContext.currentStage.value !== 'tax-form') return false
	const ytd = props.balance.withdrawn_ytd ?? 0
	const available = props.balance.available ?? 0
	const status = props.balance.form_completion_status
	return status !== 'complete' && ytd + available >= 600
})

const remainingLimit = computed(() => {
	if (!props.balance) return 0
	const ytd = props.balance.withdrawn_ytd ?? 0
	const raw = 600 - ytd
	if (raw <= 0) return 0
	const cents = Math.floor(raw * 100)
	return cents / 100
})

function continueWithLimit() {
	withdrawContext.withdrawData.value.skippedTaxForm = true
	withdrawContext.setStage(withdrawContext.nextStep.value)
}

const stageLabels = computed<Record<WithdrawStage, MessageDescriptor>>(() => ({
	'tax-form': messages.taxFormStage,
	'method-selection': messages.methodSelectionStage,
	'tremendous-details': messages.tremendousDetailsStage,
	'muralpay-kyc': messages.muralpayKycStage,
	'muralpay-details': messages.muralpayDetailsStage,
	completion: messages.completionStage,
}))

const shouldShowTitle = computed(() => {
	return withdrawContext.currentStage.value !== 'completion'
})

const isDetailsStage = computed(() => {
	const detailsStages: WithdrawStage[] = ['tremendous-details', 'muralpay-kyc', 'muralpay-details']
	const currentStage = withdrawContext.currentStage.value
	return currentStage ? detailsStages.includes(currentStage) : false
})

const withdrawModal = useTemplateRef<InstanceType<typeof NewModal>>('withdrawModal')
const taxFormModal = ref<InstanceType<typeof CreatorTaxFormModal> | null>(null)
const scrollContainer = ref<HTMLElement | null>(null)

const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollContainer)

function showTaxFormModal(e?: MouseEvent) {
	withdrawModal.value?.hide()
	taxFormModal.value?.startTaxForm(e ?? new MouseEvent('click'))
}

function onTaxFormSuccess() {
	emit('refresh-data')
	nextTick(() => {
		show('method-selection')
	})
}

function onTaxFormCancelled() {
	show('tax-form')
}

function onModalHide() {
	withdrawContext.resetData()
	emit('hide')
}

function goToBreadcrumbStage(stage: WithdrawStage) {
	withdrawContext.setStage(stage, true)
}

function show(preferred?: WithdrawStage) {
	if (preferred) {
		withdrawContext.setStage(preferred, true)
		withdrawModal.value?.show()
		checkScrollState()
		return
	}

	const firstStage = withdrawContext.stages.value[0]
	if (firstStage) {
		withdrawContext.setStage(firstStage, true)
	}

	withdrawModal.value?.show()
	checkScrollState()
}

defineExpose({
	show,
})

const messages = defineMessages({
	taxFormStage: {
		id: 'dashboard.creator-withdraw-modal.stage.tax-form',
		defaultMessage: 'Tax form',
	},
	methodSelectionStage: {
		id: 'dashboard.creator-withdraw-modal.stage.method-selection',
		defaultMessage: 'Method',
	},
	tremendousDetailsStage: {
		id: 'dashboard.creator-withdraw-modal.stage.tremendous-details',
		defaultMessage: 'Details',
	},
	muralpayKycStage: {
		id: 'dashboard.creator-withdraw-modal.stage.muralpay-kyc',
		defaultMessage: 'Verification',
	},
	muralpayDetailsStage: {
		id: 'dashboard.creator-withdraw-modal.stage.muralpay-details',
		defaultMessage: 'Account Details',
	},
	completionStage: {
		id: 'dashboard.creator-withdraw-modal.stage.completion',
		defaultMessage: 'Complete',
	},
	detailsLabel: {
		id: 'dashboard.creator-withdraw-modal.details-label',
		defaultMessage: 'Details',
	},
	completeTaxForm: {
		id: 'dashboard.creator-withdraw-modal.complete-tax-form',
		defaultMessage: 'Complete tax form',
	},
	continueWithLimit: {
		id: 'dashboard.creator-withdraw-modal.continue-with-limit',
		defaultMessage: 'Continue with limit',
	},
})
</script>
