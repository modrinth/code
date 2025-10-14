import { createContext } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref } from 'vue'

export type WithdrawStage =
	| 'tax-form'
	| 'method-selection'
	| 'tremendous-details'
	| 'muralpay-kyc'
	| 'muralpay-details'
	| 'completion'

export type PaymentProvider = 'tremendous' | 'muralpay'
export type PaymentMethod = 'gift_card' | 'paypal' | 'venmo' | 'bank' | 'crypto'

// TODO: need backend backend
export interface WithdrawData {
	selectedCountry: { id: string; name: string } | null
	selectedProvider: PaymentProvider | null
	selectedMethod: PaymentMethod | null
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
	showScrollFade: Ref<boolean>
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

export function createWithdrawContext(
	balance: any,
	userPayoutData?: any,
	testTaxForm = false,
): WithdrawContextValue {
	const currentStage = ref<WithdrawStage | undefined>()

	const withdrawData = ref<WithdrawData>({
		selectedCountry: null,
		selectedProvider: null,
		selectedMethod: null,
		amount: 0,
		skippedTaxForm: false,
	})

	const showScrollFade = ref(false)
	const balanceRef = ref(balance)

	const stages = computed<WithdrawStage[]>(() => {
		const dynamicStages: WithdrawStage[] = []

		const usedLimit = balance?.withdrawn_ytd ?? 0
		const available = balance?.available ?? 0

		const needsTaxForm = balance?.form_completion_status !== 'complete' && (usedLimit + available >= 600)

		console.log('Tax form check:', {
			usedLimit,
			available,
			total: usedLimit + available,
			status: balance?.form_completion_status,
			needsTaxForm
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

		// If tax form is completed, user can withdraw full balance
		if (formCompleted) {
			return availableBalance
		}

		// If user hasn't skipped tax form yet, they can withdraw full balance
		if (!withdrawData.value.skippedTaxForm) {
			return availableBalance
		}

		// If user skipped tax form, limit them to remaining YTD limit
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
			case 'tax-form':
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
			case 'method-selection':
				return !!(
					withdrawData.value.selectedCountry &&
					withdrawData.value.selectedProvider &&
					withdrawData.value.selectedMethod
				)
			case 'tremendous-details':
				if (withdrawData.value.selectedMethod === 'gift_card') {
					return !!(
						withdrawData.value.giftCardDetails?.type &&
						withdrawData.value.giftCardDetails?.amount > 0
					)
				}
				return true // For paypal/venmo, just need method selected
			case 'muralpay-kyc':
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
					return !!(
						kycData.name &&
						kycData.email &&
						hasValidAddress
					)
				}

				return false
			case 'muralpay-details':
				const accountDetails = withdrawData.value.accountDetails
				if (!accountDetails) return false

				// Validate amount (required for both methods)
				if (!withdrawData.value.amount || withdrawData.value.amount <= 0) return false

				// Validate bank transfer details
				if (withdrawData.value.selectedMethod === 'bank') {
					const bank = accountDetails.bankAccount
					return !!(
						bank &&
						bank.bankName &&
						bank.accountType &&
						bank.accountNumber &&
						bank.routingNumber
					)
				}

				// Validate crypto wallet details
				if (withdrawData.value.selectedMethod === 'crypto') {
					const crypto = accountDetails.cryptoWallet
					return !!(crypto && crypto.walletAddress)
				}

				return false
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

		// If completing the withdraw process (no more stages)
		if (!stage) {
			// TBD: Handle final withdraw submission
			console.log('Withdraw process completed!', withdrawData.value)
			return
		}

		currentStage.value = stage
	}

	function resetData() {
		withdrawData.value = {
			selectedCountry: null,
			selectedProvider: null,
			selectedMethod: null,
			amount: 0,
			skippedTaxForm: false,
		}
		currentStage.value = undefined
		showScrollFade.value = false
	}

	return {
		currentStage,
		stages,
		canProceed,
		nextStep,
		previousStep,
		currentStepIndex,
		withdrawData,
		showScrollFade,
		balance: balanceRef,
		maxWithdrawAmount,
		setStage,
		validateCurrentStage,
		resetData,
	}
}
