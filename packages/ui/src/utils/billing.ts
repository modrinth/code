import type Stripe from 'stripe'
import type { Loaders } from '@modrinth/utils'

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

/*
  Response types
*/
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
