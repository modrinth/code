<template>
  <NewModal ref="purchaseModal">
    <template #title>
      <span class="text-contrast text-xl font-extrabold">
        <template v-if="product.metadata.type === 'midas'">Subscribe to Modrinth Plus!</template>
        <template v-else>Purchase product</template>
      </span>
    </template>
    <div class="flex items-center gap-1 pb-4">
      <span
        :class="{
          'text-secondary': purchaseModalStep !== 0,
          'font-bold': purchaseModalStep === 0,
        }"
      >
        Select plan
      </span>
      <ChevronRightIcon class="h-5 w-5 text-secondary" />
      <span
        :class="{
          'text-secondary': purchaseModalStep !== 1,
          'font-bold': purchaseModalStep === 1,
        }"
      >
        Payment
      </span>
      <ChevronRightIcon class="h-5 w-5 text-secondary" />
      <span
        :class="{
          'text-secondary': purchaseModalStep !== 2,
          'font-bold': purchaseModalStep === 2,
        }"
      >
        Review
      </span>
    </div>
    <div v-if="purchaseModalStep === 0" class="md:w-[600px]">
      <div>
        <p class="my-2 text-lg font-bold">Choose billing interval</p>
        <div class="flex flex-col gap-4">
          <div
            v-for="([interval, rawPrice], index) in Object.entries(price.prices.intervals)"
            :key="index"
            class="flex cursor-pointer items-center gap-2"
            @click="selectedPlan = interval"
          >
            <RadioButtonChecked v-if="selectedPlan === interval" class="h-8 w-8 text-brand" />
            <RadioButtonIcon v-else class="h-8 w-8 text-secondary" />
            <span
              class="text-lg capitalize"
              :class="{ 'text-secondary': selectedPlan !== interval }"
            >
              {{ interval }}
            </span>
            <span
              v-if="interval === 'yearly'"
              class="rounded-full bg-brand px-2 py-1 font-bold text-brand-inverted"
            >
              SAVE {{ calculateSavings(price.prices.intervals.monthly, rawPrice) }}%
            </span>
            <span class="ml-auto text-lg" :class="{ 'text-secondary': selectedPlan !== interval }">
              {{ formatPrice(locale, rawPrice, price.currency_code) }}
            </span>
          </div>
        </div>
        <div class="mt-4 flex justify-between border-0 border-t border-solid border-code-bg pt-4">
          <span class="text-xl text-secondary">Total</span>
          <div class="flex items-baseline gap-2">
            <span class="text-2xl font-extrabold text-primary">
              {{ formatPrice(locale, price.prices.intervals[selectedPlan], price.currency_code) }}
            </span>
            <span class="text-lg text-secondary">/ {{ selectedPlan }}</span>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-4">
          <InfoIcon />
          <span class="text-sm text-secondary">
            Final price and currency will be based on your selected payment method.
          </span>
        </div>
      </div>
    </div>
    <template v-if="purchaseModalStep === 1">
      <div
        v-show="loadingPaymentMethodModal !== 2"
        class="flex min-h-[16rem] items-center justify-center md:w-[600px]"
      >
        <AnimatedLogo class="w-[80px]" />
      </div>
      <div v-show="loadingPaymentMethodModal === 2" class="min-h-[16rem] p-1 md:w-[600px]">
        <div id="address-element"></div>
        <div id="payment-element" class="mt-4"></div>
      </div>
    </template>
    <div v-if="purchaseModalStep === 2" class="md:w-[650px]">
      <div>
        <div class="r-4 rounded-xl bg-bg p-4">
          <p class="my-2 text-lg font-bold text-primary">Purchase details</p>
          <div class="mb-2 flex justify-between">
            <span class="text-secondary">Modrinth+ {{ selectedPlan }}</span>
            <span class="text-secondary">
              {{ formatPrice(locale, price.prices.intervals[selectedPlan], price.currency_code) }} /
              {{ selectedPlan }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-secondary">Tax</span>
            <span class="text-secondary">{{ formatPrice(locale, tax, price.currency_code) }}</span>
          </div>
          <div class="mt-4 flex justify-between border-0 border-t border-solid border-code-bg pt-4">
            <span class="text-lg font-bold">Today's total</span>
            <span class="text-lg font-extrabold text-primary">
              {{ formatPrice(locale, price.prices.intervals[selectedPlan], price.currency_code) }}
            </span>
          </div>
        </div>
        <p class="my-2 text-lg font-bold">Pay for it with</p>
        <multiselect
          v-model="selectedPaymentMethod"
          placeholder="Payment method"
          label="id"
          track-by="id"
          :options="selectablePaymentMethods"
          :option-height="104"
          :show-labels="false"
          :searchable="false"
          :close-on-select="true"
          :allow-empty="false"
          open-direction="top"
          class="max-w-[20rem]"
          @select="selectPaymentMethod"
        >
          <!-- TODO: Move this to component to remove duplicated code -->
          <template #singleLabel="props">
            <div class="flex items-center gap-2">
              <CardIcon v-if="props.option.type === 'card'" class="h-8 w-8" />
              <CurrencyIcon v-else-if="props.option.type === 'cashapp'" class="h-8 w-8" />
              <PayPalIcon v-else-if="props.option.type === 'paypal'" class="h-8 w-8" />

              <span v-if="props.option.type === 'card'">
                {{
                  formatMessage(messages.paymentMethodCardDisplay, {
                    card_brand:
                      formatMessage(paymentMethodTypes[props.option.card.brand]) ??
                      formatMessage(paymentMethodTypes.unknown),
                    last_four: props.option.card.last4,
                  })
                }}
              </span>
              <template v-else>
                {{
                  formatMessage(paymentMethodTypes[props.option.type]) ??
                  formatMessage(paymentMethodTypes.unknown)
                }}
              </template>

              <span v-if="props.option.type === 'cashapp' && props.option.cashapp.cashtag">
                ({{ props.option.cashapp.cashtag }})
              </span>
              <span v-else-if="props.option.type === 'paypal' && props.option.paypal.payer_email">
                ({{ props.option.paypal.payer_email }})
              </span>
            </div>
          </template>
          <template #option="props">
            <div class="flex items-center gap-2">
              <template v-if="props.option.id === 'new'">
                <PlusIcon class="h-8 w-8" />
                <span class="text-secondary">Add payment method</span>
              </template>
              <template v-else>
                <CardIcon v-if="props.option.type === 'card'" class="h-8 w-8" />
                <CurrencyIcon v-else-if="props.option.type === 'cashapp'" class="h-8 w-8" />
                <PayPalIcon v-else-if="props.option.type === 'paypal'" class="h-8 w-8" />

                <span v-if="props.option.type === 'card'">
                  {{
                    formatMessage(messages.paymentMethodCardDisplay, {
                      card_brand:
                        formatMessage(paymentMethodTypes[props.option.card.brand]) ??
                        formatMessage(paymentMethodTypes.unknown),
                      last_four: props.option.card.last4,
                    })
                  }}
                </span>
                <template v-else>
                  {{
                    formatMessage(paymentMethodTypes[props.option.type]) ??
                    formatMessage(paymentMethodTypes.unknown)
                  }}
                </template>

                <span v-if="props.option.type === 'cashapp'">
                  ({{ props.option.cashapp.cashtag }})
                </span>
                <span v-else-if="props.option.type === 'paypal'">
                  ({{ props.option.paypal.payer_email }})
                </span>
              </template>
            </div>
          </template>
        </multiselect>
      </div>
      <p class="m-0 mt-9 text-sm text-secondary">
        <strong>By clicking "Subscribe", you are purchasing a recurring subscription.</strong>
        You'll be charged
        {{ formatPrice(locale, price.prices.intervals[selectedPlan], price.currency_code) }} /
        {{ selectedPlan }} plus applicable taxes starting today, until you cancel. Cancel anytime
        from your settings page.
      </p>
    </div>
    <div class="input-group push-right pt-4">
      <template v-if="purchaseModalStep === 0">
        <button class="btn" @click="$refs.purchaseModal.hide()">
          <XIcon />
          Cancel
        </button>
        <button class="btn btn-primary" :disabled="paymentLoading" @click="beginPurchaseFlow(true)">
          <RightArrowIcon />
          Select
        </button>
      </template>
      <template v-else-if="purchaseModalStep === 1">
        <button
          class="btn"
          @click="
            () => {
              purchaseModalStep = 0
              loadingPaymentMethodModal = 0
              paymentLoading = false
            }
          "
        >
          Back
        </button>
        <button class="btn btn-primary" :disabled="paymentLoading" @click="validatePayment">
          <RightArrowIcon />
          Continue
        </button>
      </template>
      <template v-else-if="purchaseModalStep === 2">
        <button class="btn" @click="$refs.purchaseModal.hide()">
          <XIcon />
          Cancel
        </button>
        <button class="btn btn-primary" :disabled="paymentLoading" @click="submitPayment">
          <CheckCircleIcon /> Subscribe
        </button>
      </template>
    </div>
  </NewModal>
</template>
<script setup>
import { ref, computed, nextTick } from 'vue'
import NewModal from '../modal/NewModal.vue'
import {
  CardIcon,
  CheckCircleIcon,
  ChevronRightIcon,
  CurrencyIcon,
  InfoIcon,
  PayPalIcon,
  PlusIcon,
  RadioButtonChecked,
  RadioButtonIcon,
  RightArrowIcon,
  XIcon,
} from '@modrinth/assets'
import AnimatedLogo from '../brand/AnimatedLogo.vue'
import { getCurrency, calculateSavings, formatPrice, createStripeElements } from '@modrinth/utils'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { Multiselect } from 'vue-multiselect'

const { locale, formatMessage } = useVIntl()

const props = defineProps({
  product: {
    type: Object,
    required: true,
  },
  customer: {
    type: Object,
    required: true,
  },
  paymentMethods: {
    type: Array,
    required: true,
  },
  country: {
    type: String,
    required: true,
  },
  returnUrl: {
    type: String,
    required: true,
  },
  publishableKey: {
    type: String,
    required: true,
  },
  fetchPaymentData: {
    type: Function,
    default: async () => {},
  },
  sendBillingRequest: {
    type: Function,
    required: true,
  },
  onError: {
    type: Function,
    required: true,
  },
})

const messages = defineMessages({
  paymentMethodCardDisplay: {
    id: 'omorphia.component.purchase_modal.payment_method_card_display',
    defaultMessage: '{card_brand} ending in {last_four}',
  },
})

const paymentMethodTypes = defineMessages({
  visa: {
    id: 'omorphia.component.purchase_modal.payment_method_type.visa',
    defaultMessage: 'Visa',
  },
  amex: {
    id: 'omorphia.component.purchase_modal.payment_method_type.amex',
    defaultMessage: 'American Express',
  },
  diners: {
    id: 'omorphia.component.purchase_modal.payment_method_type.diners',
    defaultMessage: 'Diners Club',
  },
  discover: {
    id: 'omorphia.component.purchase_modal.payment_method_type.discover',
    defaultMessage: 'Discover',
  },
  eftpos: {
    id: 'omorphia.component.purchase_modal.payment_method_type.eftpos',
    defaultMessage: 'EFTPOS',
  },
  jcb: { id: 'omorphia.component.purchase_modal.payment_method_type.jcb', defaultMessage: 'JCB' },
  mastercard: {
    id: 'omorphia.component.purchase_modal.payment_method_type.mastercard',
    defaultMessage: 'MasterCard',
  },
  unionpay: {
    id: 'omorphia.component.purchase_modal.payment_method_type.unionpay',
    defaultMessage: 'UnionPay',
  },
  paypal: {
    id: 'omorphia.component.purchase_modal.payment_method_type.paypal',
    defaultMessage: 'PayPal',
  },
  cashapp: {
    id: 'omorphia.component.purchase_modal.payment_method_type.cashapp',
    defaultMessage: 'Cash App',
  },
  amazon_pay: {
    id: 'omorphia.component.purchase_modal.payment_method_type.amazon_pay',
    defaultMessage: 'Amazon Pay',
  },
  unknown: {
    id: 'omorphia.component.purchase_modal.payment_method_type.unknown',
    defaultMessage: 'Unknown payment method',
  },
})

let stripe = null
let elements = null

const purchaseModal = ref()

const primaryPaymentMethodId = computed(() => {
  if (
    props.customer &&
    props.customer.invoice_settings &&
    props.customer.invoice_settings.default_payment_method
  ) {
    return props.customer.invoice_settings.default_payment_method
  } else if (props.paymentMethods && props.paymentMethods[0] && props.paymentMethods[0].id) {
    return props.paymentMethods[0].id
  } else {
    return null
  }
})

defineExpose({
  show: () => {
    // eslint-disable-next-line no-undef
    stripe = Stripe(props.publishableKey)

    selectedPlan.value = 'yearly'

    purchaseModalStep.value = 0
    loadingPaymentMethodModal.value = 0
    paymentLoading.value = false

    purchaseModal.value.show()
  },
})

const purchaseModalStep = ref(0)
const loadingPaymentMethodModal = ref(0)

const selectedPlan = ref('yearly')
const currency = computed(() => getCurrency(props.country))

const price = ref(props.product.prices.find((x) => x.currency_code === currency.value))

const clientSecret = ref()
const paymentIntentId = ref()
const confirmationToken = ref()
const tax = ref()
const total = ref()

const selectedPaymentMethod = ref()
const inputtedPaymentMethod = ref()
const selectablePaymentMethods = computed(() => {
  const values = [
    ...(props.paymentMethods ?? []),
    {
      id: 'new',
    },
  ]

  if (inputtedPaymentMethod.value) {
    values.unshift(inputtedPaymentMethod.value)
  }

  return values
})

const paymentLoading = ref(false)

async function beginPurchaseFlow(skip = false) {
  if (!props.customer) {
    paymentLoading.value = true
    await props.fetchPaymentData()
    paymentLoading.value = false
  }

  if (primaryPaymentMethodId.value && skip) {
    paymentLoading.value = true
    await refreshPayment(null, primaryPaymentMethodId.value)
    paymentLoading.value = false
    purchaseModalStep.value = 2
  } else {
    try {
      loadingPaymentMethodModal.value = 0
      purchaseModalStep.value = 1

      await nextTick()

      const {
        elements: elementsVal,
        addressElement,
        paymentElement,
      } = createStripeElements(stripe, props.paymentMethods, {
        mode: 'payment',
        amount: price.value.prices.intervals[selectedPlan.value],
        currency: price.value.currency_code.toLowerCase(),
        paymentMethodCreation: 'manual',
        setupFutureUsage: 'off_session',
      })

      elements = elementsVal
      paymentElement.on('ready', () => {
        loadingPaymentMethodModal.value += 1
      })
      addressElement.on('ready', () => {
        loadingPaymentMethodModal.value += 1
      })
    } catch (err) {
      props.onError(err)
    }
  }
}

async function createConfirmationToken() {
  const { error, confirmationToken: confirmation } = await stripe.createConfirmationToken({
    elements,
  })

  if (error) {
    props.onError(error)

    return
  }

  return confirmation.id
}

async function validatePayment() {
  paymentLoading.value = true
  const { error: submitError } = await elements.submit()

  if (submitError) {
    paymentLoading.value = false
    props.onError(submitError)
    return
  }

  await refreshPayment(await createConfirmationToken())

  elements.update({ currency: price.value.currency_code.toLowerCase(), amount: total.value })

  loadingPaymentMethodModal.value = 0
  confirmationToken.value = await createConfirmationToken()
  purchaseModalStep.value = 2
  paymentLoading.value = false
}

async function selectPaymentMethod(paymentMethod) {
  if (paymentMethod.id === 'new') {
    await beginPurchaseFlow(false)
  } else if (inputtedPaymentMethod.value && inputtedPaymentMethod.value.id === paymentMethod.id) {
    paymentLoading.value = true
    await refreshPayment(confirmationToken.value)
    paymentLoading.value = false
  } else {
    paymentLoading.value = true
    await refreshPayment(null, paymentMethod.id)
    paymentLoading.value = false
  }
}

async function refreshPayment(confirmationId, paymentMethodId) {
  try {
    const base = confirmationId
      ? {
          type: 'confirmation_token',
          token: confirmationId,
        }
      : {
          type: 'payment_method',
          id: paymentMethodId,
        }

    const result = await props.sendBillingRequest({
      charge: {
        type: 'new',
        product_id: props.product.id,
        interval: selectedPlan.value,
      },
      existing_payment_intent: paymentIntentId.value,
      ...base,
    })

    if (!paymentIntentId.value) {
      paymentIntentId.value = result.payment_intent_id
      clientSecret.value = result.client_secret
    }

    price.value = props.product.prices.find((x) => x.id === result.price_id)
    currency.value = price.value.currency_code
    tax.value = result.tax
    total.value = result.total

    if (confirmationId) {
      confirmationToken.value = confirmationId
      inputtedPaymentMethod.value = result.payment_method
    }

    selectedPaymentMethod.value = result.payment_method
  } catch (err) {
    props.onError(err)
  }
}

async function submitPayment() {
  paymentLoading.value = true
  const { error } = await stripe.confirmPayment({
    clientSecret: clientSecret.value,
    confirmParams: {
      confirmation_token: confirmationToken.value,
      return_url: `${props.returnUrl}?priceId=${price.value.id}&plan=${selectedPlan.value}`,
    },
  })

  if (error) {
    props.onError(error)
  }
  paymentLoading.value = false
}
</script>
<style scoped lang="scss"></style>
