<template>
	<NewModal
		ref="withdrawModal"
		:closable="currentStage !== 'completion'"
		:hide-header="currentStage === 'completion'"
		:merge-header="currentStage === 'completion'"
		:scrollable="true"
		max-content-height="72vh"
		:on-hide="onModalHide"
	>
		<template #title>
			<div v-if="shouldShowTitle" class="flex flex-wrap items-center gap-1 text-secondary">
				<template v-if="currentStage === 'tax-form'">
					<span class="text-lg font-bold text-contrast sm:text-xl">{{
						formatMessage(messages.taxFormStage)
					}}</span>
				</template>
				<template v-else-if="currentStage === 'method-selection'">
					<span class="text-lg font-bold text-contrast sm:text-xl">{{
						formatMessage(messages.methodSelectionStage)
					}}</span>
					<ChevronRightIcon class="size-5 text-secondary" stroke-width="3" />
					<span class="text-lg text-secondary sm:text-xl">{{
						formatMessage(messages.detailsLabel)
					}}</span>
				</template>
				<template v-else-if="isDetailsStage">
					<button
						class="active:scale-9 bg-transparent p-0 text-lg text-secondary transition-colors duration-200 hover:text-primary sm:text-xl"
						@click="goToBreadcrumbStage('method-selection')"
					>
						{{ formatMessage(messages.methodSelectionStage) }}
					</button>
					<ChevronRightIcon class="size-5 text-secondary" stroke-width="3" />
					<span class="text-lg font-bold text-contrast sm:text-xl">{{
						formatMessage(messages.detailsLabel)
					}}</span>
				</template>
			</div>
		</template>
		<div class="mx-auto w-full max-w-[496px] sm:mx-0 sm:min-w-[496px]">
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
			<LegacyPaypalDetailsStage v-else-if="currentStage === 'paypal-details'" />
			<CompletionStage v-else-if="currentStage === 'completion'" />
			<div v-else>Something went wrong</div>
		</div>
		<template #actions>
			<div v-if="currentStage === 'completion'" class="mt-4 flex w-full gap-3">
				<ButtonStyled class="flex-1">
					<button class="w-full text-contrast" @click="handleClose">
						{{ formatMessage(messages.closeButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled class="flex-1">
					<button class="w-full text-contrast" @click="handleViewTransactions">
						{{ formatMessage(messages.transactionsButton) }}
					</button>
				</ButtonStyled>
			</div>
			<div v-else class="mt-4 flex flex-col justify-end gap-2 sm:flex-row">
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
				<ButtonStyled :color="rightButtonConfig.color">
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
				</ButtonStyled>
			</div>
		</template>
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
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, nextTick, onMounted, ref, useTemplateRef, watch } from 'vue'

import {
	createWithdrawContext,
	type PayoutMethod,
	provideWithdrawContext,
	TAX_THRESHOLD_ACTUAL,
	type WithdrawStage,
} from '@/providers/creator-withdraw.ts'

import CreatorTaxFormModal from './CreatorTaxFormModal.vue'
import CompletionStage from './withdraw-stages/CompletionStage.vue'
import LegacyPaypalDetailsStage from './withdraw-stages/LegacyPaypalDetailsStage.vue'
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
	preloadedPaymentData?: { country: string; methods: PayoutMethod[] } | null
}>()

const emit = defineEmits<{
	(e: 'refresh-data' | 'hide'): void
}>()

const withdrawModal = useTemplateRef<InstanceType<typeof NewModal>>('withdrawModal')
const taxFormModal = ref<InstanceType<typeof CreatorTaxFormModal> | null>(null)
const isSubmitting = ref(false)

function show(preferred?: WithdrawStage) {
	if (preferred) {
		setStage(preferred, true)
		withdrawModal.value?.show()
		return
	}

	const firstStage = stages.value[0]
	if (firstStage) {
		setStage(firstStage, true)
	}

	withdrawModal.value?.show()
}

defineExpose({
	show,
})

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const withdrawContext = createWithdrawContext(
	props.balance,
	props.preloadedPaymentData || undefined,
)
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
	restoreStateFromStorage,
	clearSavedState,
} = withdrawContext

watch(
	() => props.balance,
	(newBalance) => {
		if (newBalance) {
			withdrawContext.balance.value = newBalance
		}
	},
	{ deep: true },
)

onMounted(() => {
	const route = useRoute()
	const router = useRouter()

	if (route.query.paypal_auth_return === 'true') {
		const savedState = restoreStateFromStorage()

		if (savedState?.data) {
			withdrawData.value = { ...savedState.data }

			nextTick(() => {
				show(savedState.stage)
			})

			clearSavedState()
		}

		const query = { ...route.query }
		delete query.paypal_auth_return
		router.replace({ query })
	}
})

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
	const raw = TAX_THRESHOLD_ACTUAL - ytd
	if (raw <= 0) return 0
	const cents = Math.floor(raw * 100)
	return cents / 100
})

const leftButtonConfig = computed(() => {
	if (previousStep.value) {
		return {
			icon: LeftArrowIcon,
			label: formatMessage(commonMessages.backButton),
			handler: () => setStage(previousStep.value, true),
			disabled: isSubmitting.value,
		}
	}
	return {
		icon: XIcon,
		label: formatMessage(commonMessages.cancelButton),
		handler: () => withdrawModal.value?.hide(),
		disabled: isSubmitting.value,
	}
})

const rightButtonConfig = computed(() => {
	const stage = currentStage.value
	const isTaxFormStage = stage === 'tax-form'
	const isDetailsStage =
		stage === 'muralpay-details' || stage === 'tremendous-details' || stage === 'paypal-details'

	if (isTaxFormStage && needsTaxForm.value && remainingLimit.value > 0) {
		return {
			icon: RightArrowIcon,
			label: formatMessage(messages.continueWithLimit),
			handler: continueWithLimit,
			disabled: false,
			color: 'standard' as const,
			iconPosition: 'after' as const,
		}
	}

	if (isTaxFormStage && needsTaxForm.value) {
		return {
			icon: FileTextIcon,
			label: formatMessage(messages.completeTaxForm),
			handler: showTaxFormModal,
			disabled: false,
			color: 'orange' as const,
			iconPosition: 'before' as const,
		}
	}

	if (isDetailsStage) {
		return {
			icon: isSubmitting.value ? SpinnerIcon : ArrowLeftRightIcon,
			iconClass: isSubmitting.value ? 'animate-spin' : undefined,
			label: formatMessage(messages.withdrawButton),
			handler: handleWithdraw,
			disabled: !canProceed.value || isSubmitting.value,
			color: 'brand' as const,
			iconPosition: 'before' as const,
		}
	}

	return {
		icon: RightArrowIcon,
		label: formatMessage(commonMessages.nextButton),
		handler: () => setStage(nextStep.value),
		disabled: !canProceed.value,
		color: 'standard' as const,
		iconPosition: 'after' as const,
	}
})

function continueWithLimit() {
	withdrawData.value.tax.skipped = true
	setStage(nextStep.value)
}

// TODO: God we need better errors from the backend (e.g error ids), this shit is insane
function getWithdrawalError(error: any): { title: string; text: string } {
	const description = error?.data?.description?.toLowerCase() || ''

	// Tax form error
	if (description.includes('tax form')) {
		return {
			title: formatMessage(messages.errorTaxFormTitle),
			text: formatMessage(messages.errorTaxFormText),
		}
	}

	// Invalid crypto wallet address
	if (
		(description.includes('wallet') && description.includes('invalid')) ||
		description.includes('wallet_address') ||
		(description.includes('blockchain') && description.includes('invalid'))
	) {
		return {
			title: formatMessage(messages.errorInvalidWalletTitle),
			text: formatMessage(messages.errorInvalidWalletText),
		}
	}

	// Invalid bank details
	if (
		(description.includes('bank') || description.includes('account')) &&
		(description.includes('invalid') || description.includes('failed'))
	) {
		return {
			title: formatMessage(messages.errorInvalidBankTitle),
			text: formatMessage(messages.errorInvalidBankText),
		}
	}

	// Invalid/fraudulent address
	if (
		description.includes('address') &&
		(description.includes('invalid') ||
			description.includes('verification') ||
			description.includes('fraudulent'))
	) {
		return {
			title: formatMessage(messages.errorInvalidAddressTitle),
			text: formatMessage(messages.errorInvalidAddressText),
		}
	}

	// Minimum amount not met
	if (
		description.includes('payoutminimumnotmeterror') ||
		description.includes('minimum') ||
		(description.includes('amount') && description.includes('less'))
	) {
		return {
			title: formatMessage(messages.errorMinimumNotMetTitle),
			text: formatMessage(messages.errorMinimumNotMetText),
		}
	}

	// Generic fallback
	return {
		title: formatMessage(messages.errorGenericTitle),
		text: formatMessage(messages.errorGenericText),
	}
}

async function handleWithdraw() {
	if (isSubmitting.value) return

	try {
		isSubmitting.value = true
		await submitWithdrawal()
		setStage('completion')
	} catch (error) {
		console.error('Withdrawal failed:', error)

		const { title, text } = getWithdrawalError(error)
		addNotification({
			title,
			text,
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
	const detailsStages: WithdrawStage[] = [
		'tremendous-details',
		'muralpay-kyc',
		'muralpay-details',
		'paypal-details',
	]
	const current = currentStage.value
	return current ? detailsStages.includes(current) : false
})

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

function handleClose() {
	withdrawModal.value?.hide()
	emit('refresh-data')
}

function handleViewTransactions() {
	withdrawModal.value?.hide()
	navigateTo('/dashboard/revenue/transfers')
}

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
	closeButton: {
		id: 'dashboard.withdraw.completion.close-button',
		defaultMessage: 'Close',
	},
	transactionsButton: {
		id: 'dashboard.withdraw.completion.transactions-button',
		defaultMessage: 'Transactions',
	},
	errorTaxFormTitle: {
		id: 'dashboard.withdraw.error.tax-form.title',
		defaultMessage: 'Please complete tax form',
	},
	errorTaxFormText: {
		id: 'dashboard.withdraw.error.tax-form.text',
		defaultMessage: 'You must complete a tax form to submit your withdrawal request.',
	},
	errorInvalidWalletTitle: {
		id: 'dashboard.withdraw.error.invalid-wallet.title',
		defaultMessage: 'Invalid wallet address',
	},
	errorInvalidWalletText: {
		id: 'dashboard.withdraw.error.invalid-wallet.text',
		defaultMessage:
			'The crypto wallet address you provided is invalid. Please double-check and try again.',
	},
	errorInvalidBankTitle: {
		id: 'dashboard.withdraw.error.invalid-bank.title',
		defaultMessage: 'Invalid bank details',
	},
	errorInvalidBankText: {
		id: 'dashboard.withdraw.error.invalid-bank.text',
		defaultMessage:
			'The bank account details you provided are invalid. Please verify your information.',
	},
	errorInvalidAddressTitle: {
		id: 'dashboard.withdraw.error.invalid-address.title',
		defaultMessage: 'Address verification failed',
	},
	errorInvalidAddressText: {
		id: 'dashboard.withdraw.error.invalid-address.text',
		defaultMessage:
			'The address you provided could not be verified. Please check your address details.',
	},
	errorMinimumNotMetTitle: {
		id: 'dashboard.withdraw.error.minimum-not-met.title',
		defaultMessage: 'Amount too low',
	},
	errorMinimumNotMetText: {
		id: 'dashboard.withdraw.error.minimum-not-met.text',
		defaultMessage:
			"The withdrawal amount (after fees) doesn't meet the minimum requirement. Please increase your withdrawal amount.",
	},
	errorGenericTitle: {
		id: 'dashboard.withdraw.error.generic.title',
		defaultMessage: 'Unable to withdraw',
	},
	errorGenericText: {
		id: 'dashboard.withdraw.error.generic.text',
		defaultMessage:
			'We were unable to submit your withdrawal request, please check your details or contact support.',
	},
})
</script>
