<template>
  <NewModal ref="purchaseModal">
    <template #title>
      <span class="text-contrast text-xl font-extrabold">
        <template v-if="productType === 'midas'">Subscribe to Modrinth+!</template>
        <template v-else-if="productType === 'pyro'">
          <template v-if="existingSubscription"> Upgrade server plan </template>
          <template v-else> Subscribe to Modrinth Servers! </template>
        </template>
        <template v-else>Purchase product</template>
      </span>
    </template>
    <div class="flex items-center gap-1 pb-4">
      <template v-if="productType === 'pyro' && !projectId">
        <span
          :class="{
            'text-secondary': purchaseModalStep !== 0,
            'font-bold': purchaseModalStep === 0,
          }"
        >
          Configure
          <span class="hidden sm:inline">server</span>
        </span>
        <ChevronRightIcon class="h-5 w-5 text-secondary" />
      </template>
      <span
        :class="{
          'text-secondary': purchaseModalStep !== (productType === 'pyro' && !projectId ? 1 : 0),
          'font-bold': purchaseModalStep === (productType === 'pyro' && !projectId ? 1 : 0),
        }"
      >
        {{ productType === 'pyro' ? 'Billing' : 'Plan' }}
        <span class="hidden sm:inline">{{
          productType === 'pyro' ? 'interval' : 'selection'
        }}</span>
      </span>
      <ChevronRightIcon class="h-5 w-5 text-secondary" />
      <span
        :class="{
          'text-secondary': purchaseModalStep !== (productType === 'pyro' && !projectId ? 2 : 1),
          'font-bold': purchaseModalStep === (productType === 'pyro' && !projectId ? 2 : 1),
        }"
      >
        Payment
      </span>
      <ChevronRightIcon class="h-5 w-5 text-secondary" />
      <span
        :class="{
          'text-secondary': purchaseModalStep !== (productType === 'pyro' && !projectId ? 3 : 2),
          'font-bold': purchaseModalStep === (productType === 'pyro' && !projectId ? 3 : 2),
        }"
      >
        Review
      </span>
    </div>
    <div
      v-if="productType === 'pyro' && !projectId && purchaseModalStep === 0"
      class="md:w-[600px] flex flex-col gap-4"
    >
      <div v-if="!existingSubscription">
        <p class="my-2 text-lg font-bold">Configure your server</p>
        <div class="flex flex-col gap-4">
          <input v-model="serverName" placeholder="Server name" class="input" maxlength="48" />
          <!-- <DropdownSelect
            v-model="serverLoader"
            v-tooltip="'Select the mod loader for your server'"
            name="server-loader"
            :options="['Vanilla', 'Fabric', 'Forge']"
            placeholder="Select mod loader..."
          /> -->
          <div class="grid lg:grid-cols-5 grid-cols-3 gap-4">
            <button
              v-for="loader in [
                'Vanilla',
                'Fabric',
                'Forge',
                'Quilt',
                'NeoForge',
                'Paper',
                'Purpur',
              ]"
              :key="loader"
              class="!h-24 btn flex !flex-col !items-center !justify-between !pt-4 !pb-3 !w-full"
              :style="{
                filter: serverLoader === loader ? 'brightness(1.5)' : '',
              }"
              @click="serverLoader = loader"
            >
              <UiServersIconsLoaderIcon :loader="loader" class="!h-12 !w-12" />
              <p class="text-lg font-bold m-0">{{ loader }}</p>
            </button>
          </div>
          <div class="flex items-center gap-2">
            <InfoIcon class="hidden sm:block" />
            <span class="text-sm text-secondary">
              You can change these settings later in your server options.
            </span>
          </div>
        </div>
      </div>
      <div v-if="customServer">
        <div class="flex gap-2 items-center">
          <p class="my-2 text-lg font-bold">
            <template v-if="existingSubscription">Upgrade your RAM</template>
            <template v-else>Configure your RAM</template>
          </p>
          <IssuesIcon
            v-if="customServerConfig.ramInGb < 4"
            v-tooltip="'This might not be powerful enough for your Minecraft server.'"
            class="h-6 w-6 text-orange"
          />
        </div>
        <p v-if="existingPlan" class="mt-1 mb-2 text-secondary">
          Your current plan has <strong>{{ existingPlan.metadata.ram / 1024 }} GB RAM</strong> and
          <strong
            >{{ existingPlan.metadata.cpu / 2 }} shared CPUs (bursts up to
            {{ existingPlan.metadata.cpu }} CPUs)</strong
          >.
        </p>
        <div class="flex flex-col gap-4">
          <div class="flex w-full gap-2 items-center">
            <Slider
              v-model="customServerConfig.ramInGb"
              class="fix-slider"
              :min="customMinRam"
              :max="customMaxRam"
              :step="2"
              unit="GB"
            />
            <div class="font-semibold text-nowrap"></div>
          </div>
          <div
            v-if="customMatchingProduct && !customOutOfStock"
            class="flex sm:flex-row flex-col gap-4 w-full"
          >
            <div class="flex flex-col w-full gap-2">
              <div class="font-semibold">Shared CPUs</div>
              <input :value="sharedCpus" disabled class="input w-full" />
            </div>
            <div class="flex flex-col w-full gap-2">
              <div class="font-semibold flex items-center gap-1">
                Max Burst CPUs
                <UnknownIcon
                  v-tooltip="
                    'CPU bursting allows your server to temporarily use additional threads to help mitigate TPS spikes. See Modrinth Servers FAQ for more info.'
                  "
                  class="h-4 w-4text-secondary opacity-60"
                />
              </div>
              <input :value="mutatedProduct.metadata.cpu" disabled class="input w-full" />
            </div>
            <div class="flex flex-col w-full gap-2">
              <div class="font-semibold">Storage</div>
              <input
                v-model="customServerConfig.storageGbFormatted"
                disabled
                class="input w-full"
              />
            </div>
          </div>
          <Admonition
            v-else-if="customOutOfStock && customMatchingProduct"
            type="info"
            header="This plan is currently out of stock"
          >
            We are currently
            <a :href="outOfStockUrl" class="underline" target="_blank">out of capacity</a>
            for your selected RAM amount. Please try again later, or try a different amount.
          </Admonition>
          <Admonition v-else type="info" header="We can't seem to find your selected plan">
            We are currently unable to find a server for your selected RAM amount. Please try again
            later, or try a different amount.
          </Admonition>

          <div class="flex gap-2">
            <InfoIcon class="hidden sm:block shrink-0 mt-1" />
            <span class="text-sm text-secondary">
              Storage and shared CPU count are currently not configurable independently, and are
              based on the amount of RAM you select.
            </span>
          </div>
        </div>
      </div>
    </div>
    <div
      v-if="purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 1 : 0)"
      class="md:w-[600px]"
    >
      <div>
        <p class="my-2 text-lg font-bold">Choose billing interval</p>
        <div v-if="existingPlan" class="flex flex-col gap-3 mb-4 text-secondary">
          <p class="m-0">
            The prices below reflect the new <strong>renewal cost</strong> of your upgraded
            subscription.
          </p>
          <p class="m-0">
            Today, you will be charged a prorated amount for the remainder of your current billing
            cycle.
          </p>
        </div>
        <div class="flex flex-col gap-4">
          <div
            v-for="([interval, rawPrice], index) in Object.entries(price.prices.intervals)"
            :key="index"
            class="flex cursor-pointer items-center gap-2"
            @click="selectedPlan = interval"
          >
            <RadioButtonCheckedIcon v-if="selectedPlan === interval" class="h-8 w-8 text-brand" />
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
          <InfoIcon class="hidden sm:block" />
          <span class="text-sm text-secondary">
            Final price and currency will be based on your selected payment method.
          </span>
        </div>
      </div>
    </div>
    <template
      v-if="purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 2 : 1)"
    >
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
    <div
      v-if="purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 3 : 2)"
      class="md:w-[650px]"
    >
      <div
        v-if="mutatedProduct.metadata.type === 'pyro' && !existingSubscription"
        class="r-4 rounded-xl bg-bg p-4 mb-4"
      >
        <p class="my-2 text-lg font-bold text-primary">Server details</p>
        <div class="flex items-center gap-4">
          <img
            v-if="projectImage"
            :src="projectImage"
            alt="Project image"
            class="w-16 h-16 rounded"
          />
          <div>
            <p v-if="projectName" class="font-bold">{{ projectName }}</p>
            <p>Server name: {{ serverName }}</p>
            <p v-if="!projectId">Loader: {{ serverLoader }}</p>
          </div>
        </div>
      </div>
      <div>
        <div class="r-4 rounded-xl bg-bg p-4">
          <p class="my-2 text-lg font-bold text-primary">Purchase details</p>
          <div class="mb-2 flex justify-between">
            <span class="text-secondary">
              {{ mutatedProduct.metadata.type === 'midas' ? 'Modrinth+' : 'Modrinth Servers' }}
              {{
                existingPlan
                  ? `(${dayjs(renewalDate).diff(dayjs(), 'days')} days prorated)`
                  : selectedPlan
              }}
            </span>
            <span v-if="existingPlan" class="text-secondary text-end">
              {{ formatPrice(locale, total - tax, price.currency_code) }}
            </span>
            <span v-else class="text-secondary text-end">
              {{ formatPrice(locale, total - tax, price.currency_code) }} /
              {{ selectedPlan }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-secondary">Tax</span>
            <span class="text-secondary text-end">{{
              formatPrice(locale, tax, price.currency_code)
            }}</span>
          </div>
          <div class="mt-4 flex justify-between border-0 border-t border-solid border-code-bg pt-4">
            <span class="text-lg font-bold">Today's total</span>
            <span class="text-lg font-extrabold text-primary text-end">
              {{ formatPrice(locale, total, price.currency_code) }}
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
          <!-- eslint-disable-next-line vue/no-template-shadow -->
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
          <!-- eslint-disable-next-line vue/no-template-shadow -->
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
        <br />
        You'll be charged
        {{ formatPrice(locale, price.prices.intervals[selectedPlan], price.currency_code) }} /
        {{ selectedPlan }} plus applicable taxes starting
        {{ existingPlan ? dayjs(renewalDate).format('MMMM D, YYYY') : 'today' }}, until you cancel.
        <br />
        You can cancel anytime from your settings page.
      </p>
      <p v-if="mutatedProduct.metadata.type === 'pyro'" class="mb-2 mt-4 text-secondary">
        <Checkbox v-model="eulaAccepted" :disabled="paymentLoading">
          <label>
            I acknowledge that I have read and agree to the
            <a class="underline" target="_blank" href="https://aka.ms/MinecraftEULA">
              Minecraft EULA
            </a>
          </label>
        </Checkbox>
      </p>
    </div>
    <div class="input-group push-right pt-4">
      <template v-if="purchaseModalStep === 0">
        <button class="btn" @click="$refs.purchaseModal.hide()">
          <XIcon />
          Cancel
        </button>
        <button
          class="btn btn-primary"
          :disabled="
            paymentLoading ||
            (mutatedProduct.metadata.type === 'pyro' && !projectId && !serverName) ||
            customNotAllowedToContinue ||
            upgradeNotAllowedToContinue
          "
          @click="nextStep"
        >
          <template v-if="customServer && customLoading">
            <SpinnerIcon class="animate-spin" />
            Checking availability...
          </template>
          <template v-else>
            <RightArrowIcon />
            {{ mutatedProduct.metadata.type === 'pyro' && !projectId ? 'Next' : 'Select' }}
          </template>
        </button>
      </template>
      <template
        v-else-if="
          purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 1 : 0)
        "
      >
        <button
          class="btn"
          @click="
            purchaseModalStep =
              mutatedProduct.metadata.type === 'pyro' && !projectId ? 0 : purchaseModalStep
          "
        >
          Back
        </button>
        <button class="btn btn-primary" :disabled="paymentLoading" @click="beginPurchaseFlow(true)">
          <RightArrowIcon />
          Select
        </button>
      </template>
      <template
        v-else-if="
          purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 2 : 1)
        "
      >
        <button
          class="btn"
          @click="
            () => {
              purchaseModalStep = mutatedProduct.metadata.type === 'pyro' && !projectId ? 1 : 0
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
      <template
        v-else-if="
          purchaseModalStep === (mutatedProduct.metadata.type === 'pyro' && !projectId ? 3 : 2)
        "
      >
        <button class="btn" @click="$refs.purchaseModal.hide()">
          <XIcon />
          Cancel
        </button>
        <button
          v-if="mutatedProduct.metadata.type === 'pyro'"
          class="btn btn-primary"
          :disabled="paymentLoading || !eulaAccepted"
          @click="submitPayment"
        >
          <CheckCircleIcon />
          Subscribe
        </button>
        <!-- Default Subscribe Button, so M+ still works -->
        <button v-else class="btn btn-primary" :disabled="paymentLoading" @click="submitPayment">
          <CheckCircleIcon />
          Subscribe
        </button>
      </template>
    </div>
  </NewModal>
</template>

<script setup>
import { ref, computed, nextTick, reactive, watch } from 'vue'
import NewModal from '../modal/NewModal.vue'
import {
  UnknownIcon,
  SpinnerIcon,
  CardIcon,
  CheckCircleIcon,
  ChevronRightIcon,
  CurrencyIcon,
  InfoIcon,
  IssuesIcon,
  PayPalIcon,
  PlusIcon,
  RadioButtonCheckedIcon,
  RadioButtonIcon,
  RightArrowIcon,
  XIcon,
} from '@modrinth/assets'
import AnimatedLogo from '../brand/AnimatedLogo.vue'
import { getCurrency, calculateSavings, formatPrice, createStripeElements } from '@modrinth/utils'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { Multiselect } from 'vue-multiselect'
import Checkbox from '../base/Checkbox.vue'
import Slider from '../base/Slider.vue'
import dayjs from 'dayjs'
import Admonition from '../base/Admonition.vue'

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
  projectName: {
    type: String,
    default: null,
  },
  projectImage: {
    type: String,
    default: null,
  },
  projectId: {
    type: String,
    default: null,
  },
  versionId: {
    type: String,
    default: null,
  },
  serverName: {
    type: String,
    default: null,
  },
  customServer: {
    type: Boolean,
    required: false,
  },
  fetchCapacityStatuses: {
    type: Function,
    required: false,
    default: null,
  },
  outOfStockUrl: {
    type: String,
    required: false,
    default: '',
  },
  existingSubscription: {
    type: Object,
    required: false,
    default: null,
  },
  existingPlan: {
    type: Object,
    required: false,
    default: null,
  },
  renewalDate: {
    type: String,
    required: false,
    default: null,
  },
})

const productType = computed(() => (props.customServer ? 'pyro' : props.product.metadata.type))

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
const purchaseModalStep = ref(0)
const loadingPaymentMethodModal = ref(0)
const paymentLoading = ref(false)

const selectedPlan = ref('yearly')
const currency = computed(() => getCurrency(props.country))
const price = computed(() => {
  return (
    mutatedProduct.value?.prices?.find((x) => x.currency_code === currency.value) ??
    mutatedProduct.value?.prices?.find((x) => x.currency_code === 'USD') ??
    null
  )
})

const clientSecret = ref()
const paymentIntentId = ref()
const confirmationToken = ref()
const tax = ref()
const total = ref()

const serverName = ref(props.serverName || '')
const serverLoader = ref('Vanilla')
const eulaAccepted = ref(!!props.existingSubscription)

const mutatedProduct = ref({ ...props.product })
const customMinRam = ref(0)
const customMaxRam = ref(0)
const customMatchingProduct = ref()
const customOutOfStock = ref(false)
const customLoading = ref(true)
const customNotAllowedToContinue = computed(
  () =>
    props.customServer &&
    !props.existingSubscription &&
    (!customMatchingProduct.value || customLoading.value || customOutOfStock.value),
)
const upgradeNotAllowedToContinue = computed(
  () => props.existingSubscription && (customOutOfStock.value || customLoading.value),
)

const customServerConfig = reactive({
  ramInGb: 4,
  storageGbFormatted: computed(() => `${mutatedProduct.value.metadata.storage / 1024} GB`),
  ram: computed(() => customServerConfig.ramInGb * 1024),
})

const updateCustomServerProduct = () => {
  customMatchingProduct.value = props.product.find(
    (product) => product.metadata.ram === customServerConfig.ram,
  )

  if (customMatchingProduct.value) {
    mutatedProduct.value = { ...customMatchingProduct.value }
  }
}

let updateCustomServerStockTimeout = null
const updateCustomServerStock = async () => {
  if (updateCustomServerStockTimeout) {
    clearTimeout(updateCustomServerStockTimeout)
    customLoading.value = true
  }

  updateCustomServerStockTimeout = setTimeout(async () => {
    if (props.fetchCapacityStatuses) {
      if (props.existingSubscription) {
        if (mutatedProduct.value) {
          const capacityStatus = await props.fetchCapacityStatuses(
            props.existingSubscription.metadata.id,
            mutatedProduct.value,
          )
          customOutOfStock.value = capacityStatus.custom?.available === 0
          console.log(capacityStatus)
        }
      } else {
        const capacityStatus = await props.fetchCapacityStatuses(mutatedProduct.value)
        customOutOfStock.value = capacityStatus.custom?.available === 0
      }
    } else {
      console.error('No fetchCapacityStatuses function provided.')
      customOutOfStock.value = true
    }
    customLoading.value = false
  }, 300)
}

function updateRamValues() {
  const ramValues = props.product.map((product) => product.metadata.ram / 1024)
  customMinRam.value = Math.min(...ramValues)
  customMaxRam.value = Math.max(...ramValues)

  if (props.product.some((product) => product.metadata.ram / 1024 === 4)) {
    customServerConfig.ramInGb = 4
  } else {
    customServerConfig.ramInGb = customMinRam.value
  }
}

if (props.customServer) {
  updateRamValues()

  const updateProductAndStock = () => {
    updateCustomServerProduct()
    updateCustomServerStock()
  }

  updateProductAndStock()
  watch(() => customServerConfig.ram, updateProductAndStock)
}

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

const metadata = computed(() => {
  if (mutatedProduct.value.metadata.type === 'pyro') {
    return {
      type: 'pyro',
      server_name: serverName.value,
      source:
        props.projectId && props.versionId
          ? {
              project_id: props.projectId,
              version_id: props.versionId,
            }
          : {
              loader: serverLoader.value,
              loader_version: '',
              game_version: 'latest',
            },
    }
  }
  return null
})

const sharedCpus = computed(() => {
  return (mutatedProduct.value?.metadata?.cpu ?? 0) / 2
})

function nextStep() {
  if (
    mutatedProduct.value.metadata.type === 'pyro' &&
    !props.projectId &&
    purchaseModalStep.value === 0
  ) {
    purchaseModalStep.value = 1
  } else {
    beginPurchaseFlow(true)
  }
}

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
    purchaseModalStep.value =
      mutatedProduct.value.metadata.type === 'pyro' && !props.projectId ? 3 : 2
  } else {
    try {
      loadingPaymentMethodModal.value = 0
      purchaseModalStep.value =
        mutatedProduct.value.metadata.type === 'pyro' && !props.projectId ? 2 : 1

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
  purchaseModalStep.value =
    mutatedProduct.value.metadata.type === 'pyro' && !props.projectId ? 3 : 2
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

    const result = await props.sendBillingRequest(
      props.existingSubscription
        ? {
            interval: selectedPlan.value,
            cancelled: false,
            product: mutatedProduct.value.id,
            payment_method: paymentMethodId,
          }
        : {
            charge: {
              type: 'new',
              product_id: mutatedProduct.value.id,
              interval: selectedPlan.value,
            },
            existing_payment_intent: paymentIntentId.value,
            metadata: metadata.value,
            ...base,
          },
    )

    if (!paymentIntentId.value) {
      paymentIntentId.value = result.payment_intent_id
      clientSecret.value = result.client_secret
    }

    price.value = mutatedProduct.value.prices.find((x) => x.id === result.price_id)
    currency.value = price.value.currency_code
    tax.value = result.tax
    total.value = result.total

    if (confirmationId) {
      confirmationToken.value = confirmationId
      if (result.payment_method) {
        inputtedPaymentMethod.value = result.payment_method
      }
    }

    if (result.payment_method) {
      selectedPaymentMethod.value = result.payment_method
    }
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

defineExpose({
  show: () => {
    if (props.customServer) {
      updateRamValues()
    }

    stripe = Stripe(props.publishableKey)

    selectedPlan.value = props.existingSubscription ? props.existingSubscription.interval : 'yearly'
    serverName.value = props.serverName || ''
    serverLoader.value = 'Vanilla'

    purchaseModalStep.value = 0
    loadingPaymentMethodModal.value = 0
    paymentLoading.value = false

    purchaseModal.value.show()
  },
})
</script>
<style scoped lang="scss"></style>
