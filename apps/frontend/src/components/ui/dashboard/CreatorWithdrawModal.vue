<template>
	<NewModal ref="withdrawModal" :noblur="true" :closable="withdrawContext.currentStage.value !== 'completion'"
		@on-hide="onModalHide">
		<template #title>
			<div v-if="shouldShowTitle" class="flex items-center gap-1 font-bold text-secondary">
				<template v-if="withdrawContext.currentStage.value === 'tax-form'">
					<span class="text-contrast">{{ formatMessage(stageLabels['tax-form']) }}</span>
				</template>
				<template v-else-if="withdrawContext.currentStage.value === 'method-selection'">
					<span class="text-contrast">{{ formatMessage(stageLabels['method-selection']) }}</span>
				</template>
				<template v-else-if="isDetailsStage">
					<button class="bg-transparent p-0 font-bold text-secondary active:scale-95"
						@click="goToBreadcrumbStage('method-selection')">
						{{ formatMessage(stageLabels['method-selection']) }}
					</button>
					<ChevronRightIcon class="h-5 w-5 text-secondary" stroke-width="3" />
					<span class="text-contrast">{{ formatMessage(messages.detailsLabel) }}</span>
				</template>
			</div>
		</template>
		<div class="min-w-[496px] max-w-[496px]">
			<TaxFormStage v-if="withdrawContext.currentStage.value === 'tax-form'" :balance="balance"
				:on-show-tax-form="showTaxFormModal" />
			<MethodSelectionStage v-else-if="withdrawContext.currentStage.value === 'method-selection'" />
			<TremendousDetailsStage v-else-if="withdrawContext.currentStage.value === 'tremendous-details'" />
			<MuralpayKycStage v-else-if="withdrawContext.currentStage.value === 'muralpay-kyc'" />
			<MuralpayDetailsStage v-else-if="withdrawContext.currentStage.value === 'muralpay-details'" />
			<CompletionStage v-else-if="withdrawContext.currentStage.value === 'completion'" />
			<div v-else>Something went wrong</div>
		</div>
		<div class="mt-4 flex justify-between gap-2">
			<ButtonStyled>
				<button v-if="withdrawContext.previousStep.value"
					@click="withdrawContext.setStage(withdrawContext.previousStep.value, true)">
					<LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
				</button>
				<button v-else @click="withdrawModal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button :disabled="!withdrawContext.canProceed.value"
					@click="withdrawContext.setStage(withdrawContext.nextStep.value)">
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
	<CreatorTaxFormModal ref="taxFormModal" @success="onTaxFormSuccess" @cancelled="onTaxFormCancelled" />
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	ChevronRightIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, NewModal } from '@modrinth/ui'
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
	(e: 'refresh-data'): void
	(e: 'hide'): void
}>()

const { formatMessage } = useVIntl()

const withdrawContext = createWithdrawContext(props.balance, props.userPayoutData)
provideWithdrawContext(withdrawContext)

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

function showTaxFormModal(e?: MouseEvent) {
	withdrawModal.value?.hide()
	taxFormModal.value?.startTaxForm(e)
}

function onTaxFormSuccess() {
	emit('refresh-data')
	nextTick(() => {
		withdrawContext.setStage('method-selection')
		show()
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
		return
	}

	// Determine initial stage based on balance and tax form status
	const b = props.balance
	if (!b || b.available <= 0) {
		withdrawContext.setStage('tax-form', true)
		withdrawModal.value?.show()
		return
	}

	const usedLimit = b.withdrawn_ytd ?? 0
	const remainingLimit = Math.max(0, 600 - usedLimit)
	const needsTaxForm = b.form_completion_status !== 'complete' && remainingLimit <= 0

	if (needsTaxForm) {
		withdrawContext.setStage('tax-form', true)
	} else {
		withdrawContext.setStage('method-selection', true)
	}

	withdrawModal.value?.show()
}

defineExpose({
	show,
})

const messages = defineMessages({
	// Stage labels for breadcrumb navigation
	taxFormStage: {
		id: 'dashboard.creator-withdraw-modal.stage.tax-form',
		defaultMessage: 'Tax Form',
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
})
</script>
