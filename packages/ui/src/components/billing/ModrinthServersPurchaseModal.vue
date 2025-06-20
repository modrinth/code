<script setup lang="ts">
import { ref, computed, useTemplateRef, nextTick, watch } from 'vue'
import NewModal from '../modal/NewModal.vue'
import { type MessageDescriptor, useVIntl, defineMessage } from '@vintl/vintl'
import {
  ChevronRightIcon,
  LeftArrowIcon,
  RightArrowIcon,
  XIcon,
  CheckCircleIcon,
  SpinnerIcon,
} from '@modrinth/assets'
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
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'

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
  onError: (err: Error) => void
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const selectedPlan = ref<ServerPlan>()
const selectedInterval = ref<ServerBillingInterval>('quarterly')
const loading = ref(false)
const selectedRegion = ref<string>()
const projectId = ref<string>()

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
  submitPayment,
  completingPurchase,
} = useStripe(
  props.publishableKey,
  props.customer,
  props.paymentMethods,
  props.currency,
  selectedPlan,
  selectedInterval,
  selectedRegion,
  projectId,
  props.initiatePayment,
  props.onError,
)

const customServer = ref<boolean>(false)
const acceptedEula = ref<boolean>(false)
const skipPaymentMethods = ref<boolean>(true)

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

const currentStepIndex = computed(() => (currentStep.value ? steps.indexOf(currentStep.value) : -1))
const previousStep = computed(() => {
  const step = currentStep.value ? steps[steps.indexOf(currentStep.value) - 1] : undefined
  if (step === 'payment' && skipPaymentMethods.value && primaryPaymentMethodId.value) {
    return 'region'
  }
  return step
})
const nextStep = computed(() =>
  currentStep.value ? steps[steps.indexOf(currentStep.value) + 1] : undefined,
)

const canProceed = computed(() => {
  switch (currentStep.value) {
    case 'region':
      return selectedRegion.value && selectedPlan.value && selectedInterval.value
    case 'payment':
      return selectedPaymentMethod.value || !loadingElements.value
    case 'review':
      return acceptedEula.value && hasPaymentMethod.value && !completingPurchase.value
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

      if (primaryPaymentMethodId.value && skipPaymentMethods.value) {
        const paymentMethod = await props.paymentMethods.find(
          (x) => x.id === primaryPaymentMethodId.value,
        )
        await selectPaymentMethod(paymentMethod)
        await setStep('review', true)
        return false
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

async function setStep(step: Step | undefined, skipValidation = false) {
  if (!step) {
    await submitPayment(props.returnUrl)
    return
  }

  if (!skipValidation && !canProceed.value) {
    return
  }

  if (await beforeProceed(step)) {
    currentStep.value = step
    await nextTick()

    await afterProceed(step)
  }
}

watch(selectedPlan, () => {
  console.log(selectedPlan.value)
})

function begin(interval: ServerBillingInterval, plan?: ServerPlan, project?: string) {
  loading.value = false
  selectedPlan.value = plan
  selectedInterval.value = interval
  customServer.value = !selectedPlan.value
  selectedPaymentMethod.value = undefined
  currentStep.value = steps[0]
  skipPaymentMethods.value = true
  projectId.value = project
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
            @click="setStep(id, true)"
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
        :currency="currency"
        :interval="selectedInterval"
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
          currentRegion &&
          selectedInterval &&
          selectedPlan
        "
        v-model:interval="selectedInterval"
        v-model:accepted-eula="acceptedEula"
        :currency="currency"
        :plan="selectedPlan"
        :region="currentRegion"
        :ping="currentPing"
        :loading="paymentMethodLoading"
        :selected-payment-method="selectedPaymentMethod || inputtedPaymentMethod"
        :tax="tax"
        :total="total"
        @change-payment-method="
          () => {
            skipPaymentMethods = false
            setStep('payment', true)
          }
        "
        @reload-payment-intent="reloadPaymentIntent"
      />
      <div v-else>Something went wrong</div>
      <div
        v-show="
          selectedPaymentMethod === undefined &&
          currentStep === 'payment' &&
          selectedPlan &&
          selectedInterval
        "
        class="min-h-[16rem] flex flex-col gap-2 mt-2 p-4 bg-table-alternateRow rounded-xl justify-center items-center"
      >
        <div v-show="loadingElements">
          <ModalLoadingIndicator :error="loadingElementsFailed">
            Loading...
            <template #error> Error loading Stripe payment UI. </template>
          </ModalLoadingIndicator>
        </div>
        <div class="w-full">
          <div id="address-element"></div>
          <div id="payment-element" class="mt-4"></div>
        </div>
      </div>
    </div>
    <div class="flex gap-2 justify-between mt-4">
      <ButtonStyled>
        <button v-if="previousStep" @click="previousStep && setStep(previousStep, true)">
          <LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
        </button>
        <button v-else @click="modal?.hide()">
          <XIcon />
          {{ formatMessage(commonMessages.cancelButton) }}
        </button>
      </ButtonStyled>
      <ButtonStyled color="brand">
        <button
          v-tooltip="
            currentStep === 'review' && !acceptedEula
              ? 'You must accept the Minecraft EULA to proceed.'
              : undefined
          "
          :disabled="!canProceed"
          @click="setStep(nextStep)"
        >
          <template v-if="currentStep === 'review'">
            <SpinnerIcon v-if="completingPurchase" class="animate-spin" />
            <CheckCircleIcon v-else />
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
