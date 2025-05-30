<script setup lang="ts">
import { ref, computed, useTemplateRef, nextTick } from 'vue'
import NewModal from '../modal/NewModal.vue'
import { type MessageDescriptor, useVIntl, defineMessage } from '@vintl/vintl'
import { ChevronRightIcon, LeftArrowIcon, RightArrowIcon, XIcon, CheckCircleIcon } from '@modrinth/assets'
import type {
  CreatePaymentIntentRequest,
  CreatePaymentIntentResponse,
  ServerBillingInterval,
  ServerPlan,
  ServerRegion,
  ServerStockRequest,
  UpdatePaymentIntentRequest,
  UpdatePaymentIntentResponse,
} from '../../utils/billing'
import { ButtonStyled } from '../index'
import type Stripe from 'stripe'

import { commonMessages } from '../../utils'
import RegionSelector from './ServersPurchase1Region.vue'
import PaymentMethodSelector from './ServersPurchase2PaymentMethod.vue'
import ConfirmPurchase from './ServersPurchase3Review.vue'
import { useStripe } from '../../composables/stripe'

const { formatMessage } = useVIntl()

export type RegionPing = {
  region: string
  ping: number
}

const props = defineProps<{
  publishableKey: string
  returnUrl: string
  paymentMethods: Stripe.PaymentMethod[]
  customer: Stripe.Customer
  currency: string
  pings: RegionPing[]
  regions: ServerRegion[]
  availableProducts: ServerPlan[]
  refreshPaymentMethods: () => Promise<void>
  fetchStock: (region: ServerRegion, request: ServerStockRequest) => Promise<number>
  initiatePayment: (
    body: CreatePaymentIntentRequest | UpdatePaymentIntentRequest,
  ) => Promise<UpdatePaymentIntentResponse | CreatePaymentIntentResponse>
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const selectedPlan = ref<ServerPlan>()
const selectedInterval = ref<ServerBillingInterval>()
const loading = ref(false)

const {
  initializeStripe,
  selectPaymentMethod,
  primaryPaymentMethodId,
  loadStripeElements,
  selectedPaymentMethod,
  inputtedPaymentMethod,
  createNewPaymentMethod,
  loadingElements,
  loadingElementsFailed,
  tax,
  total,
  paymentMethodLoading,
  reloadPaymentIntent,
  hasPaymentMethod,
  submitPayment
} = useStripe(
  props.publishableKey,
  props.customer,
  props.paymentMethods,
  props.clientSecret,
  props.currency,
  selectedPlan,
  selectedInterval,
  props.initiatePayment,
  console.error,
)

const selectedRegion = ref<string>()
const customServer = ref<boolean>(false)
const acceptedEula = ref<boolean>(false)

type Step = 'region' | 'payment' | 'review'

const steps: Step[] = ['region', 'payment', 'review']

const titles: Record<Step, MessageDescriptor> = {
  region: defineMessage({ id: 'servers.purchase.step.region.title', defaultMessage: 'Region' }),
  payment: defineMessage({
    id: 'servers.purchase.step.payment.title',
    defaultMessage: 'Payment method',
  }),
  review: defineMessage({ id: 'servers.purchase.step.review.title', defaultMessage: 'Review' }),
}

const currentRegion = computed(() => {
  return props.regions.find((region) => region.shortcode === selectedRegion.value)
})

const currentPing = computed(() => {
  return props.pings.find((ping) => ping.region === currentRegion.value?.shortcode)?.ping
})

const currentStep = ref<Step>()

const currentStepIndex = computed(() => steps.indexOf(currentStep.value))
const previousStep = computed(() => steps[steps.indexOf(currentStep.value) - 1])
const nextStep = computed(() => steps[steps.indexOf(currentStep.value) + 1])

const canProceed = computed(() => {
  switch (currentStep.value) {
    case 'region':
      return selectedRegion.value && selectedPlan.value && selectedInterval.value
    case 'payment':
      return selectedPaymentMethod.value || !loadingElements.value
    case 'review':
      return acceptedEula.value && hasPaymentMethod.value
    default:
      return false
  }
})

async function beforeProceed(step: string) {
  switch (step) {
    case 'region':
      return true
    case 'payment':
      await initializeStripe()

      if (primaryPaymentMethodId.value) {
        const paymentMethod = await props.paymentMethods.find((x) => x.id === primaryPaymentMethodId.value)
        await selectPaymentMethod(paymentMethod)
        await setStep('review', true)
        return true
      }
      return true
    case 'review':
      if (selectedPaymentMethod.value) {
        return true
      } else {
        const token = await createNewPaymentMethod()
        return !!token
      }
  }
}

async function afterProceed(step: string) {
  switch (step) {
    case 'region':
      break
    case 'payment':
      await loadStripeElements()
      break
    case 'review':
      break
  }
}

async function setStep(step: Step, skipValidation = false) {
  if (!step) {
    await submitPayment(props.returnUrl)
    return
  }

  if (!canProceed.value || skipValidation) {
    return
  }

  if (await beforeProceed(step)) {
    currentStep.value = step
    await nextTick()

    await afterProceed(step)
  }
}

function begin(interval: ServerBillingInterval, plan?: ServerPlan) {
  loading.value = false
  selectedPlan.value = plan
  selectedInterval.value = interval
  customServer.value = !selectedPlan.value
  selectedPaymentMethod.value = undefined
  currentStep.value = steps[0]
  modal.value?.show()
}

defineExpose({
  show: begin,
})
</script>
<template>
  <NewModal ref="modal">
    <template #title>
      <div class="flex items-center gap-1 font-bold text-secondary">
        <template v-for="(title, id, index) in titles" :key="id">
          <button
            v-if="index < currentStepIndex"
            class="bg-transparent active:scale-95 font-bold text-secondary p-0"
            @click="setStep(id)"
          >
            {{ formatMessage(title) }}
          </button>
          <span
            v-else
            :class="{
              'text-contrast': index === currentStepIndex,
            }"
          >
            {{ formatMessage(title) }}
          </span>
          <ChevronRightIcon
            v-if="index < steps.length - 1"
            class="h-5 w-5 text-secondary"
            stroke-width="3"
          />
        </template>
      </div>
    </template>
    <div class="w-[40rem] max-w-full">
      <RegionSelector
        v-if="currentStep === 'region'"
        v-model:region="selectedRegion"
        v-model:plan="selectedPlan"
        :regions="regions"
        :pings="pings"
        :custom="customServer"
        :available-products="availableProducts"
        :fetch-stock="fetchStock"
      />
      <PaymentMethodSelector
        v-else-if="currentStep === 'payment' && selectedPlan && selectedInterval"
        :payment-methods="paymentMethods"
        :selected="selectedPaymentMethod"
        :loading-elements="loadingElements"
        :loading-elements-failed="loadingElementsFailed"
        @select="selectPaymentMethod"
      />
      <ConfirmPurchase
        v-else-if="
          currentStep === 'review' &&
          hasPaymentMethod &&
          selectedRegion &&
          selectedInterval &&
          selectedPlan
        "
        ref="currentStepRef"
        v-model:interval="selectedInterval"
        v-model:accepted-eula="acceptedEula"
        :currency="currency"
        :plan="selectedPlan"
        :region="regions.find((x) => x.shortcode === selectedRegion)"
        :ping="currentPing"
        :loading="paymentMethodLoading"
        :selected-payment-method="selectedPaymentMethod || inputtedPaymentMethod"
        :tax="tax"
        :total="total"
        :on-error="console.error"
        @change-payment-method="setStep('payment')"
        @reload-payment-intent="reloadPaymentIntent"
        @error="console.error"
      />
      <div v-else>Something went wrong</div>
    </div>
    <div class="flex gap-2 justify-between mt-4">
      <ButtonStyled>
        <button v-if="previousStep" @click="previousStep && setStep(previousStep)">
          <LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
        </button>
        <button v-else @click="modal?.hide()">
          <XIcon />
          {{ formatMessage(commonMessages.cancelButton) }}
        </button>
      </ButtonStyled>
      <ButtonStyled color="brand">
        <button :disabled="!canProceed" @click="setStep(nextStep)">
          <template v-if="currentStep === 'review'">
            <CheckCircleIcon />
            Subscribe
          </template>
          <template v-else>
            {{ formatMessage(commonMessages.nextButton) }} <RightArrowIcon />
          </template>
        </button>
      </ButtonStyled>
    </div>
  </NewModal>
</template>
