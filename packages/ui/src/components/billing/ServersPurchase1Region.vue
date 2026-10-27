<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { InfoIcon, XIcon } from '@modrinth/assets'
import { computed, onMounted, ref, watch } from 'vue'

import { useFormatPrice } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { useRegionPings } from '../../composables/region-pings'
import { getPriceForInterval, monthsInInterval } from '../../utils/product-utils.ts'
import { regionOverrides } from '../../utils/regions.ts'
import IntlFormatted from '../base/IntlFormatted.vue'
import Slider from '../base/Slider.vue'
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'
import type { ServerBillingInterval } from './ModrinthServersPurchaseModal.vue'
import ServersRegionButton from './ServersRegionButton.vue'
import ServersSpecs from './ServersSpecs.vue'

const { formatMessage } = useVIntl()
const formatPrice = useFormatPrice()

const props = defineProps<{
	regions: Archon.Servers.v1.Region[]
	stock?: Archon.Servers.v1.StockResponse
	custom: boolean
	hideRegionSelection?: boolean
	currency: string
	interval: ServerBillingInterval
	availableProducts: Labrinth.Billing.Internal.Product[]
}>()

const PING_WAIT_TIMEOUT = 5000

const { regionPings, waitForRegionPings } = useRegionPings()

const loading = ref(true)
const selectedPlan = defineModel<Labrinth.Billing.Internal.Product>('plan')
const selectedRegion = defineModel<string>('region')

const selectedPrice = computed(() => {
	if (!selectedPlan.value) return undefined
	const amount = getPriceForInterval(selectedPlan.value, props.currency, props.interval)
	return amount ? amount / monthsInInterval[props.interval] : undefined
})

const regionOrder = ref<string[]>(Object.keys(regionOverrides))

const sortedRegions = computed(() => {
	return props.regions.slice().sort((a, b) => {
		return regionOrder.value.indexOf(a.shortcode) - regionOrder.value.indexOf(b.shortcode)
	})
})

function successfulPing(region: string): number {
	const ping = regionPings.value[region]
	return ping !== undefined && ping > 0 ? ping : Infinity
}

/**
 * Orders the regions by ping once, so the cards don't move around while late pings come in.
 */
function sortRegionsByPing() {
	regionOrder.value = sortedRegions.value
		.slice()
		.sort((a, b) => {
			const pingA = successfulPing(a.shortcode)
			const pingB = successfulPing(b.shortcode)
			return pingA === pingB ? 0 : pingA - pingB
		})
		.map((region) => region.shortcode)
}

const visibleRegions = computed(() => {
	if (!loading.value) {
		return sortedRegions.value.filter((region) => {
			// only show eu-lim if it has stock because we're not restocking it.
			if (region.shortcode === 'eu-lim') {
				return currentStock.value[region.shortcode] > 0
			}
			return true
		})
	}
	return sortedRegions.value
})

const selectedRam = ref<number>(-1)

function productRam(product: Labrinth.Billing.Internal.Product): number {
	const metadata = product.metadata
	return metadata.type === 'pyro' || metadata.type === 'medal' ? metadata.ram : 0
}

function regionStock(region: string, ramMb: number): number {
	return props.stock?.regions.find((entry) => entry.region === region)?.stock[ramMb]?.available ?? 0
}

const ramOptions = computed(() => {
	return props.availableProducts.map((product) => productRam(product) / 1024).filter((x) => x > 0)
})

const minRam = computed(() => {
	return Math.min(...ramOptions.value)
})
const maxRam = computed(() => {
	return Math.max(...ramOptions.value)
})

const lowestProduct = computed(() => {
	return (
		props.availableProducts.find((product) => productRam(product) / 1024 === minRam.value) ??
		props.availableProducts[0]
	)
})

const selectedPlanSpecs = computed(() => {
	if (!selectedPlan.value) return null
	const metadata = selectedPlan.value.metadata
	if (metadata.type === 'pyro' || metadata.type === 'medal') {
		return {
			ram: metadata.ram,
			storage: metadata.storage,
			cpu: metadata.cpu,
		}
	}
	return null
})

function updateRamStock(regionToCheck: string, newRam: number) {
	if (newRam <= 0) return
	const plan = props.availableProducts.find((product) => productRam(product) === newRam * 1024)
	if (plan) {
		selectedPlan.value = regionStock(regionToCheck, productRam(plan)) > 0 ? plan : undefined
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

const currentStock = computed<Record<string, number>>(() => {
	const product = props.custom ? lowestProduct.value : (selectedPlan.value ?? lowestProduct.value)
	const ramMb = product ? productRam(product) : 0
	return Object.fromEntries(
		props.regions.map((region) => [region.shortcode, regionStock(region.shortcode, ramMb)]),
	)
})

const bestPing = computed(() => {
	const best = props.regions.reduce<Archon.Servers.v1.Region | undefined>(
		(best, region) =>
			successfulPing(region.shortcode) < (best ? successfulPing(best.shortcode) : Infinity)
				? region
				: best,
		undefined,
	)
	return best?.shortcode
})

const messages = defineMessages({
	prompt: {
		id: 'servers.region.prompt',
		defaultMessage: 'Where would you like your server to be located?',
	},
	regionUnsupported: {
		id: 'servers.region.region-unsupported',
		defaultMessage: `Region not listed? <link>Let us know where you'd like to see Modrinth Hosting next!</link>`,
	},
	customPrompt: {
		id: 'servers.region.custom.prompt',
		defaultMessage: `How much RAM do you want your server to have?`,
	},
	customPromptRamOnly: {
		id: 'servers.region.custom.prompt-ram-only',
		defaultMessage: `RAM`,
	},
	billedInterval: {
		id: 'servers.purchase.step.plan.billed',
		defaultMessage:
			'billed {interval, select, monthly {monthly} quarterly {quarterly} yearly {yearly} other {{interval}}}',
	},
})

/**
 * Selects the first region with stock, which is the one with the best ping once the regions are sorted by ping.
 */
function selectInitialRegion() {
	const stockedRegion =
		selectedRegion.value ??
		sortedRegions.value.find((region) => currentStock.value[region.shortcode] > 0)?.shortcode
	selectedRegion.value = stockedRegion
	if (props.custom && stockedRegion) {
		updateRamStock(stockedRegion, minRam.value)
	}
}

function whenStockLoaded(): Promise<void> {
	if (props.stock) return Promise.resolve()
	return new Promise((resolve) => {
		watch(
			() => props.stock,
			() => resolve(),
			{ once: true },
		)
	})
}

onMounted(async () => {
	selectedRam.value = minRam.value
	if (!props.hideRegionSelection) {
		selectedRegion.value = undefined
	}
	await whenStockLoaded()
	if (!props.hideRegionSelection) {
		await waitForRegionPings(PING_WAIT_TIMEOUT)
		sortRegionsByPing()
	}
	selectInitialRegion()
	loading.value = false
})
</script>

<template>
	<ModalLoadingIndicator v-if="loading" class="flex py-40 justify-center">
		Checking availability...
	</ModalLoadingIndicator>
	<template v-else>
		<template v-if="!hideRegionSelection">
			<h2 class="mt-0 mb-4 text-xl font-bold text-contrast">
				{{ formatMessage(messages.prompt) }}
			</h2>
			<div class="grid grid-cols-1 sm:grid-cols-2 auto-rows-fr gap-4">
				<ServersRegionButton
					v-for="region in visibleRegions"
					:key="region.shortcode"
					v-model="selectedRegion"
					:region="region"
					:out-of-stock="currentStock[region.shortcode] === 0"
					:ping="regionPings[region.shortcode]"
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
		</template>
		<template v-if="custom">
			<h2
				class="mb-2 text-xl font-bold text-contrast"
				:class="hideRegionSelection ? 'mt-0' : 'mt-4'"
			>
				{{
					formatMessage(hideRegionSelection ? messages.customPromptRamOnly : messages.customPrompt)
				}}
			</h2>
			<div>
				<Slider v-model="selectedRam" :min="minRam" :max="maxRam" :step="2" unit="GB" />
				<p v-if="selectedPrice" class="mt-2 mb-0">
					<span class="text-contrast text-lg font-bold"
						>{{ formatPrice(selectedPrice, currency, true) }} / month</span
					><span v-if="interval !== 'monthly'"
						>, {{ formatMessage(messages.billedInterval, { interval }) }}</span
					>
				</p>
				<div class="bg-bg rounded-xl p-4 mt-2 text-secondary h-14">
					<div v-if="selectedPlanSpecs">
						<ServersSpecs
							class="!flex-row justify-between"
							:ram="selectedPlanSpecs.ram"
							:storage="selectedPlanSpecs.storage"
							:cpus="selectedPlanSpecs.cpu"
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
