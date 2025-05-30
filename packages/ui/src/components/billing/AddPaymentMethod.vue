<script setup lang="ts">
import { ref } from 'vue'
import { createStripeElements } from '@modrinth/utils'
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'
import { loadStripe, type Stripe as StripsJs, type StripeElements } from '@stripe/stripe-js'

const emit = defineEmits<{
  (e: 'startLoading' | 'stopLoading'): void
}>()

export type SetupIntentResponse = {
  client_secret: string
}

export type AddPaymentMethodProps = {
  publishableKey: string
  createSetupIntent: () => Promise<SetupIntentResponse>
  returnUrl: string
  onError: (error: Error) => void
}

const props = defineProps<AddPaymentMethodProps>()

const elementsLoaded = ref<0 | 1 | 2>(0)
const stripe = ref<StripsJs>()
const elements = ref<StripeElements>()
const error = ref(false)

function handleError(error: Error) {
  props.onError(error)
  error.value = true
}

async function reload(paymentMethods: Stripe.PaymentMethod[]) {
  try {
    elementsLoaded.value = 0
    error.value = false

    const result = await props.createSetupIntent()

    stripe.value = await loadStripe(props.publishableKey)
    const {
      elements: newElements,
      addressElement,
      paymentElement,
    } = createStripeElements(stripe.value, paymentMethods, {
      clientSecret: result.client_secret,
    })

    elements.value = newElements
    paymentElement.on('ready', () => {
      elementsLoaded.value += 1
    })
    addressElement.on('ready', () => {
      elementsLoaded.value += 1
    })
  } catch (err) {
    handleError(err)
  }
}

async function submit(): Promise<boolean> {
  emit('startLoading')

  const result = await stripe.value.confirmSetup({
    elements: elements.value,
    confirmParams: {
      return_url: props.returnUrl,
    },
  })

  console.log(result)

  const { error } = result

  emit('stopLoading')
  if (error && error.type !== 'validation_error') {
    handleError(error.message)
    return false
  } else if (!error) {
    return true
  }
}

defineExpose({
  reload,
  submit,
})
</script>

<template>
  <div class="min-h-[16rem] flex flex-col gap-2 justify-center items-center">
    <div v-show="elementsLoaded < 2">
      <ModalLoadingIndicator :error="error">
        Loading...
        <template #error> Error loading Stripe payment UI. </template>
      </ModalLoadingIndicator>
    </div>
    <div class="w-full">
      <div id="address-element"></div>
      <div id="payment-element" class="mt-4"></div>
    </div>
  </div>
</template>
