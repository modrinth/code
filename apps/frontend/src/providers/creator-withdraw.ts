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
export type PaymentMethod = 'gift-card' | 'paypal' | 'venmo' | 'bank-transfer' | 'crypto'

// TODO: need backend backend
export interface WithdrawData {
	selectedCountry: { id: string; name: string } | null
	selectedProvider: PaymentProvider | null
	selectedMethod: PaymentMethod | null
	amount: number
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

	setStage: (stage: WithdrawStage | undefined, skipValidation?: boolean) => Promise<void>
	validateCurrentStage: () => boolean
	resetData: () => void
}

export const [injectWithdrawContext, provideWithdrawContext] =
	createContext<WithdrawContextValue>('CreatorWithdrawModal')

export function useWithdrawContext() {
	return injectWithdrawContext()
}

export function createWithdrawContext(balance: any, userPayoutData: any): WithdrawContextValue {
	const currentStage = ref<WithdrawStage | undefined>()

	const withdrawData = ref<WithdrawData>({
		selectedCountry: null,
		selectedProvider: null,
		selectedMethod: null,
		amount: 0,
	})

	const stages = computed<WithdrawStage[]>(() => {
		const dynamicStages: WithdrawStage[] = []

		const usedLimit = balance?.withdrawn_ytd ?? 0
		const remainingLimit = Math.max(0, 600 - usedLimit)
		const needsTaxForm = balance?.form_completion_status !== 'complete' && remainingLimit <= 0

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
				// Tax form validation is handled by the existing tax form component
				return true
			case 'method-selection':
				return !!(
					withdrawData.value.selectedCountry &&
					withdrawData.value.selectedProvider &&
					withdrawData.value.selectedMethod
				)
			case 'tremendous-details':
				if (withdrawData.value.selectedMethod === 'gift-card') {
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
						kycData.dateOfBirth?.day &&
						kycData.dateOfBirth?.month &&
						kycData.dateOfBirth?.year &&
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
				return !!(
					withdrawData.value.accountDetails?.bankAccount ||
					withdrawData.value.accountDetails?.cryptoWallet
				)
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
		setStage,
		validateCurrentStage,
		resetData,
	}
}
