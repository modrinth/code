<script setup lang="ts">
import ServersRegionButton from './ServersRegionButton.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { onMounted, ref, computed, watch } from 'vue'
import type { RegionPing } from './ModrinthServersPurchaseModal.vue'
import {
  monthsInInterval,
  type ServerBillingInterval,
  type ServerPlan,
  type ServerRegion,
  type ServerStockRequest,
} from '../../utils/billing'
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'
import Slider from '../base/Slider.vue'
import { SpinnerIcon, XIcon, InfoIcon } from '@modrinth/assets'
import ServersSpecs from './ServersSpecs.vue'
import { formatPrice } from '../../../../utils'

const { formatMessage, locale } = useVIntl()

const props = defineProps<{
  regions: ServerRegion[]
  pings: RegionPing[]
  fetchStock: (region: ServerRegion, request: ServerStockRequest) => Promise<number>
  custom: boolean
  currency: string
  interval: ServerBillingInterval
  availableProducts: ServerPlan[]
}>()

const loading = ref(true)
const checkingCustomStock = ref(false)
const selectedPlan = defineModel<ServerPlan>('plan')
const selectedRegion = defineModel<string>('region')

const selectedPrice = computed(() => {
  const amount = selectedPlan.value?.prices?.find((price) => price.currency_code === props.currency)
    ?.prices?.intervals?.[props.interval]
  return amount ? amount / monthsInInterval[props.interval] : undefined
})

const regionOrder: string[] = ['us-vin', 'eu-lim']

const sortedRegions = computed(() => {
  return props.regions.slice().sort((a, b) => {
    return regionOrder.indexOf(a.shortcode) - regionOrder.indexOf(b.shortcode)
  })
})

const selectedRam = ref<number>(-1)

const ramOptions = computed(() => {
  return props.availableProducts
    .map((product) => (product.metadata.ram ?? 0) / 1024)
    .filter((x) => x > 0)
})

const minRam = computed(() => {
  return Math.min(...ramOptions.value)
})
const maxRam = computed(() => {
  return Math.max(...ramOptions.value)
})

const lowestProduct = computed(() => {
  return (
    props.availableProducts.find(
      (product) => (product.metadata.ram ?? 0) / 1024 === minRam.value,
    ) ?? props.availableProducts[0]
  )
})

function updateRamStock(regionToCheck: string, newRam: number) {
  if (newRam > 0) {
    checkingCustomStock.value = true
    const plan = props.availableProducts.find(
      (product) => (product.metadata.ram ?? 0) / 1024 === newRam,
    )
    if (plan) {
      const region = sortedRegions.value.find((region) => region.shortcode === regionToCheck)
      if (region) {
        props
          .fetchStock(region, {
            cpu: plan.metadata.cpu ?? 0,
            memory_mb: plan.metadata.ram ?? 0,
            swap_mb: plan.metadata.swap ?? 0,
            storage_mb: plan.metadata.storage ?? 0,
          })
          .then((stock: number) => {
            if (stock > 0) {
              selectedPlan.value = plan
            } else {
              selectedPlan.value = undefined
            }
          })
          .finally(() => {
            checkingCustomStock.value = false
          })
      } else {
        checkingCustomStock.value = false
      }
    }
  }
}

watch(selectedRam, (newRam: number) => {
  if (props.custom && selectedRegion.value) {
    updateRamStock(selectedRegion.value, newRam)
  }
})

watch(selectedRegion, (newRegion: string | undefined) => {
  if (props.custom && newRegion) {
    updateRamStock(newRegion, selectedRam.value)
  }
})

const currentStock = ref<{ [region: string]: number }>({})
const bestPing = ref<string>()

const messages = defineMessages({
  prompt: {
    id: 'servers.region.prompt',
    defaultMessage: 'Where would you like your server to be located?',
  },
  regionUnsupported: {
    id: 'servers.region.region-unsupported',
    defaultMessage: `Region not listed? <link>Let us know where you'd like to see Modrinth Servers next!</link>`,
  },
  customPrompt: {
    id: 'servers.region.custom.prompt',
    defaultMessage: `How much RAM do you want your server to have?`,
  },
})

async function updateStock() {
  currentStock.value = {}
  const capacityChecks = sortedRegions.value.map((region) =>
    props.fetchStock(
      region,
      selectedPlan.value
        ? {
            cpu: selectedPlan.value?.metadata.cpu ?? 0,
            memory_mb: selectedPlan.value?.metadata.ram ?? 0,
            swap_mb: selectedPlan.value?.metadata.swap ?? 0,
            storage_mb: selectedPlan.value?.metadata.storage ?? 0,
          }
        : {
            cpu: lowestProduct.value.metadata.cpu ?? 0,
            memory_mb: lowestProduct.value.metadata.ram ?? 0,
            swap_mb: lowestProduct.value.metadata.swap ?? 0,
            storage_mb: lowestProduct.value.metadata.storage ?? 0,
          },
    ),
  )
  const results = await Promise.all(capacityChecks)
  results.forEach((result, index) => {
    currentStock.value[sortedRegions.value[index].shortcode] = result
  })
}

onMounted(() => {
  // auto select region with lowest ping
  loading.value = true
  bestPing.value =
    props.pings.length > 0
      ? props.pings.reduce((acc, cur) => {
          return acc.ping < cur.ping ? acc : cur
        })?.region
      : undefined
  selectedRegion.value = undefined
  selectedRam.value = minRam.value
  checkingCustomStock.value = true
  updateStock().then(() => {
    const firstWithStock = sortedRegions.value.find(
      (region) => currentStock.value[region.shortcode] > 0,
    )
    let stockedRegion = selectedRegion.value
    if (!stockedRegion) {
      stockedRegion =
        bestPing.value && currentStock.value[bestPing.value] > 0
          ? bestPing.value
          : firstWithStock?.shortcode
    }
    selectedRegion.value = stockedRegion
    if (props.custom && stockedRegion) {
      updateRamStock(stockedRegion, minRam.value)
    }
    loading.value = false
  })
})
</script>

<template>
  <ModalLoadingIndicator v-if="loading" class="flex py-40 justify-center">
    Checking availability...
  </ModalLoadingIndicator>
  <template v-else>
    <h2 class="mt-0 mb-4 text-xl font-bold text-contrast">
      {{ formatMessage(messages.prompt) }}
    </h2>
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <ServersRegionButton
        v-for="region in sortedRegions"
        :key="region.shortcode"
        v-model="selectedRegion"
        :region="region"
        :out-of-stock="currentStock[region.shortcode] === 0"
        :ping="pings.find((p) => p.region === region.shortcode)?.ping"
        :best-ping="bestPing === region.shortcode"
      />
    </div>
    <div class="mt-3 text-sm">
      <IntlFormatted :message-id="messages.regionUnsupported">
        <template #link="{ children }">
          <a
            class="text-link"
            target="_blank"
            rel="noopener noreferrer"
            href="https://surveys.modrinth.com/servers-region-waitlist"
          >
            <component :is="() => children" />
          </a>
        </template>
      </IntlFormatted>
    </div>
    <template v-if="custom">
      <h2 class="mt-4 mb-2 text-xl font-bold text-contrast">
        {{ formatMessage(messages.customPrompt) }}
      </h2>
      <div>
        <Slider v-model="selectedRam" :min="minRam" :max="maxRam" :step="2" unit="GB" />
        <p v-if="selectedPrice" class="mt-2 mb-0">
          <span class="text-contrast text-lg font-bold"
            >{{ formatPrice(locale, selectedPrice, currency, true) }} / month</span
          ><span v-if="interval !== 'monthly'">, billed {{ interval }}</span>
        </p>
        <div class="bg-bg rounded-xl p-4 mt-2 text-secondary">
          <div v-if="checkingCustomStock" class="flex gap-2 items-center">
            <SpinnerIcon class="size-5 shrink-0 animate-spin" /> Checking availability...
          </div>
          <div v-else-if="selectedPlan">
            <ServersSpecs
              class="!flex-row justify-between"
              :ram="selectedPlan.metadata.ram ?? 0"
              :storage="selectedPlan.metadata.storage ?? 0"
              :cpus="selectedPlan.metadata.cpu ?? 0"
            />
          </div>
          <div v-else class="flex gap-2 items-center">
            <XIcon class="size-5 shrink-0 text-red" /> Sorry, we don't have any plans available with
            {{ selectedRam }} GB RAM in this region.
          </div>
        </div>
        <div class="flex gap-2 mt-2">
          <InfoIcon class="hidden sm:block shrink-0 mt-1" />
          <span class="text-sm text-secondary">
            Storage and shared CPU count are currently not configurable independently, and are based
            on the amount of RAM you select.
          </span>
        </div>
      </div>
    </template>
  </template>
</template>
