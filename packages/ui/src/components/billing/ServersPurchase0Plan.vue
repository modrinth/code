<script setup lang="ts">
import { computed } from 'vue'
import { useVIntl, defineMessages } from '@vintl/vintl'
import ServersSpecs from './ServersSpecs.vue'
import { monthsInInterval, type ServerBillingInterval, type ServerPlan } from '../../utils/billing'
import { formatPrice } from '@modrinth/utils'
import { DropdownIcon } from '@modrinth/assets'
import { Menu } from 'floating-vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import OptionGroup from '../base/OptionGroup.vue'

const { formatMessage, locale } = useVIntl()

const props = defineProps<{
  availableProducts: ServerPlan[]
  currency: string
}>()

const availableBillingIntervals = ['monthly', 'quarterly']

const selectedPlan = defineModel<ServerPlan>('plan')
const selectedInterval = defineModel<ServerBillingInterval>('interval')
const emit = defineEmits<{
  (e: 'choose-custom'): void
}>()

const messages = defineMessages({
  title: {
    id: 'servers.purchase.step.plan.prompt',
    defaultMessage: 'Choose a plan',
  },
  subtitle: {
    id: 'servers.purchase.step.plan.subtitle',
    defaultMessage: 'Pick the amount of RAM and specs that fit your needs.',
  },
  selectPlan: {
    id: 'servers.purchase.step.plan.select',
    defaultMessage: 'Select Plan',
  },
  getStarted: {
    id: 'servers.purchase.step.plan.get-started',
    defaultMessage: 'Get started',
  },
  billed: {
    id: 'servers.purchase.step.plan.billed',
    defaultMessage: 'billed {interval}',
  },
  smallDesc: {
    id: 'servers.purchase.step.plan.small.desc',
    defaultMessage: 'Perfect for 1–5 friends with a few light mods.',
  },
  mediumDesc: {
    id: 'servers.purchase.step.plan.medium.desc',
    defaultMessage: 'Great for 6–15 players and multiple mods.',
  },
  largeDesc: {
    id: 'servers.purchase.step.plan.large.desc',
    defaultMessage: 'Ideal for 15–25 players, modpacks, or heavy modding.',
  },
  customDesc: {
    id: 'servers.purchase.step.plan.custom.desc',
    defaultMessage: 'Pick your own RAM and storage options.',
  },
  mostPopular: {
    id: 'servers.purchase.step.plan.most-popular',
    defaultMessage: 'Most Popular',
  },
})

const sortedPlans = computed(() => {
  return props.availableProducts
    .slice()
    .sort((a, b) => (a.metadata?.ram ?? 0) - (b.metadata?.ram ?? 0))
})

function planName(plan: ServerPlan): string {
  if (!plan?.metadata || plan.metadata.type !== 'pyro') return 'Custom'
  const ram = plan.metadata.ram
  if (ram === 4096) return 'Small'
  if (ram === 6144) return 'Medium'
  if (ram === 8192) return 'Large'
  return 'Custom'
}

function select(plan: ServerPlan) {
  selectedPlan.value = plan
}

const plansByRam = computed(() => {
  const byName: Record<'small' | 'medium' | 'large', ServerPlan | undefined> = {
    small: undefined,
    medium: undefined,
    large: undefined,
  }
  for (const p of props.availableProducts) {
    if (p?.metadata?.type !== 'pyro') continue
    if (p.metadata.ram === 4096) byName.small = p
    else if (p.metadata.ram === 6144) byName.medium = p
    else if (p.metadata.ram === 8192) byName.large = p
  }
  return byName
})

function pricePerMonth(plan?: ServerPlan) {
  if (!plan) return undefined
  const total = plan.prices?.find((x) => x.currency_code === props.currency)?.prices?.intervals?.[
    selectedInterval.value!
  ]
  if (!total) return undefined
  return total / monthsInInterval[selectedInterval.value!]
}

const customPricePerGb = computed(() => {
  // Calculate lowest price per GB among products for current interval
  let min: number | undefined
  for (const p of props.availableProducts) {
    const perMonth = pricePerMonth(p)
    const ramGb = (p?.metadata?.ram ?? 0) / 1024
    if (perMonth && ramGb > 0) {
      const perGb = perMonth / ramGb
      if (min === undefined || perGb < min) min = perGb
    }
  }
  return min
})

const customStartingPrice = computed(() => {
  let min: number | undefined
  for (const p of props.availableProducts) {
    const perMonth = pricePerMonth(p)
    if (perMonth && (min === undefined || perMonth < min)) min = perMonth
  }
  return min
})
</script>

<template>
  <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-3 mb-2">
    <span></span>
    <OptionGroup
      v-slot="{ option }"
      class="!bg-bg"
      v-model="selectedInterval"
      :options="availableBillingIntervals"
    >
      <template v-if="option === 'monthly'"> Pay monthly </template>
      <span v-else-if="option === 'quarterly'"> Pay quarterly </span>
      <span v-else-if="option === 'yearly'"> Pay yearly </span>
    </OptionGroup>
  </div>
  <div class="grid grid-cols-1 place-items-center mb-2">
    <span class="bg-transparent p-0 text-sm font-medium text-brand">
      Save 16% with quarterly billing!
    </span>
  </div>
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
    <div
      class="!bg-bg card !p-4"
      v-if="plansByRam.small"
      :class="{
        '!border-brand !border-[2px] border-solid': selectedPlan?.id === plansByRam.small.id,
      }"
    >
      <div class="flex flex-col gap-2">
        <span class="text-2xl font-semibold text-contrast">Small</span>
        <span class="m-0 text-lg font-bold text-contrast">
          {{ formatPrice(locale, pricePerMonth(plansByRam.small), currency, true) }}
          <span class="text-sm font-semibold text-secondary">
            / month<template v-if="selectedInterval !== 'monthly'"
              >, billed {{ selectedInterval }}</template
            >
          </span>
        </span>
        <span class="text-sm">{{ formatMessage(messages.smallDesc) }}</span>
        <ButtonStyled size="large">
          <button class="!w-full" @click="selectedPlan = plansByRam.small">Select plan</button>
        </ButtonStyled>
        <Menu placement="bottom-start" :distance="8">
          <template #default="{ shown }">
            <div>
              <span class="flex justify-between text-sm">
                View plan details
                <DropdownIcon
                  class="ml-auto my-auto size-4 transition-transform duration-300 shrink-0"
                  :class="{ 'rotate-180': shown }"
                />
              </span>
            </div>
          </template>

          <template #popper>
            <div class="w-72 rounded-md border border-contrast/10 bg-bg p-3 shadow-lg">
              <ServersSpecs
                :ram="plansByRam.small.metadata.ram!"
                :storage="plansByRam.small.metadata.storage!"
                :cpus="plansByRam.small.metadata.cpu!"
              />
            </div>
          </template>
        </Menu>
      </div>
    </div>
    <div class="!bg-bg card !p-4" v-if="customStartingPrice">
      <div class="flex flex-col gap-3">
        <span class="text-2xl font-semibold text-contrast">Custom</span>
        <span class="m-0 text-lg font-bold text-contrast">
          {{ formatPrice(locale, customStartingPrice, currency, true) }}
          <span class="text-sm font-semibold text-secondary">
            / month<template v-if="interval !== 'monthly'">, billed {{ interval }}</template>
          </span>
        </span>
        <span class="text-sm mb-2">{{ formatMessage(messages.customDesc) }}</span>
        <ButtonStyled size="large">
          <button class="!w-full" @click="handleConfigureCustomPlan">Configure</button>
        </ButtonStyled>
        <div class="flex items-center gap-3">
          <span v-if="customPricePerGb" class="text-sm text-secondary">
            From {{ formatPrice(locale, customPricePerGb, currency, true) }} / GB
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
details > summary {
  list-style: none;
}
details > summary::-webkit-details-marker {
  display: none;
}
</style>
