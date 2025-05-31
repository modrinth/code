import type Stripe from 'stripe'

export type ServerBillingInterval = 'monthly' | 'yearly' | 'quarterly'

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
    source: {
      loader: string
      game_version?: string
      loader_version?: string
    }
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
