import type { Loaders } from '@modrinth/ui'
import type {
	StripeAddressElement,
	StripeAddressElementChangeEvent,
	StripeAddressElementOptions,
	StripeElements,
	StripeElementsOptions,
	StripePaymentElement,
} from '@stripe/stripe-js'
import type Stripe from 'stripe'

export type ServerBillingInterval = 'monthly' | 'yearly' | 'quarterly'

export const monthsInInterval: Record<ServerBillingInterval, number> = {
	monthly: 1,
	quarterly: 3,
	yearly: 12,
}

export interface ServerPlan {
	id: string
	name: string
	description: string
	metadata: {
		type: string
		ram?: number
		cpu?: number
		storage?: number
		swap?: number
	}
	prices: {
		id: string
		currency_code: string
		prices: {
			intervals: {
				monthly: number
				yearly: number
			}
		}
	}[]
}

export interface ServerStockRequest {
	cpu?: number
	memory_mb?: number
	swap_mb?: number
	storage_mb?: number
}

export interface ServerRegion {
	shortcode: string
	country_code: string
	display_name: string
	lat: number
	lon: number
}

/*
  Request types
*/
export type PaymentMethodRequest = {
	type: 'payment_method'
	id: string
}

export type ConfirmationTokenRequest = {
	type: 'confirmation_token'
	token: string
}

export type PaymentRequestType = PaymentMethodRequest | ConfirmationTokenRequest

export type ChargeRequestType =
	| {
			type: 'existing'
			id: string
	  }
	| {
			type: 'new'
			product_id: string
			interval?: ServerBillingInterval
	  }

export type CreatePaymentIntentRequest = PaymentRequestType & {
	charge: ChargeRequestType
	metadata?: {
		type: 'pyro'
		server_name?: string
		server_region?: string
		source:
			| {
					loader: Loaders
					game_version?: string
					loader_version?: string
			  }
			| {
					project_id: string
					version_id?: string
			  }
			// eslint-disable-next-line @typescript-eslint/no-empty-object-type
			| {}
	}
}

export type UpdatePaymentIntentRequest = CreatePaymentIntentRequest & {
	existing_payment_intent: string
}

export type BasePaymentIntentResponse = {
	price_id: string
	tax: number
	total: number
	payment_method: Stripe.PaymentMethod
}

export type UpdatePaymentIntentResponse = BasePaymentIntentResponse

export type CreatePaymentIntentResponse = BasePaymentIntentResponse & {
	payment_intent_id: string
	client_secret: string
}

export const getCurrency = (userCountry: string | number) => {
	const countryCurrency = {
		US: 'USD',
		GB: 'GBP',
		EU: 'EUR',
		AT: 'EUR',
		BE: 'EUR',
		CY: 'EUR',
		EE: 'EUR',
		FI: 'EUR',
		FR: 'EUR',
		DE: 'EUR',
		GR: 'EUR',
		IE: 'EUR',
		IT: 'EUR',
		LV: 'EUR',
		LT: 'EUR',
		LU: 'EUR',
		MT: 'EUR',
		NL: 'EUR',
		PT: 'EUR',
		SK: 'EUR',
		SI: 'EUR',
		RU: 'RUB',
		BR: 'BRL',
		JP: 'JPY',
		ID: 'IDR',
		MY: 'MYR',
		PH: 'PHP',
		TH: 'THB',
		VN: 'VND',
		KR: 'KRW',
		TR: 'TRY',
		UA: 'UAH',
		MX: 'MXN',
		CA: 'CAD',
		NZ: 'NZD',
		NO: 'NOK',
		PL: 'PLN',
		CH: 'CHF',
		LI: 'CHF',
		IN: 'INR',
		CL: 'CLP',
		PE: 'PEN',
		CO: 'COP',
		ZA: 'ZAR',
		HK: 'HKD',
		AR: 'ARS',
		KZ: 'KZT',
		UY: 'UYU',
		CN: 'CNY',
		AU: 'AUD',
		TW: 'TWD',
		SA: 'SAR',
		QA: 'QAR',
	}

	return countryCurrency[userCountry] ?? 'USD'
}

export function formatPrice(
	locale: Intl.LocalesArgument,
	price: number,
	currency: string,
	trimZeros = false,
) {
	let formatter = new Intl.NumberFormat(locale, {
		style: 'currency',
		currency,
	})

	const maxDigits = formatter.resolvedOptions().maximumFractionDigits ?? 2
	const convertedPrice = price / Math.pow(10, maxDigits)

	let minimumFractionDigits = maxDigits

	if (trimZeros && Number.isInteger(convertedPrice)) {
		minimumFractionDigits = 0
	}

	formatter = new Intl.NumberFormat(locale, {
		style: 'currency',
		currency,
		minimumFractionDigits,
	})
	return formatter.format(convertedPrice)
}

export function calculateSavings(monthlyPlan: number, plan: number, months = 12) {
	const monthlyAnnualized = monthlyPlan * months

	return Math.floor(((monthlyAnnualized - plan) / monthlyAnnualized) * 100)
}

// Extend StripeElements update type locally to allow currency updates without using `any`.
type StripeElementsUpdateOptionsWithCurrency = Parameters<StripeElements['update']>[0] & {
	currency?: string
}
interface StripeElementsCurrencyUpdatable extends StripeElements {
	update(options: StripeElementsUpdateOptionsWithCurrency): void
}

type MinimalStripe = { elements: (options?: unknown) => StripeElements }

export function createStripeElements(
	stripe: MinimalStripe,
	paymentMethods: Stripe.PaymentMethod[] | undefined,
	options?: StripeElementsOptions,
) {
	const styles = getComputedStyle(document.body)

	const baseOptions = {
		appearance: {
			variables: {
				colorPrimary: styles.getPropertyValue('--color-brand'),
				colorBackground: styles.getPropertyValue('--color-button-bg'),
				colorText: styles.getPropertyValue('--color-base'),
				colorTextPlaceholder: styles.getPropertyValue('--color-secondary'),
				colorDanger: styles.getPropertyValue('--color-red'),
				fontFamily: styles.getPropertyValue('--font-standard'),
				spacingUnit: '0.25rem',
				borderRadius: '0.75rem',
			},
		},
		loader: 'never',
	} as const

	const elements = stripe.elements(
		(options ? { ...baseOptions, ...options } : baseOptions) as unknown as StripeElementsOptions,
	)

	const paymentElement: StripePaymentElement = elements.create('payment')
	paymentElement.mount('#payment-element')

	const addressElement: StripeAddressElement = elements.create('address', {
		mode: 'billing',
		contacts: paymentMethods
			? (paymentMethods.map((pm) => ({
					address: pm.billing_details.address ?? undefined,
					email: pm.billing_details.email ?? undefined,
					name: pm.billing_details.name ?? undefined,
				})) as NonNullable<StripeAddressElementOptions['contacts']>)
			: undefined,
	})
	addressElement.mount('#address-element')

	addressElement.on('change', (e: StripeAddressElementChangeEvent) => {
		const country = e.value?.address?.country
		if (country) {
			;(elements as unknown as StripeElementsCurrencyUpdatable).update({
				currency: getCurrency(country).toLowerCase(),
			})
		}
	})

	return { elements, paymentElement, addressElement }
}
