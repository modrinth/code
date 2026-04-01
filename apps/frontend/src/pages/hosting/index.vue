<template>
	<div class="px-4 pb-6">
		<ModrinthServersPurchaseModal
			v-if="customer"
			:key="`purchase-modal-${customer.id}`"
			ref="purchaseModal"
			:publishable-key="config.public.stripePublishableKey"
			:initiate-payment="
				async (body) => await client.labrinth.billing_internal.initiatePayment(body)
			"
			:available-products="pyroProducts"
			:on-error="handleError"
			:customer="customer"
			:payment-methods="paymentMethods"
			:currency="selectedCurrency"
			:return-url="`${config.public.siteUrl}/hosting/manage`"
			:server-name="`${auth?.user?.username}'s server`"
			:out-of-stock-url="outOfStockUrl"
			:fetch-capacity-statuses="fetchCapacityStatuses"
			:pings="regionPings"
			:regions="regions"
			:refresh-payment-methods="fetchPaymentData"
			:fetch-stock="fetchStock"
			:affiliate-code="affiliateCode"
		/>

		<div class="mx-auto">
			<h1 class="mb-2 mt-6 text-3xl font-normal">Modrinth Hosting</h1>
			<p class="m-0 text-secondary">
				Spin up a server in minutes. Install modpacks from Modrinth, manage files in your browser,
				and play with friends—no separate panel login.
			</p>

			<div class="mt-4 flex flex-wrap gap-3">
				<ButtonStyled color="brand" size="large">
					<nuxt-link class="flex w-fit items-center gap-2" to="#plans">
						{{ hasServers ? 'Start a new server' : 'Start your server' }}
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled v-if="hasServers" type="outlined" size="large">
					<nuxt-link class="flex w-fit items-center gap-2" to="/hosting/manage">
						<SettingsIcon aria-hidden="true" class="size-5 shrink-0" />
						Manage your servers
					</nuxt-link>
				</ButtonStyled>
			</div>

			<div class="mt-8 border border-solid border-[#96CEE0] bg-[#E6F1F5] p-4">
				<p class="m-0 text-lg font-bold">Every plan includes</p>
				<ul class="mb-0 mt-2 list-disc space-y-1 pl-6">
					<li>One-click installs for mods and modpacks from Modrinth</li>
					<li>A custom modrinth.gg domain name for your server</li>
					<li>Access your files manually whenever you wish with SFTP</li>
					<li>
						Access your server's console, installed content, files, and settings all from the
						Modrinth web page.
					</li>
					<li>Up to 15 off-site backups stored at a time</li>
					<li>High-performance AMD CPUs</li>
					<li>Access to our dedicated support team that can help you if you run into issues.</li>
				</ul>
			</div>

			<div
				id="plans"
				class="mt-8 scroll-mt-4 border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-4"
			>
				<h2 class="m-0 text-lg font-bold">Plans & pricing</h2>
				<p class="mb-4 mt-2 text-secondary">
					Available in North America, Europe, and Southeast Asia for wide coverage.
				</p>

				<OptionGroup v-slot="{ option }" v-model="billingPeriod" :options="billingPeriods">
					<template v-if="option === 'monthly'">Pay monthly</template>
					<span v-else-if="option === 'quarterly'">Pay quarterly</span>
					<span v-else-if="option === 'yearly'">Pay yearly</span>
				</OptionGroup>
				<p v-if="billingPeriods.includes('quarterly')">Save 16% with quarterly billing!</p>

				<table
					class="mt-6 w-full border-collapse border border-solid border-[#c3c3c3] bg-[#fafafa] text-left text-sm"
				>
					<thead>
						<tr class="bg-[#e8e8e8]">
							<th class="border border-solid border-[#c3c3c3] p-2">Plan</th>
							<th class="border border-solid border-[#c3c3c3] p-2">RAM</th>
							<th class="border border-solid border-[#c3c3c3] p-2">Storage</th>
							<th class="border border-solid border-[#c3c3c3] p-2">CPU</th>
							<th class="border border-solid border-[#c3c3c3] p-2">Price</th>
							<th class="border border-solid border-[#c3c3c3] p-2">Order</th>
						</tr>
					</thead>
					<tbody>
						<tr>
							<td class="border border-solid border-[#c3c3c3] p-2 font-medium">Small</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.small.metadata.ram }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.small.metadata.storage }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.small.metadata.cpu }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ formatPlanIntervalPrice(plans.small) }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								<button
									v-if="capacityStatuses?.small?.available !== 0"
									class="m-0 bg-transparent p-0 text-link"
									type="button"
									@click="selectProduct('small')"
								>
									Order ►
								</button>
								<span v-else class="text-secondary">Sold out</span>
							</td>
						</tr>
						<tr>
							<td class="border border-solid border-[#c3c3c3] p-2 font-medium">Medium</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.medium.metadata.ram }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.medium.metadata.storage }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.medium.metadata.cpu }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ formatPlanIntervalPrice(plans.medium) }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								<button
									v-if="capacityStatuses?.medium?.available !== 0"
									class="m-0 bg-transparent p-0 text-link"
									type="button"
									@click="selectProduct('medium')"
								>
									Order ►
								</button>
								<span v-else class="text-secondary">Sold out</span>
							</td>
						</tr>
						<tr>
							<td class="border border-solid border-[#c3c3c3] p-2 font-medium">Large</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.large.metadata.ram }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.large.metadata.storage }} MB
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ plans.large.metadata.cpu }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								{{ formatPlanIntervalPrice(plans.large) }}
							</td>
							<td class="border border-solid border-[#c3c3c3] p-2">
								<button
									v-if="capacityStatuses?.large?.available !== 0"
									class="m-0 bg-transparent p-0 text-link"
									type="button"
									@click="selectProduct('large')"
								>
									Order ►
								</button>
								<span v-else class="text-secondary">Sold out</span>
							</td>
						</tr>
					</tbody>
				</table>

				<div
					class="mt-6 flex flex-col gap-2 border border-dashed border-[#c3c3c3] bg-white p-4 sm:flex-row sm:items-center sm:justify-between"
				>
					<div>
						<p class="m-0 font-bold">Know exactly what you need?</p>
						<p class="m-0 mt-1 text-secondary">
							Pick a customized plan with just the specs you need.
						</p>
						<p v-if="lowestPrice" class="m-0 mt-1">
							Starting at {{ formatPrice(lowestPrice, selectedCurrency, true) }} / month
						</p>
					</div>
					<ButtonStyled color="standard" size="large">
						<button class="flex items-center gap-2" type="button" @click="selectProduct('custom')">
							Get started
							<RightArrowIcon class="shrink-0" />
						</button>
					</ButtonStyled>
				</div>

				<p class="m-0 mt-6 text-center text-xs text-secondary">
					Prices shown for the selected billing period. Taxes may apply.
				</p>
			</div>
		</div>
	</div>
</template>

<script setup>
import { RightArrowIcon, SettingsIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	injectNotificationManager,
	ModrinthServersPurchaseModal,
	useFormatPrice,
} from '@modrinth/ui'
import { monthsInInterval } from '@modrinth/ui/src/utils/billing.ts'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import OptionGroup from '~/components/ui/OptionGroup.vue'
import { products } from '~/generated/state.json'

const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()

const { setAffiliateCode, getAffiliateCode } = useAffiliates()

const affiliateCode = ref(route.query.afl ?? null)

if (affiliateCode.value) {
	router.replace({
		query: {
			...route.query,
			afl: undefined,
		},
	})
	setAffiliateCode(affiliateCode.value)
} else {
	affiliateCode.value = getAffiliateCode()
}

const { addNotification } = injectNotificationManager()
const formatPrice = useFormatPrice()

const billingPeriods = ref(['monthly', 'quarterly'])
const billingPeriod = ref(billingPeriods.value.includes('quarterly') ? 'quarterly' : 'monthly')

const pyroProducts = products
	.filter((p) => p.metadata.type === 'pyro')
	.sort((a, b) => a.metadata.ram - b.metadata.ram)
const pyroPlanProducts = pyroProducts.filter(
	(p) => p.metadata.ram === 4096 || p.metadata.ram === 6144 || p.metadata.ram === 8192,
)

const selectedCurrency = ref('USD')

const lowestPrice = computed(() => {
	const amount = pyroProducts[0]?.prices?.find(
		(price) => price.currency_code === selectedCurrency.value,
	)?.prices?.intervals?.[billingPeriod.value]
	return amount ? amount / monthsInInterval[billingPeriod.value] : undefined
})

const title = 'Modrinth Hosting'
const description =
	'Start your own Minecraft server directly on Modrinth. Play your favorite mods, plugins, and datapacks — without the hassle of setup.'

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

const auth = await useAuth()
const config = useRuntimeConfig()
const purchaseModal = ref(null)
const customer = ref(null)
const paymentMethods = ref([])
const selectedProduct = ref(null)
const selectedProjectId = ref()

const loggedOut = computed(() => !auth.value.user)
const outOfStockUrl = 'https://discord.modrinth.com'

const { data: hasServers } = useQuery({
	queryKey: computed(() => ['servers', 'list-count', auth.value?.user?.id]),
	queryFn: async () => {
		try {
			if (!auth.value.user) return false
			const response = await client.archon.servers_v0.list()
			return response.servers && response.servers.length > 0
		} catch {
			return false
		}
	},
	enabled: computed(() => !!auth.value?.user),
})

function fetchStock(region, request) {
	return client.archon.servers_v0.checkStock(region.shortcode, request).then((res) => res.available)
}

async function fetchCapacityStatuses(customProduct = null) {
	try {
		const productsToCheck = customProduct?.metadata
			? [customProduct]
			: [
					...pyroPlanProducts,
					pyroProducts.reduce((min, product) =>
						product.metadata.ram < min.metadata.ram ? product : min,
					),
				]
		const capacityChecks = []
		for (const product of productsToCheck) {
			capacityChecks.push(
				client.archon.servers_v0.checkStockGlobal({
					cpu: product.metadata.cpu,
					memory_mb: product.metadata.ram,
					swap_mb: product.metadata.swap,
					storage_mb: product.metadata.storage,
				}),
			)
		}

		if (customProduct?.metadata) {
			return {
				custom: await capacityChecks[0],
			}
		}
		return {
			small: await capacityChecks[0],
			medium: await capacityChecks[1],
			large: await capacityChecks[2],
			custom: await capacityChecks[3],
		}
	} catch (error) {
		console.error('Error checking server capacities:', error)
		return {
			custom: { available: 0 },
			small: { available: 0 },
			medium: { available: 0 },
			large: { available: 0 },
		}
	}
}

const { data: capacityStatuses, refetch: refreshCapacity } = useQuery({
	queryKey: ['server', 'capacity', 'all'],
	queryFn: fetchCapacityStatuses,
	staleTime: 0,
	gcTime: 0,
})

const isSmallAtCapacity = computed(() => capacityStatuses.value?.small?.available === 0)
const isMediumAtCapacity = computed(() => capacityStatuses.value?.medium?.available === 0)
const isLargeAtCapacity = computed(() => capacityStatuses.value?.large?.available === 0)
const isCustomAtCapacity = computed(() => capacityStatuses.value?.custom?.available === 0)

const handleError = (err) => {
	addNotification({
		title: 'An error occurred',
		type: 'error',
		text: err.message ?? (err.data ? err.data.description : err),
	})
}

async function fetchPaymentData() {
	if (!auth.value.user) return
	try {
		const [customerData, paymentMethodsData] = await Promise.all([
			client.labrinth.billing_internal.getCustomer(),
			client.labrinth.billing_internal.getPaymentMethods(),
		])
		customer.value = customerData
		paymentMethods.value = paymentMethodsData
	} catch (error) {
		console.error('Error fetching payment data:', error)
		addNotification({
			title: 'Error fetching payment data',
			type: 'error',
			text: error.message || 'An unexpected error occurred',
		})
	}
}

const isAtCapacity = computed(
	() => isSmallAtCapacity.value && isMediumAtCapacity.value && isLargeAtCapacity.value,
)

const plans = {
	small: pyroPlanProducts?.[0],
	medium: pyroPlanProducts?.[1],
	large: pyroPlanProducts?.[2],
	custom: pyroProducts || [],
}

function formatBillingPeriodSuffix(interval) {
	const months = monthsInInterval[interval]
	if (months === 1) return ' / month'
	return ` / ${months} months`
}

function formatPlanIntervalPrice(plan) {
	const raw = plan?.prices?.find((x) => x.currency_code === selectedCurrency.value)?.prices
		?.intervals?.[billingPeriod.value]
	if (raw == null) return '—'
	return `${formatPrice(raw, selectedCurrency.value, true)}${formatBillingPeriodSuffix(billingPeriod.value)}`
}

const selectProduct = async (product) => {
	const data = useNuxtApp()
	if (loggedOut.value) {
		data.$router.push(`/auth/sign-in?redirect=${encodeURIComponent('/servers?plan=' + product)}`)
		return
	}

	await refreshCapacity()

	if ((product === 'custom' && isCustomAtCapacity.value) || isAtCapacity.value) {
		addNotification({
			title: 'Server Capacity Full',
			type: 'error',
			text: 'We are currently at capacity. Please try again later.',
		})
		return
	}

	const selectedPlan = plans[product]
	if (!selectedPlan) return

	if (
		(product === 'custom' && !selectedPlan.length) ||
		(product !== 'custom' && !selectedPlan.metadata)
	) {
		addNotification({
			title: 'Invalid product',
			type: 'error',
			text: 'The selected product was found but lacks necessary data. Please contact support.',
		})
		return
	}

	if (!pyroProducts.metadata) {
		pyroProducts.metadata = {}
	}
	pyroProducts.metadata.type = 'pyro'

	selectedProduct.value = selectedPlan
	await nextTick()

	if (product === 'custom') {
		purchaseModal.value?.show(billingPeriod.value, null, selectedProjectId.value)
	} else {
		purchaseModal.value?.show(billingPeriod.value, selectedProduct.value, selectedProjectId.value)
	}
}

const planQuery = async () => {
	if ('plan' in route.query) {
		await nextTick()
		const planElement = document.getElementById('plans')
		if (planElement) {
			planElement.scrollIntoView({ behavior: 'smooth' })
			if (route.query.plan !== null) {
				await selectProduct(route.query.plan)
			}
		}
	}
}

const regions = ref([])
const regionPings = ref([])

function pingRegions() {
	client.archon.servers_v1.getRegions().then((res) => {
		regions.value = res
		regions.value.forEach((region) => {
			runPingTest(region)
		})
	})
}

const PING_COUNT = 20
const PING_INTERVAL = 200
const MAX_PING_TIME = 1000

function runPingTest(region, index = 1) {
	if (index > 10) {
		regionPings.value.push({
			region: region.shortcode,
			ping: -1,
		})
		return
	}

	const wsUrl = `wss://${region.shortcode}${index}.${region.zone}/pingtest`
	try {
		const socket = new WebSocket(wsUrl)
		const pings = []

		socket.onopen = () => {
			for (let i = 0; i < PING_COUNT; i++) {
				setTimeout(() => {
					socket.send(performance.now())
				}, i * PING_INTERVAL)
			}
			setTimeout(
				() => {
					socket.close()

					const median = Math.round([...pings].sort((a, b) => a - b)[Math.floor(pings.length / 2)])
					if (median) {
						regionPings.value.push({
							region: region.shortcode,
							ping: median,
						})
					}
				},
				PING_COUNT * PING_INTERVAL + MAX_PING_TIME,
			)
		}

		socket.onmessage = (event) => {
			pings.push(performance.now() - event.data)
		}

		socket.onerror = (event) => {
			console.error(
				`Failed to connect pingtest WebSocket with ${wsUrl}, trying index ${index + 1}:`,
				event,
			)
			runPingTest(region, index + 1)
		}
	} catch (error) {
		console.error(`Failed to connect pingtest WebSocket with ${wsUrl}:`, error)
	}
}

onMounted(() => {
	if (route.query?.project) {
		selectedProjectId.value = route.query?.project
	}
	planQuery()
	pingRegions()
	fetchPaymentData()
})

watch(customer, (newCustomer) => {
	if (newCustomer) planQuery()
})

onUnmounted(() => {
	if (window.Stripe) {
		window.Stripe = null
	}
})
</script>
