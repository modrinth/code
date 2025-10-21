import { createContext, useDebugLogger } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref } from 'vue'

import { getRailConfig } from '@/utils/muralpay-rails'

export type WithdrawStage =
	| 'tax-form'
	| 'method-selection'
	| 'tremendous-details'
	| 'muralpay-kyc'
	| 'muralpay-details'
	| 'completion'

export type PaymentProvider = 'tremendous' | 'muralpay'

/**
 * only used for the withdraw modal stage logic - not actually for API requests
 **/
export type PaymentMethod = 'gift_card' | 'paypal' | 'venmo' | 'bank' | 'crypto'

export interface WithdrawData {
	selectedCountry: { id: string; name: string } | null
	selectedProvider: PaymentProvider | null
	selectedMethod: string | null
	selectedMethodId: string | null
	amount: number
	skippedTaxForm: boolean
	giftCardDetails?: any
	kycData?: any
	accountDetails?: any
}

export interface WithdrawContextValue {
	currentStage: Ref<WithdrawStage | undefined>
	stages: ComputedRef<WithdrawStage[]>

	canProceed: ComputedRef<boolean>
	nextStep: ComputedRef<WithdrawStage | undefined>
	previousStep: ComputedRef<WithdrawStage | undefined>
	currentStepIndex: ComputedRef<number>

	withdrawData: Ref<WithdrawData>
	balance: Ref<any>
	maxWithdrawAmount: ComputedRef<number>

	setStage: (stage: WithdrawStage | undefined, skipValidation?: boolean) => Promise<void>
	validateCurrentStage: () => boolean
	resetData: () => void
}

export const [injectWithdrawContext, provideWithdrawContext] =
	createContext<WithdrawContextValue>('CreatorWithdrawModal')

export function useWithdrawContext() {
	return injectWithdrawContext()
}

export function createWithdrawContext(balance: any): WithdrawContextValue {
	const debug = useDebugLogger('CreatorWithdraw')
	const currentStage = ref<WithdrawStage | undefined>()

	const withdrawData = ref<WithdrawData>({
		selectedCountry: null,
		selectedProvider: null,
		selectedMethod: null,
		selectedMethodId: null,
		amount: 0,
		skippedTaxForm: false,
	})

	const balanceRef = ref(balance)

	const stages = computed<WithdrawStage[]>(() => {
		const dynamicStages: WithdrawStage[] = []

		const usedLimit = balance?.withdrawn_ytd ?? 0
		const available = balance?.available ?? 0

		const needsTaxForm =
			balance?.form_completion_status !== 'complete' && usedLimit + available >= 600

		debug('Tax form check:', {
			usedLimit,
			available,
			total: usedLimit + available,
			status: balance?.form_completion_status,
			needsTaxForm,
		})

		if (needsTaxForm) {
			dynamicStages.push('tax-form')
		}

		dynamicStages.push('method-selection')

		const selectedProvider = withdrawData.value.selectedProvider
		if (selectedProvider === 'tremendous') {
			dynamicStages.push('tremendous-details')
		} else if (selectedProvider === 'muralpay') {
			dynamicStages.push('muralpay-kyc')
			dynamicStages.push('muralpay-details')
		}

		dynamicStages.push('completion')

		return dynamicStages
	})

	const maxWithdrawAmount = computed(() => {
		const availableBalance = balance?.available ?? 0
		const formCompleted = balance?.form_completion_status === 'complete'

		if (formCompleted) {
			return availableBalance
		}

		if (!withdrawData.value.skippedTaxForm) {
			return availableBalance
		}

		const usedLimit = balance?.withdrawn_ytd ?? 0
		const remainingLimit = Math.max(0, 600 - usedLimit)
		return Math.min(remainingLimit, availableBalance)
	})

	const currentStepIndex = computed(() =>
		currentStage.value ? stages.value.indexOf(currentStage.value) : -1,
	)

	const nextStep = computed(() => {
		if (!currentStage.value) return undefined
		const currentIndex = currentStepIndex.value
		if (currentIndex === -1 || currentIndex >= stages.value.length - 1) return undefined
		return stages.value[currentIndex + 1]
	})

	const previousStep = computed(() => {
		if (!currentStage.value) return undefined
		const currentIndex = currentStepIndex.value
		if (currentIndex <= 0) return undefined
		return stages.value[currentIndex - 1]
	})

	const canProceed = computed(() => {
		return validateCurrentStage()
	})

	function validateCurrentStage(): boolean {
		switch (currentStage.value) {
			case 'tax-form': {
				// If no balance data, allow proceeding
				if (!balanceRef.value) return true
				const ytd = balanceRef.value.withdrawn_ytd ?? 0
				const remainingLimit = Math.max(0, 600 - ytd)
				const form_completion_status = balanceRef.value.form_completion_status
				// If they haven't hit $600 yet, they can proceed without completing the form
				if (ytd < 600) return true
				// If user skipped tax form to proceed with limited withdrawal
				if (withdrawData.value.skippedTaxForm && remainingLimit > 0) return true
				// If they hit $600, they must complete the form to proceed
				return form_completion_status === 'complete'
			}
			case 'method-selection':
				return !!(
					withdrawData.value.selectedCountry &&
					withdrawData.value.selectedProvider &&
					withdrawData.value.selectedMethod &&
					(withdrawData.value.selectedMethod === 'gift_cards' ||
						withdrawData.value.selectedMethodId)
				)
			case 'tremendous-details':
				if (withdrawData.value.selectedMethod === 'gift_cards') {
					return !!(withdrawData.value.selectedMethodId && withdrawData.value.amount > 0)
				}
				return !!(withdrawData.value.amount > 0) // for paypal/venmo, only need amount
			case 'muralpay-kyc': {
				if (!withdrawData.value.kycData) return false

				const kycData = withdrawData.value.kycData
				const hasValidAddress = !!(
					kycData.physicalAddress?.address1 &&
					kycData.physicalAddress?.city &&
					kycData.physicalAddress?.state &&
					kycData.physicalAddress?.country &&
					kycData.physicalAddress?.zip
				)

				if (kycData.type === 'individual') {
					return !!(
						kycData.firstName &&
						kycData.lastName &&
						kycData.email &&
						kycData.dateOfBirth &&
						hasValidAddress
					)
				} else if (kycData.type === 'business') {
					return !!(kycData.name && kycData.email && hasValidAddress)
				}

				return false
			}
			case 'muralpay-details': {
				const railId = withdrawData.value.selectedMethod
				const rail = getRailConfig(railId as string)
				if (!rail) return false

				if (!withdrawData.value.amount || withdrawData.value.amount <= 0) return false

				const accountDetails = withdrawData.value.accountDetails
				if (!accountDetails) return false

				if (rail.requiresBankName && !accountDetails.bankName) return false

				const requiredFields = rail.fields.filter((f) => f.required)
				const allRequiredPresent = requiredFields.every((f) => {
					const value = accountDetails[f.name]
					return value !== undefined && value !== null && value !== ''
				})

				return allRequiredPresent
			}
			case 'completion':
				return true
			default:
				return false
		}
	}

	async function setStage(stage: WithdrawStage | undefined, skipValidation = false) {
		if (!skipValidation && !canProceed.value) {
			return
		}

		if (!stage) {
			// TBD: Handle final withdraw submission
			debug('Withdraw process completed!', withdrawData.value)
			return
		}

		currentStage.value = stage
	}

	function resetData() {
		withdrawData.value = {
			selectedCountry: null,
			selectedProvider: null,
			selectedMethod: null,
			selectedMethodId: null,
			amount: 0,
			skippedTaxForm: false,
		}
		currentStage.value = undefined
	}

	return {
		currentStage,
		stages,
		canProceed,
		nextStep,
		previousStep,
		currentStepIndex,
		withdrawData,
		balance: balanceRef,
		maxWithdrawAmount,
		setStage,
		validateCurrentStage,
		resetData,
	}
}
