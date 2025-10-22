import {
	GiftIcon,
	HeartIcon,
	LandmarkIcon,
	PayPalIcon,
	PolygonIcon,
	VenmoIcon,
} from '@modrinth/assets'
import { createContext, paymentMethodMessages, useDebugLogger } from '@modrinth/ui'
import { type Component, computed, type ComputedRef, type Ref, ref } from 'vue'

import { getBlockchainIcon } from '@/utils/finance-icons'
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

export interface PayoutMethod {
	id: string
	type: string
	name: string
	category?: string
	image_url: string | null
	image_logo_url: string | null
	fee: {
		percentage: number
		min: number
		max: number | null
	}
	interval: {
		standard: {
			min: number
			max: number
		}
	}
	config?: {
		fiat?: string | null
		blockchain?: string[]
	}
}

export interface PaymentOption {
	value: string
	label: string | { id: string; defaultMessage: string }
	icon: Component
	methodId: string | undefined
	fee: string
	type: string
}

export interface WithdrawData {
	selectedCountry: { id: string; name: string } | null
	selectedProvider: PaymentProvider | null
	selectedMethod: string | null
	selectedMethodId: string | null
	amount: number
	skippedTaxForm: boolean
	deliveryEmail?: string | null
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
	availableMethods: Ref<PayoutMethod[]>
	paymentOptions: ComputedRef<PaymentOption[]>

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
	const availableMethods = ref<PayoutMethod[]>([])

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

	const paymentOptions = computed<PaymentOption[]>(() => {
		const methods = availableMethods.value
		if (!methods || methods.length === 0) {
			debug('No payment methods available')
			return []
		}

		debug('Available methods:', methods)

		const options: PaymentOption[] = []

		const tremendousMethods = methods.filter((m) => m.type === 'tremendous')

		const paypalMethods = tremendousMethods.filter((m) => m.category === 'paypal')
		if (paypalMethods.length > 0) {
			options.push({
				value: 'paypal',
				label: paymentMethodMessages.paypal,
				icon: PayPalIcon,
				methodId: paypalMethods[0].id,
				fee: '≈ 6%, max $25',
				type: 'tremendous',
			})
		}

		const venmoMethods = tremendousMethods.filter((m) => m.category === 'venmo')
		if (venmoMethods.length > 0) {
			options.push({
				value: 'venmo',
				label: paymentMethodMessages.venmo,
				icon: VenmoIcon,
				methodId: venmoMethods[0].id,
				fee: '≈ 6%, max $25',
				type: 'tremendous',
			})
		}

		const merchantMethods = tremendousMethods.filter(
			(m) => m.category === 'merchant_card' || m.category === 'merchant_cards',
		)
		if (merchantMethods.length > 0) {
			options.push({
				value: 'merchant_card',
				label: paymentMethodMessages.giftCard,
				icon: GiftIcon,
				methodId: undefined,
				fee: '≈ 0%',
				type: 'tremendous',
			})
		}

		const charityMethods = tremendousMethods.filter((m) => m.category === 'charity')
		if (charityMethods.length > 0) {
			options.push({
				value: 'charity',
				label: paymentMethodMessages.charity,
				icon: HeartIcon,
				methodId: undefined,
				fee: '≈ 0%',
				type: 'tremendous',
			})
		}

		const muralPayMethods = methods.filter((m) => m.type === 'muralpay')
		for (const method of muralPayMethods) {
			const methodId = method.id

			if (methodId.startsWith('fiat_')) {
				const railCode = methodId.replace('fiat_', '')
				const rail = getRailConfig(methodId)

				if (!rail) {
					debug('Warning: No rail config found for', methodId)
					continue
				}

				options.push({
					value: methodId,
					label: rail.name || `Bank transfer (${railCode.toUpperCase()})`,
					icon: LandmarkIcon,
					methodId: method.id,
					fee: rail.fee,
					type: 'fiat',
				})
			} else if (methodId.startsWith('blockchain_')) {
				const rail = getRailConfig(methodId)

				if (!rail) {
					debug('Warning: No rail config found for', methodId)
					continue
				}

				options.push({
					value: methodId,
					label: rail.name || method.name,
					icon: getBlockchainIcon(rail.blockchain || 'POLYGON') || PolygonIcon,
					methodId: method.id,
					fee: rail.fee,
					type: 'crypto',
				})
			}
		}

		const sortOrder: Record<string, number> = {
			fiat: 1,
			paypal: 2,
			venmo: 3,
			visa_card: 4,
			merchant_card: 5,
			charity: 6,
			crypto: 7,
		}
		options.sort((a, b) => {
			const aOrder = sortOrder[a.type] ?? sortOrder[a.value] ?? 999
			const bOrder = sortOrder[b.type] ?? sortOrder[b.value] ?? 999
			return aOrder - bOrder
		})

		debug('Payment options computed:', options)
		return options
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
					(withdrawData.value.selectedMethod === 'merchant_card' ||
						withdrawData.value.selectedMethod === 'charity' ||
						withdrawData.value.selectedMethodId)
				)
			case 'tremendous-details': {
				const method = withdrawData.value.selectedMethod
				// For gift card categories (merchant, charity), we need a specific method ID
				if (method === 'merchant_card' || method === 'charity') {
					return !!(
						withdrawData.value.selectedMethodId &&
						withdrawData.value.amount > 0 &&
						withdrawData.value.deliveryEmail
					)
				}
				// For paypal/venmo/visa, we just need amount and email
				return !!(withdrawData.value.amount > 0 && withdrawData.value.deliveryEmail)
			}
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
		availableMethods,
		paymentOptions,
		setStage,
		validateCurrentStage,
		resetData,
	}
}
