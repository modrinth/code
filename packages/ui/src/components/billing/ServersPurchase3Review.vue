<script setup lang="ts">
import { computed } from 'vue'
import {
  monthsInInterval,
  type ServerBillingInterval,
  type ServerPlan,
  type ServerRegion,
} from '../../utils/billing'
import TagItem from '../base/TagItem.vue'
import ServersSpecs from './ServersSpecs.vue'
import { formatPrice, getPingLevel } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { regionOverrides } from '../../utils/regions'
import {
  EditIcon,
  RightArrowIcon,
  SignalIcon,
  SpinnerIcon,
  XIcon,
  RadioButtonIcon,
  RadioButtonCheckedIcon,
  ExternalIcon,
} from '@modrinth/assets'
import type Stripe from 'stripe'
import FormattedPaymentMethod from './FormattedPaymentMethod.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import ExpandableInvoiceTotal from './ExpandableInvoiceTotal.vue'

const vintl = useVIntl()
const { locale, formatMessage } = vintl

const emit = defineEmits<{
  (e: 'changePaymentMethod' | 'reloadPaymentIntent'): void
}>()

const props = defineProps<{
  plan: ServerPlan
  region: ServerRegion
  tax?: number
  total?: number
  currency: string
  ping?: number
  loading?: boolean
  selectedPaymentMethod: Stripe.PaymentMethod | undefined
}>()

const interval = defineModel<ServerBillingInterval>('interval', { required: true })
const acceptedEula = defineModel<boolean>('acceptedEula', { required: true })

const prices = computed(() => {
  return props.plan.prices.find((x) => x.currency_code === props.currency)
})

const planName = computed(() => {
  if (!props.plan || !props.plan.metadata || props.plan.metadata.type !== 'pyro') return 'Unknown'
  const ram = props.plan.metadata.ram
  if (ram === 4096) return 'Small'
  if (ram === 6144) return 'Medium'
  if (ram === 8192) return 'Large'
  return 'Custom'
})

const flag = computed(
  () =>
    regionOverrides[props.region.shortcode]?.flag ??
    `https://flagcdn.com/${props.region.country_code}.svg`,
)
const overrideTitle = computed(() => regionOverrides[props.region.shortcode]?.name)
const title = computed(() =>
  overrideTitle.value ? formatMessage(overrideTitle.value) : props.region.display_name,
)
const locationSubtitle = computed(() =>
  overrideTitle.value ? props.region.display_name : undefined,
)
const pingLevel = computed(() => getPingLevel(props.ping ?? 0))

const period = computed(() => {
  if (interval.value === 'monthly') return 'month'
  if (interval.value === 'quarterly') return '3 months'
  if (interval.value === 'yearly') return 'year'
  return '???'
})

function setInterval(newInterval: ServerBillingInterval) {
  interval.value = newInterval
  emit('reloadPaymentIntent')
}
</script>

<template>
  <div class="grid sm:grid-cols-[3fr_2fr] gap-4">
    <div class="bg-table-alternateRow p-4 rounded-2xl">
      <div class="flex items-center gap-2 mb-3">
        <LazyUiServersModrinthServersIcon class="flex h-5 w-fit" />
        <TagItem>{{ planName }}</TagItem>
      </div>
      <div>
        <ServersSpecs
          v-if="plan.metadata && plan.metadata.ram && plan.metadata.storage && plan.metadata.cpu"
          class="!grid sm:grid-cols-2"
          :ram="plan.metadata.ram"
          :storage="plan.metadata.storage"
          :cpus="plan.metadata.cpu"
        />
      </div>
    </div>
    <div
      class="bg-table-alternateRow p-4 rounded-2xl flex flex-col gap-2 items-center justify-center"
    >
      <img
        v-if="flag"
        class="aspect-[16/10] max-w-12 w-full object-cover rounded-md border-1 border-button-border border-solid"
        :src="flag"
        alt=""
        aria-hidden="true"
      />
      <span class="font-semibold">
        {{ title }}
      </span>
      <span class="text-xs flex items-center gap-1 text-secondary font-medium">
        <template v-if="locationSubtitle">
          <span>
            {{ locationSubtitle }}
          </span>
          <span v-if="ping !== -1">•</span>
        </template>
        <template v-if="ping !== -1">
          <SignalIcon
            v-if="ping"
            aria-hidden="true"
            :style="`--_signal-${pingLevel}: ${pingLevel <= 2 ? 'var(--color-red)' : pingLevel <= 4 ? 'var(--color-orange)' : 'var(--color-green)'}`"
            stroke-width="3px"
            class="shrink-0"
          />
          <SpinnerIcon v-else class="animate-spin" />
          <template v-if="ping"> {{ ping }}ms </template>
          <span v-else> Testing connection... </span>
        </template>
      </span>
    </div>
  </div>

  <div class="grid grid-cols-2 gap-2 mt-4">
    <button
      :class="
        interval === 'monthly'
          ? 'bg-button-bg border-transparent'
          : 'bg-transparent  border-button-border'
      "
      class="rounded-2xl active:scale-[0.98] transition-transform duration-100 border-2 border-solid p-4 flex items-center gap-2"
      @click="setInterval('monthly')"
    >
      <RadioButtonCheckedIcon v-if="interval === 'monthly'" class="size-6 text-brand" />
      <RadioButtonIcon v-else class="size-6 text-secondary" />
      <div class="flex flex-col items-start gap-1 font-medium text-primary">
        <span class="flex items-center gap-1" :class="{ 'text-contrast': interval === 'monthly' }"
          >Pay monthly</span
        >
        <span class="text-sm text-secondary flex items-center gap-1"
          >{{ formatPrice(locale, prices?.prices.intervals['monthly'], currency, true) }} /
          month</span
        >
      </div>
    </button>
    <button
      :class="
        interval === 'quarterly'
          ? 'bg-button-bg border-transparent'
          : 'bg-transparent  border-button-border'
      "
      class="rounded-2xl active:scale-[0.98] transition-transform duration-100 border-2 border-solid p-4 flex items-center gap-2"
      @click="setInterval('quarterly')"
    >
      <RadioButtonCheckedIcon v-if="interval === 'quarterly'" class="size-6 text-brand" />
      <RadioButtonIcon v-else class="size-6 text-secondary" />
      <div class="flex flex-col items-start gap-1 font-medium text-primary">
        <span class="flex items-center gap-1" :class="{ 'text-contrast': interval === 'quarterly' }"
          >Pay quarterly
          <span class="text-xs font-bold text-brand px-1.5 py-0.5 rounded-full bg-brand-highlight"
            >{{ interval === 'quarterly' ? 'Saving' : 'Save' }} 16%</span
          ></span
        >
        <span class="text-sm text-secondary flex items-center gap-1"
          >{{
            formatPrice(
              locale,
              (prices?.prices?.intervals?.['quarterly'] ?? 0) / monthsInInterval['quarterly'],
              currency,
              true,
            )
          }}
          / month</span
        >
      </div>
    </button>
  </div>
  <div class="mt-2">
    <ExpandableInvoiceTotal
      :period="period"
      :currency="currency"
      :loading="loading"
      :total="total ?? -1"
      :billing-items="
        total !== undefined && tax !== undefined
          ? [
              {
                title: `Modrinth Servers (${planName})`,
                amount: total - tax,
              },
              {
                title: 'Tax',
                amount: tax,
              },
            ]
          : []
      "
    />
  </div>
  <div class="mt-2 flex items-center pl-4 pr-2 py-3 bg-bg rounded-2xl gap-2 text-secondary">
    <template v-if="selectedPaymentMethod">
      <FormattedPaymentMethod :method="selectedPaymentMethod" />
    </template>
    <template v-else>
      <div class="flex items-center gap-2 text-red">
        <XIcon />
        No payment method selected
      </div>
    </template>
    <ButtonStyled size="small" type="transparent">
      <button class="ml-auto" @click="emit('changePaymentMethod')">
        <template v-if="selectedPaymentMethod"> <EditIcon /> Change </template>
        <template v-else> Select payment method <RightArrowIcon /> </template>
      </button>
    </ButtonStyled>
  </div>
  <p class="m-0 mt-4 text-sm text-secondary">
    <span class="font-semibold"
      >By clicking "Subscribe", you are purchasing a recurring subscription.</span
    >
    <br />
    You'll be charged
    <SpinnerIcon v-if="loading" class="animate-spin relative top-0.5 mx-2" /><template v-else>{{
      formatPrice(locale, total, currency)
    }}</template>
    every {{ period }} plus applicable taxes starting today, until you cancel. You can cancel
    anytime from your settings page.
  </p>
  <div class="mt-2 flex items-center gap-1 text-sm">
    <Checkbox
      v-model="acceptedEula"
      label="I acknowledge that I have read and agree to the"
      description="I acknowledge that I have read and agree to the Minecraft EULA"
    />
    <a
      href="https://www.minecraft.net/en-us/eula"
      target="_blank"
      class="text-brand underline hover:brightness-[--hover-brightness]"
      >Minecraft EULA<ExternalIcon class="size-3 shrink-0 ml-0.5 mb-0.5"
    /></a>
  </div>
</template>
