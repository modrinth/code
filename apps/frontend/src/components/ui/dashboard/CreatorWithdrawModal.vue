<template>
	<NewModal
		ref="withdrawModal"
		:closable="currentStage !== 'completion'"
		:hide-header="currentStage === 'completion'"
		:merge-header="currentStage === 'completion'"
		@on-hide="onModalHide"
	>
		<template #title>
			<div v-if="shouldShowTitle" class="flex items-center gap-1 text-secondary">
				<template v-if="currentStage === 'tax-form'">
					<span class="text-xl font-bold text-contrast">{{
						formatMessage(messages.taxFormStage)
					}}</span>
				</template>
				<template v-else-if="currentStage === 'method-selection'">
					<span class="text-xl font-bold text-contrast">{{
						formatMessage(messages.methodSelectionStage)
					}}</span>
					<ChevronRightIcon class="size-5 text-secondary" stroke-width="3" />
					<span class="text-xl text-secondary">{{ formatMessage(messages.detailsLabel) }}</span>
				</template>
				<template v-else-if="isDetailsStage">
					<button
						class="active:scale-9 bg-transparent p-0 text-xl text-secondary transition-colors duration-200 hover:text-primary"
						@click="goToBreadcrumbStage('method-selection')"
					>
						{{ formatMessage(messages.methodSelectionStage) }}
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
					v-if="currentStage === 'tax-form'"
					:balance="balance"
					:on-show-tax-form="showTaxFormModal"
				/>
				<MethodSelectionStage
					v-else-if="currentStage === 'method-selection'"
					:on-show-tax-form="showTaxFormModal"
					@close-modal="withdrawModal?.hide()"
				/>
				<TremendousDetailsStage v-else-if="currentStage === 'tremendous-details'" />
				<MuralpayKycStage v-else-if="currentStage === 'muralpay-kyc'" />
				<MuralpayDetailsStage v-else-if="currentStage === 'muralpay-details'" />
				<CompletionStage
					v-else-if="currentStage === 'completion'"
					@close="withdrawModal?.hide()"
					@view-transactions="handleViewTransactions"
				/>
				<div v-else>Something went wrong</div>
			</div>

			<div
				v-show="showBottomFade"
				class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-10 bg-gradient-to-t from-bg-raised to-transparent transition-all duration-300"
			/>
		</div>
		<div v-if="currentStage !== 'completion'" class="mt-4 flex justify-end gap-2">
			<ButtonStyled type="outlined">
				<button v-if="previousStep" class="!border-surface-5" @click="setStage(previousStep, true)">
					<LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
				</button>
				<button v-else class="!border-surface-5" @click="withdrawModal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled
				:color="
					currentStage === 'tax-form' && needsTaxForm && remainingLimit <= 0
						? 'orange'
						: currentStage === 'muralpay-details' || currentStage === 'tremendous-details'
							? 'brand'
							: 'standard'
				"
			>
				<button
					v-if="currentStage === 'tax-form' && needsTaxForm && remainingLimit > 0"
					@click="continueWithLimit"
				>
					{{ formatMessage(messages.continueWithLimit) }}
					<RightArrowIcon />
				</button>
				<button v-else-if="currentStage === 'tax-form' && needsTaxForm" @click="showTaxFormModal">
					<FileTextIcon />
					{{ formatMessage(messages.completeTaxForm) }}
				</button>
				<button
					v-else-if="currentStage === 'muralpay-details' || currentStage === 'tremendous-details'"
					:disabled="!canProceed || isSubmitting"
					@click="handleWithdraw"
				>
					<ArrowLeftRightIcon />
					{{ formatMessage(messages.withdrawButton) }}
				</button>
				<button v-else :disabled="!canProceed" @click="setStage(nextStep)">
					{{ formatMessage(commonMessages.nextButton) }}
					<RightArrowIcon />
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
	ArrowLeftRightIcon,
	ChevronRightIcon,
	FileTextIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	injectNotificationManager,
	NewModal,
	useScrollIndicator,
} from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
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
const { addNotification } = injectNotificationManager()

const withdrawContext = createWithdrawContext(props.balance)
provideWithdrawContext(withdrawContext)

const {
	currentStage,
	previousStep,
	nextStep,
	canProceed,
	setStage,
	withdrawData,
	resetData,
	stages,
	submitWithdrawal,
} = withdrawContext

const needsTaxForm = computed(() => {
	if (!props.balance || currentStage.value !== 'tax-form') return false
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
	withdrawData.value.tax.skipped = true
	setStage(nextStep.value)
}

const isSubmitting = ref(false)
async function handleWithdraw() {
	if (isSubmitting.value) return

	try {
		isSubmitting.value = true
		await submitWithdrawal()
		setStage('completion')
	} catch (error) {
		console.error('Withdrawal failed:', error)
		addNotification({
			title: 'Unable to withdraw',
			text: 'We were unable to submit your withdrawal request, please check your details or contact support.',
			type: 'error',
		})
	} finally {
		isSubmitting.value = false
	}
}

const shouldShowTitle = computed(() => {
	return currentStage.value !== 'completion'
})

const isDetailsStage = computed(() => {
	const detailsStages: WithdrawStage[] = ['tremendous-details', 'muralpay-kyc', 'muralpay-details']
	const current = currentStage.value
	return current ? detailsStages.includes(current) : false
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
	resetData()
	emit('hide')
}

function goToBreadcrumbStage(stage: WithdrawStage) {
	setStage(stage, true)
}

function handleViewTransactions() {
	withdrawModal.value?.hide()
	navigateTo('/dashboard/revenue/transfers')
}

function show(preferred?: WithdrawStage) {
	if (preferred) {
		setStage(preferred, true)
		withdrawModal.value?.show()
		checkScrollState()
		return
	}

	const firstStage = stages.value[0]
	if (firstStage) {
		setStage(firstStage, true)
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
	withdrawButton: {
		id: 'dashboard.creator-withdraw-modal.withdraw-button',
		defaultMessage: 'Withdraw',
	},
})
</script>
