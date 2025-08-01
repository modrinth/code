import type Stripe from 'stripe'
import { type Stripe as StripeJs, loadStripe, type StripeElements } from '@stripe/stripe-js'
import { computed, ref, type Ref } from 'vue'
import type { ContactOption } from '@stripe/stripe-js/dist/stripe-js/elements/address'
import type {
  ServerPlan,
  BasePaymentIntentResponse,
  ChargeRequestType,
  CreatePaymentIntentRequest,
  CreatePaymentIntentResponse,
  PaymentRequestType,
  ServerBillingInterval,
  UpdatePaymentIntentRequest,
  UpdatePaymentIntentResponse,
} from '../utils/billing.ts'

// export type CreateElements = (
//   paymentMethods: Stripe.PaymentMethod[],
//   options: StripeElementsOptionsMode,
// ) => {
//   elements: StripeElements
//   paymentElement: StripePaymentElement
//   addressElement: StripeAddressElement
// }

export const useStripe = (
  publishableKey: string,
  customer: Stripe.Customer,
  paymentMethods: Stripe.PaymentMethod[],
  currency: string,
  product: Ref<ServerPlan | undefined>,
  interval: Ref<ServerBillingInterval>,
  region: Ref<string | undefined>,
  project: Ref<string | undefined>,
  initiatePayment: (
    body: CreatePaymentIntentRequest | UpdatePaymentIntentRequest,
  ) => Promise<CreatePaymentIntentResponse | UpdatePaymentIntentResponse>,
  onError: (err: Error) => void,
) => {
  const stripe = ref<StripeJs | null>(null)

  let elements: StripeElements | undefined = undefined
  const elementsLoaded = ref<0 | 1 | 2>(0)
  const loadingElementsFailed = ref<boolean>(false)

  const paymentMethodLoading = ref(false)
  const loadingFailed = ref<string>()
  const paymentIntentId = ref<string>()
  const tax = ref<number>()
  const total = ref<number>()
  const confirmationToken = ref<string>()
  const submittingPayment = ref(false)
  const selectedPaymentMethod = ref<Stripe.PaymentMethod>()
  const inputtedPaymentMethod = ref<Stripe.PaymentMethod>()
  const clientSecret = ref<string>()
  const completingPurchase = ref<boolean>(false)

  async function initialize() {
    stripe.value = await loadStripe(publishableKey)
  }

  function createIntent(body: CreatePaymentIntentRequest): Promise<CreatePaymentIntentResponse> {
    return initiatePayment(body) as Promise<CreatePaymentIntentResponse>
  }

  function updateIntent(body: UpdatePaymentIntentRequest): Promise<UpdatePaymentIntentResponse> {
    return initiatePayment(body) as Promise<UpdatePaymentIntentResponse>
  }

  const planPrices = computed(() => {
    return product.value?.prices.find((x) => x.currency_code === currency)
  })

  const createElements = (options) => {
    const styles = getComputedStyle(document.body)

    if (!stripe.value) {
      throw new Error('Stripe API not yet loaded')
    }

    elements = stripe.value.elements({
      appearance: {
        variables: {
          colorPrimary: styles.getPropertyValue('--color-brand'),
          colorBackground: styles.getPropertyValue('--experimental-color-button-bg'),
          colorText: styles.getPropertyValue('--color-base'),
          colorTextPlaceholder: styles.getPropertyValue('--color-secondary'),
          colorDanger: styles.getPropertyValue('--color-red'),
          fontFamily: styles.getPropertyValue('--font-standard'),
          spacingUnit: '0.25rem',
          borderRadius: '0.75rem',
        },
      },
      loader: 'never',
      ...options,
    })

    const paymentElement = elements.create('payment', {
      layout: {
        type: 'tabs',
        defaultCollapsed: false,
      },
    })
    paymentElement.mount('#payment-element')

    const contacts: ContactOption[] = []

    paymentMethods.forEach((method) => {
      const addr = method.billing_details?.address
      if (
        addr &&
        addr.line1 &&
        addr.city &&
        addr.postal_code &&
        addr.country &&
        addr.state &&
        method.billing_details.name
      ) {
        contacts.push({
          address: {
            line1: addr.line1,
            line2: addr.line2 ?? undefined,
            city: addr.city,
            state: addr.state,
            postal_code: addr.postal_code,
            country: addr.country,
          },
          name: method.billing_details.name,
        })
      }
    })

    const addressElement = elements.create('address', {
      mode: 'billing',
      contacts: contacts.length > 0 ? contacts : undefined,
    })
    addressElement.mount('#address-element')

    return { elements, paymentElement, addressElement }
  }

  const primaryPaymentMethodId = computed<string | null>(() => {
    if (customer && customer.invoice_settings && customer.invoice_settings.default_payment_method) {
      const method = customer.invoice_settings.default_payment_method
      if (typeof method === 'string') {
        return method
      } else {
        return method.id
      }
    } else if (paymentMethods && paymentMethods[0] && paymentMethods[0].id) {
      return paymentMethods[0].id
    } else {
      return null
    }
  })

  const loadStripeElements = async () => {
    loadingFailed.value = undefined
    try {
      if (!customer && primaryPaymentMethodId.value) {
        paymentMethodLoading.value = true
        await refreshPaymentIntent(primaryPaymentMethodId.value, false)
        paymentMethodLoading.value = false
      }

      if (!selectedPaymentMethod.value) {
        elementsLoaded.value = 0

        const {
          elements: newElements,
          addressElement,
          paymentElement,
        } = createElements({
          mode: 'payment',
          currency: currency.toLowerCase(),
          amount: product.value?.prices.find((x) => x.currency_code === currency)?.prices.intervals[
            interval.value
          ],
          paymentMethodCreation: 'manual',
          setupFutureUsage: 'off_session',
        })

        elements = newElements
        paymentElement.on('ready', () => {
          elementsLoaded.value += 1
        })
        addressElement.on('ready', () => {
          elementsLoaded.value += 1
        })
      }
    } catch (err) {
      loadingFailed.value = String(err)
      console.log(err)
    }
  }

  async function refreshPaymentIntent(id: string, confirmation: boolean) {
    try {
      paymentMethodLoading.value = true
      if (!confirmation) {
        selectedPaymentMethod.value = paymentMethods.find((x) => x.id === id)
      }

      const requestType: PaymentRequestType = confirmation
        ? {
            type: 'confirmation_token',
            token: id,
          }
        : {
            type: 'payment_method',
            id: id,
          }

      if (!product.value) {
        return handlePaymentError('No product selected')
      }

      const charge: ChargeRequestType = {
        type: 'new',
        product_id: product.value?.id,
        interval: interval.value,
      }

      let result: BasePaymentIntentResponse

      const metadata: CreatePaymentIntentRequest['metadata'] = {
        type: 'pyro',
        server_region: region.value,
        source: project.value
          ? {
              project_id: project.value,
            }
          : {},
      }

      if (paymentIntentId.value) {
        result = await updateIntent({
          ...requestType,
          charge,
          existing_payment_intent: paymentIntentId.value,
          metadata,
        })
        console.log(`Updated payment intent: ${interval.value} for ${result.total}`)
      } else {
        ;({
          payment_intent_id: paymentIntentId.value,
          client_secret: clientSecret.value,
          ...result
        } = await createIntent({
          ...requestType,
          charge,
          metadata: metadata,
        }))
        console.log(`Created payment intent: ${interval.value} for ${result.total}`)
      }

      tax.value = result.tax
      total.value = result.total

      if (confirmation) {
        confirmationToken.value = id
        if (result.payment_method) {
          inputtedPaymentMethod.value = result.payment_method
        }
      }
    } catch (err) {
      handlePaymentError(err as string)
    }
    paymentMethodLoading.value = false
  }

  async function createConfirmationToken() {
    if (!elements) {
      return handlePaymentError('No elements')
    }
    if (!stripe.value) {
      return handlePaymentError('No stripe')
    }

    const { error, confirmationToken: confirmation } = await stripe.value.createConfirmationToken({
      elements,
    })

    if (error) {
      handlePaymentError(error.message ?? 'Unknown error creating confirmation token')
      return
    }

    return confirmation.id
  }

  function handlePaymentError(err: string | Error) {
    paymentMethodLoading.value = false
    completingPurchase.value = false
    onError(typeof err === 'string' ? new Error(err) : err)
  }

  async function createNewPaymentMethod() {
    paymentMethodLoading.value = true

    if (!elements) {
      return handlePaymentError('No elements')
    }

    const { error: submitError } = await elements.submit()

    if (submitError) {
      return handlePaymentError(submitError.message ?? 'Unknown error creating payment method')
    }

    const token = await createConfirmationToken()
    if (!token) {
      return handlePaymentError('Failed to create confirmation token')
    }
    await refreshPaymentIntent(token, true)

    if (!planPrices.value) {
      return handlePaymentError('No plan prices')
    }
    if (!total.value) {
      return handlePaymentError('No total amount')
    }

    elements.update({ currency: planPrices.value.currency_code.toLowerCase(), amount: total.value })

    elementsLoaded.value = 0
    confirmationToken.value = token
    paymentMethodLoading.value = false

    return token
  }

  async function selectPaymentMethod(paymentMethod: Stripe.PaymentMethod | undefined) {
    selectedPaymentMethod.value = paymentMethod
    if (paymentMethod === undefined) {
      await loadStripeElements()
    } else {
      refreshPaymentIntent(paymentMethod.id, false)
    }
  }

  const loadingElements = computed(() => elementsLoaded.value < 2)

  async function submitPayment(returnUrl: string) {
    completingPurchase.value = true
    const secert = clientSecret.value

    if (!secert) {
      return handlePaymentError('No client secret')
    }

    if (!stripe.value) {
      return handlePaymentError('No stripe')
    }

    submittingPayment.value = true
    const { error } = await stripe.value.confirmPayment({
      clientSecret: secert,
      confirmParams: {
        confirmation_token: confirmationToken.value,
        return_url: `${returnUrl}?priceId=${product.value?.prices.find((x) => x.currency_code === currency)?.id}&plan=${interval.value}`,
      },
    })

    if (error) {
      handlePaymentError(error.message ?? 'Unknown error submitting payment')
      return false
    }
    submittingPayment.value = false
    completingPurchase.value = false
    return true
  }

  async function reloadPaymentIntent() {
    console.log('selected:', selectedPaymentMethod.value)
    console.log('token:', confirmationToken.value)
    if (selectedPaymentMethod.value) {
      await refreshPaymentIntent(selectedPaymentMethod.value.id, false)
    } else if (confirmationToken.value) {
      await refreshPaymentIntent(confirmationToken.value, true)
    } else {
      throw new Error('No payment method selected')
    }
  }

  const hasPaymentMethod = computed(() => selectedPaymentMethod.value || confirmationToken.value)

  return {
    initializeStripe: initialize,
    selectPaymentMethod,
    reloadPaymentIntent,
    primaryPaymentMethodId,
    selectedPaymentMethod,
    inputtedPaymentMethod,
    hasPaymentMethod,
    createNewPaymentMethod,
    loadingElements,
    loadingElementsFailed,
    paymentMethodLoading,
    loadStripeElements,
    tax,
    total,
    submitPayment,
    completingPurchase,
  }
}
