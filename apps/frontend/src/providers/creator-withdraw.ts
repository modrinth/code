import {
	BadgeDollarSignIcon,
	GiftIcon,
	HandHelpingIcon,
	LandmarkIcon,
	SSOPayPalIcon,
	SSOVenmoIcon,
} from '@modrinth/assets'
import { createContext, paymentMethodMessages, useDebugLogger } from '@modrinth/ui'
import type { MessageDescriptor } from '@vintl/vintl'
import { type Component, computed, type ComputedRef, type Ref, ref } from 'vue'

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
 * only used for the method selection stage logic - not actually for API requests
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
	label: string | MessageDescriptor
	icon: Component
	methodId: string | undefined
	fee: string
	type: string
}

export interface Country {
	id: string
	name: string
}

export interface WithdrawalResult {
	created: Date
	amount: number
	fee: number
	netAmount: number
	methodType: string
	recipientDisplay: string
}

export interface KycData {
	type: 'individual' | 'business'
	email: string
	firstName?: string
	lastName?: string
	dateOfBirth?: string
	name?: string
	physicalAddress: {
		address1: string
		address2?: string
		city: string
		state: string
		country: string
		zip: string
	}
}

export interface AccountDetails {
	bankName?: string
	walletAddress?: string
	documentNumber?: string
	[key: string]: any // for dynamic rail fields
}

export interface GiftCardDetails {
	[key: string]: any
}

export interface SelectionData {
	country: Country | null
	provider: PaymentProvider | null
	method: string | null
	methodId: string | null
}

export interface TaxData {
	skipped: boolean
}

export interface CalculationData {
	amount: number
	fee: number | null
	exchangeRate: number | null
}

export interface TremendousProviderData {
	type: 'tremendous'
	deliveryEmail: string
	giftCardDetails: GiftCardDetails | null
}

export interface MuralPayProviderData {
	type: 'muralpay'
	kycData: KycData
	accountDetails: AccountDetails
}

export interface NoProviderData {
	type: null
}

export type ProviderData = TremendousProviderData | MuralPayProviderData | NoProviderData

export interface WithdrawData {
	selection: SelectionData
	tax: TaxData
	calculation: CalculationData
	providerData: ProviderData
	result: WithdrawalResult | null
	agreedTerms: boolean
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
	calculateFees: () => Promise<{ fee: number | null; exchange_rate: number | null }>
	submitWithdrawal: () => Promise<void>
}

export const [injectWithdrawContext, provideWithdrawContext] =
	createContext<WithdrawContextValue>('CreatorWithdrawModal')

export function useWithdrawContext() {
	return injectWithdrawContext()
}

function isTremendousProvider(data: ProviderData): data is TremendousProviderData {
	return data.type === 'tremendous'
}

function isMuralPayProvider(data: ProviderData): data is MuralPayProviderData {
	return data.type === 'muralpay'
}

function buildRecipientInfo(kycData: KycData) {
	return {
		type: kycData.type,
		...(kycData.type === 'individual'
			? {
					firstName: kycData.firstName,
					lastName: kycData.lastName,
					dateOfBirth: kycData.dateOfBirth,
				}
			: {
					name: kycData.name,
				}),
		email: kycData.email,
		physicalAddress: kycData.physicalAddress,
	}
}

function getAccountOwnerName(kycData: KycData): string {
	if (kycData.type === 'individual') {
		return `${kycData.firstName} ${kycData.lastName}`
	}
	return kycData.name || ''
}

function getMethodDisplayName(method: string | null): string {
	if (!method) return ''
	const methodMap: Record<string, string> = {
		paypal: 'PayPal',
		venmo: 'Venmo',
		merchant_card: 'Gift Card',
		charity: 'Charity',
		visa_card: 'Virtual Visa',
	}
	if (methodMap[method]) return methodMap[method]
	if (method.startsWith('fiat_')) {
		return 'Bank Transfer'
	}
	if (method.startsWith('blockchain_')) {
		return 'Crypto'
	}
	return method
}

function getRecipientDisplay(data: WithdrawData): string {
	if (isTremendousProvider(data.providerData)) {
		return data.providerData.deliveryEmail
	}
	if (isMuralPayProvider(data.providerData)) {
		const kycData = data.providerData.kycData
		if (kycData.type === 'individual') {
			return `${kycData.firstName} ${kycData.lastName}`
		}
		return kycData.name || ''
	}
	return ''
}

interface PayoutPayload {
	amount: number
	method: 'tremendous' | 'muralpay'
	method_id: string
	method_details?: {
		delivery_email?: string
		payout_details?: any
		recipient_info?: any
	}
}

function buildPayoutPayload(data: WithdrawData): PayoutPayload {
	if (data.selection.provider === 'tremendous') {
		if (!isTremendousProvider(data.providerData)) {
			throw new Error('Invalid provider data for Tremendous')
		}
		return {
			amount: data.calculation.amount,
			method: 'tremendous',
			method_id: data.selection.methodId!,
			method_details: {
				delivery_email: data.providerData.deliveryEmail,
			},
		}
	} else if (data.selection.provider === 'muralpay') {
		if (!isMuralPayProvider(data.providerData)) {
			throw new Error('Invalid provider data for MuralPay')
		}
		const railId = data.selection.method!
		const rail = getRailConfig(railId)

		if (!rail) throw new Error('Invalid payment method')

		if (rail.type === 'crypto') {
			return {
				amount: data.calculation.amount,
				method: 'muralpay',
				method_id: data.selection.methodId!,
				method_details: {
					payout_details: {
						type: 'blockchain',
						wallet_address: data.providerData.accountDetails.walletAddress || null,
					},
					recipient_info: buildRecipientInfo(data.providerData.kycData),
				},
			}
		} else if (rail.type === 'fiat') {
			const fiatAndRailDetails: Record<string, any> = {
				type: rail.railCode || '',
				symbol: rail.currency || '',
			}

			for (const field of rail.fields) {
				const value = data.providerData.accountDetails[field.name]
				if (value !== undefined && value !== null && value !== '') {
					fiatAndRailDetails[field.name] = value
				}
			}

			if (data.providerData.accountDetails.documentNumber) {
				fiatAndRailDetails.documentNumber = data.providerData.accountDetails.documentNumber
			}

			return {
				amount: data.calculation.amount,
				method: 'muralpay',
				method_id: data.selection.methodId!,
				method_details: {
					payout_details: {
						type: 'fiat',
						bank_name: data.providerData.accountDetails.bankName || '',
						bank_account_owner: getAccountOwnerName(data.providerData.kycData),
						fiat_and_rail_details: fiatAndRailDetails,
					},
					recipient_info: buildRecipientInfo(data.providerData.kycData),
				},
			}
		}
	}

	throw new Error('Invalid provider')
}

export function createWithdrawContext(balance: any): WithdrawContextValue {
	const debug = useDebugLogger('CreatorWithdraw')
	const currentStage = ref<WithdrawStage | undefined>()

	const withdrawData = ref<WithdrawData>({
		selection: {
			country: null,
			provider: null,
			method: null,
			methodId: null,
		},
		tax: {
			skipped: false,
		},
		calculation: {
			amount: 0,
			fee: null,
			exchangeRate: null,
		},
		providerData: {
			type: null,
		},
		result: null,
		agreedTerms: false,
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

		const selectedProvider = withdrawData.value.selection.provider
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

		if (!withdrawData.value.tax.skipped) {
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
				icon: SSOPayPalIcon,
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
				icon: SSOVenmoIcon,
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
				icon: HandHelpingIcon,
				methodId: undefined,
				fee: '≈ 0%',
				type: 'tremendous',
			})
		}

		const muralPayMethods = methods.filter((m) => m.type === 'muralpay')
		for (const method of muralPayMethods) {
			const methodId = method.id

			if (methodId.startsWith('fiat_')) {
				const rail = getRailConfig(methodId)

				if (!rail) {
					debug('Warning: No rail config found for', methodId)
					continue
				}

				options.push({
					value: methodId,
					label: rail.name,
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
					label: rail.name,
					icon: getCurrencyIcon(rail.currency) || BadgeDollarSignIcon,
					methodId: method.id,
					fee: rail.fee,
					type: 'crypto',
				})
			}
		}

		const sortOrder = ['paypal', 'venmo', 'crypto', 'fiat', 'merchant_card', 'charity']
		options.sort((a, b) => {
			const getOrder = (item: PaymentOption) => {
				let order = sortOrder.indexOf(item.type)
				if (order === -1) order = sortOrder.indexOf(item.value)
				return order !== -1 ? order : 999
			}
			return getOrder(a) - getOrder(b)
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
				if (!balanceRef.value) return true
				const ytd = balanceRef.value.withdrawn_ytd ?? 0
				const remainingLimit = Math.max(0, 600 - ytd)
				const form_completion_status = balanceRef.value.form_completion_status
				if (ytd < 600) return true
				if (withdrawData.value.tax.skipped && remainingLimit > 0) return true
				return form_completion_status === 'complete'
			}
			case 'method-selection':
				return !!(
					withdrawData.value.selection.country &&
					withdrawData.value.selection.provider &&
					withdrawData.value.selection.method &&
					(withdrawData.value.selection.method === 'merchant_card' ||
						withdrawData.value.selection.method === 'charity' ||
						withdrawData.value.selection.methodId)
				)
			case 'tremendous-details': {
				const method = withdrawData.value.selection.method
				if (method === 'merchant_card' || method === 'charity') {
					if (!isTremendousProvider(withdrawData.value.providerData)) return false
					return !!(
						withdrawData.value.selection.methodId &&
						withdrawData.value.calculation.amount > 0 &&
						withdrawData.value.providerData.deliveryEmail &&
						withdrawData.value.agreedTerms
					)
				}
				if (!isTremendousProvider(withdrawData.value.providerData)) return false
				return !!(
					withdrawData.value.calculation.amount > 0 &&
					withdrawData.value.providerData.deliveryEmail &&
					withdrawData.value.agreedTerms
				)
			}
			case 'muralpay-kyc': {
				if (!isMuralPayProvider(withdrawData.value.providerData)) return false

				const kycData = withdrawData.value.providerData.kycData
				if (!kycData) return false

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
				if (!isMuralPayProvider(withdrawData.value.providerData)) return false

				const railId = withdrawData.value.selection.method
				const rail = getRailConfig(railId as string)
				if (!rail) return false

				if (!withdrawData.value.calculation.amount || withdrawData.value.calculation.amount <= 0)
					return false

				const accountDetails = withdrawData.value.providerData.accountDetails
				if (!accountDetails) return false

				if (rail.requiresBankName && !accountDetails.bankName) return false

				const requiredFields = rail.fields.filter((f) => f.required)
				const allRequiredPresent = requiredFields.every((f) => {
					const value = accountDetails[f.name]
					return value !== undefined && value !== null && value !== ''
				})

				return allRequiredPresent && withdrawData.value.agreedTerms
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
			selection: {
				country: null,
				provider: null,
				method: null,
				methodId: null,
			},
			tax: {
				skipped: false,
			},
			calculation: {
				amount: 0,
				fee: null,
				exchangeRate: null,
			},
			providerData: {
				type: null,
			},
			result: null,
			agreedTerms: false,
		}
		currentStage.value = undefined
	}

	async function calculateFees(): Promise<{ fee: number | null; exchange_rate: number | null }> {
		const payload = buildPayoutPayload(withdrawData.value)

		const response = (await useBaseFetch('payout/fees', {
			apiVersion: 3,
			method: 'POST',
			body: payload,
		})) as { fee: number | string | null; exchange_rate: number | string | null }

		const parsedFee = response.fee ? Number.parseFloat(String(response.fee)) : 0
		const parsedExchangeRate = response.exchange_rate
			? Number.parseFloat(String(response.exchange_rate))
			: null

		withdrawData.value.calculation.fee = parsedFee
		withdrawData.value.calculation.exchangeRate = parsedExchangeRate

		return {
			fee: parsedFee,
			exchange_rate: parsedExchangeRate,
		}
	}

	async function submitWithdrawal(): Promise<void> {
		const payload = buildPayoutPayload(withdrawData.value)

		debug('Withdrawal payload:', payload)

		await useBaseFetch('payout', {
			apiVersion: 3,
			method: 'POST',
			body: payload,
		})

		withdrawData.value.result = {
			created: new Date(),
			amount: withdrawData.value.calculation.amount,
			fee: withdrawData.value.calculation.fee || 0,
			netAmount: withdrawData.value.calculation.amount - (withdrawData.value.calculation.fee || 0),
			methodType: getMethodDisplayName(withdrawData.value.selection.method),
			recipientDisplay: getRecipientDisplay(withdrawData.value),
		}

		debug('Withdrawal submitted successfully', withdrawData.value.result)
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
		calculateFees,
		submitWithdrawal,
	}
}
