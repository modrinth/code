export namespace Labrinth {
	export namespace Billing {
		export namespace Internal {
			export type PriceDuration = 'five-days' | 'monthly' | 'quarterly' | 'yearly'

			export type SubscriptionStatus = 'provisioned' | 'unprovisioned'

			export type UserSubscription = {
				id: string
				user_id: string
				price_id: string
				interval: PriceDuration
				status: SubscriptionStatus
				created: string // ISO datetime string
				metadata?: SubscriptionMetadata
			}

			export type SubscriptionMetadata =
				| { type: 'pyro'; id: string; region?: string }
				| { type: 'medal'; id: string }

			export type ChargeStatus =
				| 'open'
				| 'processing'
				| 'succeeded'
				| 'failed'
				| 'cancelled'
				| 'expiring'

			export type ChargeType = 'one-time' | 'subscription' | 'proration' | 'refund'

			export type PaymentPlatform = 'Stripe' | 'None'

			export type Charge = {
				id: string
				user_id: string
				price_id: string
				amount: number
				currency_code: string
				status: ChargeStatus
				due: string // ISO datetime string
				last_attempt: string | null // ISO datetime string
				type: ChargeType
				subscription_id: string | null
				subscription_interval: PriceDuration | null
				platform: PaymentPlatform
				parent_charge_id: string | null
				net: number | null
			}

			export type ProductMetadata =
				| { type: 'midas' }
				| {
						type: 'pyro'
						cpu: number
						ram: number
						swap: number
						storage: number
				  }
				| {
						type: 'medal'
						cpu: number
						ram: number
						swap: number
						storage: number
						region: string
				  }

			export type Price =
				| { type: 'one-time'; price: number }
				| { type: 'recurring'; intervals: Record<PriceDuration, number> }

			export type ProductPrice = {
				id: string
				product_id: string
				prices: Price
				currency_code: string
			}

			export type Product = {
				id: string
				metadata: ProductMetadata
				prices: ProductPrice[]
				unitary: boolean
			}

			export type EditSubscriptionRequest = {
				interval?: PriceDuration
				payment_method?: string
				cancelled?: boolean
				region?: string
				product?: string
			}

			export type EditSubscriptionResponse = {
				payment_intent_id: string
				client_secret: string
				tax: number
				total: number
			}

			export type AddPaymentMethodFlowResponse = {
				client_secret: string
			}

			export type EditPaymentMethodRequest = {
				primary: boolean
			}

			export type InitiatePaymentRequest = {
				type: 'payment_method' | 'confirmation_token'
				id?: string
				token?: string
				charge:
					| { type: 'existing'; id: string }
					| { type: 'new'; product_id: string; interval?: PriceDuration }
				existing_payment_intent?: string
				metadata?: {
					type: 'pyro'
					server_name?: string
					server_region?: string
					source: unknown
				}
			}

			export type InitiatePaymentResponse = {
				payment_intent_id?: string
				client_secret?: string
				price_id: string
				tax: number
				total: number
				payment_method?: string
			}

			export type RefundChargeRequest = {
				type: 'full' | 'partial' | 'none'
				amount?: number
				unprovision?: boolean
			}

			export type CreditRequest =
				| { subscription_ids: string[]; days: number; send_email: boolean; message: string }
				| { nodes: string[]; days: number; send_email: boolean; message: string }
				| { region: string; days: number; send_email: boolean; message: string }
		}
	}
}
