<template>
	<div
		data-pyro-server-list-root
		class="experimental-styles-within relative mx-auto mb-6 flex w-full max-w-[1280px] flex-col px-6"
		:class="serverList.length ? 'min-h-screen' : 'min-h-[calc(100vh-4.5rem)]'"
	>
		<ServersGuestPlanModal
			ref="guestPlanModal"
			:available-products="pyroProducts"
			:currency="selectedCurrency"
			@continue="handleGuestPlanContinue"
		/>
		<ModrinthServersPurchaseModal
			v-if="customer && paymentMethods && regions"
			ref="purchaseModal"
			:publishable-key="props.stripePublishableKey"
			:initiate-payment="
				async (body) => await client.labrinth.billing_internal.initiatePayment(body)
			"
			:available-products="pyroProducts"
			:on-error="handleError"
			:customer="customer"
			:payment-methods="paymentMethods"
			:currency="selectedCurrency"
			:return-url="`${props.siteUrl}/hosting/manage`"
			:pings="regionPings"
			:regions="regions"
			:refresh-payment-methods="fetchPaymentData"
			:fetch-stock="fetchStock"
			:affiliate-code="affiliateCode"
			plan-stage
		/>
		<ResubscribeModal ref="resubscribeModal" @resubscribe="handleResubscribeConfirm" />

		<div
			v-if="hasError"
			class="mx-auto flex h-full min-h-[calc(100vh-4rem)] flex-col items-center justify-center gap-4 text-left"
		>
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-blue p-4">
							<HammerIcon class="size-12 text-blue" />
						</div>
						<h1 class="m-0 w-fit text-3xl font-bold">Servers could not be loaded</h1>
					</div>
					<p class="text-lg text-secondary">We may have temporary issues with our servers.</p>
					<ul class="m-0 list-disc space-y-4 p-0 pl-4 text-left text-sm leading-[170%]">
						<li>
							Our systems automatically alert our team when there's an issue. We are already working
							on getting them back online.
						</li>
						<li>
							If you recently purchased your Modrinth Hosting server, it is currently in a queue and
							will appear here as soon as it's ready. <br />
							<span class="font-medium text-contrast"
								>Do not attempt to purchase a new server.</span
							>
						</li>
						<li>
							If you require personalized support regarding the status of your server, please
							contact Modrinth Support.
						</li>

						<li v-if="fetchError" class="text-red">
							<p>Error details:</p>
							<CopyCode
								:text="(fetchError as ModrinthServersFetchError).message || 'Unknown error'"
								:copyable="false"
								:selectable="false"
								:language="'json'"
							/>
						</li>
					</ul>
				</div>
				<ButtonStyled size="large" type="standard" color="brand">
					<AutoLink class="mt-6 !w-full" to="https://support.modrinth.com"
						>Contact Modrinth Support</AutoLink
					>
				</ButtonStyled>
				<ButtonStyled size="large" @click="() => router.go(0)">
					<button class="mt-3 !w-full">Reload</button>
				</ButtonStyled>
			</div>
		</div>

		<Transition v-else name="fade" mode="out-in">
			<div v-if="isLoading && !serverResponse" key="loading" class="flex flex-col gap-4 py-8">
				<div class="mb-4 text-center">
					<LoaderCircleIcon class="mx-auto size-8 animate-spin text-contrast" />
					<p class="m-0 mt-2 text-secondary">Loading your servers...</p>
				</div>
				<div
					v-for="i in 3"
					:key="i"
					class="flex animate-pulse flex-row items-center gap-4 overflow-x-hidden rounded-2xl border-[1px] border-solid border-button-bg bg-bg-raised p-4"
				>
					<div class="size-16 rounded-xl bg-button-bg"></div>
					<div class="flex flex-1 flex-col gap-2">
						<div class="h-6 w-48 rounded bg-button-bg"></div>
						<div class="h-4 w-64 rounded bg-button-bg opacity-75"></div>
					</div>
				</div>
			</div>

			<div
				v-else-if="serverList.length === 0 && !isPollingForNewServers"
				key="empty"
				class="flex h-full flex-col items-center justify-center gap-8 grow max-h-[1100px]"
			>
				<ServerListEmpty
					:logged-in="loggedIn"
					@click-new-server="openPurchaseModal"
					@click-sign-in="handleSignIn"
				/>
			</div>

			<div v-else key="list">
				<div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
					<h1 class="w-full text-4xl font-bold text-contrast">Servers</h1>
					<div class="mb-4 flex w-full flex-row items-center justify-end gap-2 md:mb-0 md:gap-4">
						<StyledInput
							id="search"
							v-model="searchInput"
							:icon="SearchIcon"
							type="search"
							name="search"
							autocomplete="off"
							placeholder="Search servers..."
							wrapper-class="w-full md:w-72"
						/>
						<ButtonStyled type="standard">
							<AutoLink v-if="isNuxt" :to="{ path: '/servers', hash: '#plan' }">
								<PlusIcon />
								New server
							</AutoLink>
							<button v-else @click="openPurchaseModal">
								<PlusIcon />
								New server
							</button>
						</ButtonStyled>
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-300 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-20"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-20"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="isPollingForNewServers"
						class="bg-brand/10 my-4 flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm text-brand"
					>
						<LoaderCircleIcon class="size-4 animate-spin" />
						<span>Checking for new servers...</span>
					</div>
				</Transition>

				<TransitionGroup
					v-if="filteredData.length > 0 || isPollingForNewServers"
					name="list"
					tag="ul"
					class="m-0 flex flex-col gap-4 p-0"
				>
					<MedalServerListing
						v-for="server in filteredData.filter((s) => s.is_medal)"
						:key="server.server_id"
						v-bind="server"
						@upgrade="openPurchaseModal"
					/>
					<ServerListing
						v-for="server in filteredData.filter((s) => !s.is_medal)"
						:key="server.server_id"
						v-bind="server"
						:cancellation-date="serverBillingMap.get(server.server_id)?.cancellationDate"
						:is-provisioning="serverBillingMap.get(server.server_id)?.isProvisioning"
						:on-resubscribe="serverBillingMap.get(server.server_id)?.onResubscribe"
						:on-download-backup="serverBillingMap.get(server.server_id)?.onDownloadBackup"
					/>
				</TransitionGroup>
				<div v-else-if="isLoading" class="flex h-full items-center justify-center">
					<p class="text-contrast"><LoaderCircleIcon class="size-5 animate-spin" /></p>
				</div>
				<div v-else>No servers found.</div>
			</div>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { type Archon, type Labrinth, NuxtModrinthClient } from '@modrinth/api-client'
import { HammerIcon, LoaderCircleIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import {
	AutoLink,
	ButtonStyled,
	CopyCode,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	ModrinthServersPurchaseModal,
	ResubscribeModal,
	ServerListEmpty,
	ServersGuestPlanModal,
	StyledInput,
} from '@modrinth/ui'
import type { ModrinthServersFetchError } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useIntervalFn } from '@vueuse/core'
import dayjs from 'dayjs'
import Fuse from 'fuse.js'
import type Stripe from 'stripe'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import MedalServerListing from '#ui/components/servers/marketing/MedalServerListing.vue'
import ServerListing from '#ui/components/servers/ServerListing.vue'

const props = defineProps<{
	stripePublishableKey: string
	siteUrl: string
	products: Labrinth.Billing.Internal.Product[]
}>()

const router = useRouter()
const route = useRoute()
const auth = injectAuth()
const client = injectModrinthClient()
const loggedIn = computed(() => !!auth.user.value)

const isNuxt = computed(() => client instanceof NuxtModrinthClient)
const PURCHASE_INTENT_STORAGE_KEY = 'modrinth:servers-purchase-intent'
const PURCHASE_INTENT_SOURCE = 'hosting-manage'

type ServerBillingInterval = 'monthly' | 'quarterly' | 'yearly'

type PendingPurchaseIntent = {
	interval: ServerBillingInterval
	planId: string | null
	source: string
	ts: number
}

const isPollingForNewServers = ref(false)
const pollingState = ref({
	enabled: false,
	count: 0,
	initialServers: [] as Archon.Servers.v0.Server[],
})

const guestPlanModal = ref<InstanceType<typeof ServersGuestPlanModal> | null>(null)
const purchaseModal = ref<InstanceType<typeof ModrinthServersPurchaseModal> | null>(null)
const resubscribeModal = ref<InstanceType<typeof ResubscribeModal> | null>(null)
const affiliateCode = ref<string | null>(null)
const selectedCurrency = ref<string>('USD')
const lastSelectedInterval = ref<ServerBillingInterval>('quarterly')
const lastSelectedPlanId = ref<string | null | undefined>(undefined)
const pendingResumeIntent = ref<PendingPurchaseIntent | null>(null)
const regionPings = ref<
	{
		region: string
		ping: number
	}[]
>([])

const pyroProducts = computed(() => {
	return [...props.products]
		.filter((p) => p?.metadata?.type === 'pyro' || p?.metadata?.type === 'medal')
		.sort((a, b) => {
			const aRam =
				a?.metadata?.type === 'pyro' || a?.metadata?.type === 'medal' ? a.metadata.ram : 0
			const bRam =
				b?.metadata?.type === 'pyro' || b?.metadata?.type === 'medal' ? b.metadata.ram : 0
			return aRam - bRam
		})
})

function readPendingPurchaseIntent(): PendingPurchaseIntent | null {
	if (typeof window === 'undefined') return null
	const rawIntent = window.sessionStorage.getItem(PURCHASE_INTENT_STORAGE_KEY)
	if (!rawIntent) return null

	try {
		const parsedIntent = JSON.parse(rawIntent) as Partial<PendingPurchaseIntent>
		if (
			(parsedIntent.interval === 'monthly' ||
				parsedIntent.interval === 'quarterly' ||
				parsedIntent.interval === 'yearly') &&
			(parsedIntent.planId === null || typeof parsedIntent.planId === 'string') &&
			typeof parsedIntent.source === 'string' &&
			typeof parsedIntent.ts === 'number'
		) {
			return parsedIntent as PendingPurchaseIntent
		}
	} catch {
		return null
	}

	return null
}

function writePendingPurchaseIntent(intent: PendingPurchaseIntent) {
	if (typeof window === 'undefined') return
	window.sessionStorage.setItem(PURCHASE_INTENT_STORAGE_KEY, JSON.stringify(intent))
}

function clearPendingPurchaseIntent() {
	if (typeof window === 'undefined') return
	window.sessionStorage.removeItem(PURCHASE_INTENT_STORAGE_KEY)
}

const {
	data: customer,
	refetch: refetchCustomer,
	isLoading: customerLoading,
} = useQuery({
	queryKey: ['billing', 'customer'],
	queryFn: () => client.labrinth.billing_internal.getCustomer() as Promise<Stripe.Customer>,
	enabled: loggedIn,
})

const {
	data: paymentMethods,
	refetch: refetchPaymentMethods,
	isLoading: paymentMethodsLoading,
} = useQuery({
	queryKey: ['billing', 'payment-methods'],
	queryFn: () =>
		client.labrinth.billing_internal.getPaymentMethods() as Promise<Stripe.PaymentMethod[]>,
	enabled: loggedIn,
})

const { data: regions, isLoading: regionsLoading } = useQuery({
	queryKey: ['servers', 'regions'],
	queryFn: () => client.archon.servers_v1.getRegions(),
	enabled: loggedIn,
})

watch(
	regions,
	(newRegions) => {
		regionPings.value = []
		if (newRegions) {
			newRegions.forEach((region) => {
				runPingTest(region)
			})
		}
	},
	{ immediate: true },
)

async function fetchPaymentData() {
	await Promise.all([refetchCustomer(), refetchPaymentMethods()])
}

async function fetchStock(
	region: Archon.Servers.v1.Region,
	request: Archon.Servers.v0.StockRequest,
): Promise<number> {
	const result = await client.archon.servers_v0.checkStock(region.shortcode, request)
	return result.available
}

const PING_COUNT = 20
const PING_INTERVAL = 200
const MAX_PING_TIME = 1000

function runPingTest(region: Archon.Servers.v1.Region, index = 1) {
	if (index > 10) {
		regionPings.value = regionPings.value.filter((entry) => entry.region !== region.shortcode)
		regionPings.value.push({
			region: region.shortcode,
			ping: -1,
		})
		return
	}

	const wsUrl = `wss://${region.shortcode}${index}.${region.zone}/pingtest`
	try {
		const socket = new WebSocket(wsUrl)
		const pings: number[] = []
		let finalized = false

		const finalize = (ping: number) => {
			if (finalized) return
			finalized = true
			clearTimeout(connectTimeout)
			regionPings.value = regionPings.value.filter((entry) => entry.region !== region.shortcode)
			regionPings.value.push({
				region: region.shortcode,
				ping,
			})
			socket.close()
		}

		const retryNext = () => {
			if (finalized) return
			finalized = true
			clearTimeout(connectTimeout)
			socket.close()
			runPingTest(region, index + 1)
		}

		// Prevent hangs where the socket never opens or errors.
		const connectTimeout = setTimeout(() => {
			retryNext()
		}, 3000)

		socket.onopen = () => {
			clearTimeout(connectTimeout)

			for (let i = 0; i < PING_COUNT; i++) {
				setTimeout(() => {
					socket.send(String(performance.now()))
				}, i * PING_INTERVAL)
			}
			setTimeout(
				() => {
					const median =
						pings.length > 0
							? Math.round([...pings].sort((a, b) => a - b)[Math.floor(pings.length / 2)])
							: -1
					finalize(median)
				},
				PING_COUNT * PING_INTERVAL + MAX_PING_TIME,
			)
		}

		socket.onmessage = (event) => {
			const start = Number(event.data)
			pings.push(performance.now() - start)
		}

		socket.onerror = () => {
			retryNext()
		}
	} catch {
		runPingTest(region, index + 1)
	}
}

const {
	data: serverResponse,
	error: fetchError,
	isLoading,
} = useQuery({
	queryKey: ['servers'],
	queryFn: async () => {
		const response = await client.archon.servers_v0.list()

		// Fetch subscriptions for medal servers
		const hasMedalServers = response.servers.some((s) => s.is_medal)
		if (hasMedalServers) {
			const subscriptions = await client.labrinth.billing_internal.getSubscriptions()

			// Inject medal_expires into servers
			for (const server of response.servers) {
				if (server.is_medal) {
					const sub = subscriptions.find((s) => s.metadata?.id === server.server_id)
					if (sub) {
						server.medal_expires = dayjs(sub.created).add(5, 'days').toISOString()
					}
				}
			}
		}

		// Check if new servers appeared (stop polling)
		if (pollingState.value.enabled) {
			pollingState.value.count++
			if (response.servers.length !== pollingState.value.initialServers.length) {
				pollingState.value.enabled = false
				isPollingForNewServers.value = false
				router.replace({ query: {} })
			} else if (pollingState.value.count >= 5) {
				pollingState.value.enabled = false
				isPollingForNewServers.value = false
			}
		}

		return response
	},
	refetchInterval: computed(() => (pollingState.value.enabled ? 5000 : false)),
	enabled: loggedIn,
})

const hasError = computed(() => loggedIn.value && !!fetchError.value)

const serverList = computed<Archon.Servers.v0.Server[]>(() => {
	if (!serverResponse.value) return []
	return serverResponse.value.servers
})

const searchInput = ref('')

const fuse = computed(() => {
	if (serverList.value.length === 0) return null
	return new Fuse(serverList.value, {
		keys: ['name', 'loader', 'mc_version', 'game', 'state'],
		includeScore: true,
		threshold: 0.4,
	})
})

function introToTop(array: Archon.Servers.v0.Server[]): Archon.Servers.v0.Server[] {
	return array.slice().sort((a, b) => {
		return Number(b.flows?.intro) - Number(a.flows?.intro)
	})
}

// files expire 30 days after cancellation
function filesExpired(server: Archon.Servers.v0.Server): boolean {
	if (server.status !== 'suspended' || server.suspension_reason !== 'cancelled') return false
	const cancellationDate = serverBillingMap.value.get(server.server_id)?.cancellationDate
	if (!cancellationDate) return false
	const cancellation = new Date(cancellationDate)
	const thirtyDaysLater = new Date(cancellation.getTime() + 30 * 24 * 60 * 60 * 1000)
	return new Date() > thirtyDaysLater
}

const filteredData = computed<Archon.Servers.v0.Server[]>(() => {
	const base = !searchInput.value.trim()
		? introToTop(serverList.value)
		: fuse.value
			? introToTop(fuse.value.search(searchInput.value).map((result) => result.item))
			: []
	return base.filter((server) => !filesExpired(server))
})

// Start polling only after initial data is available so the baseline is correct
watch(serverResponse, (response) => {
	if (
		route.query.redirect_status === 'succeeded' &&
		response &&
		!pollingState.value.enabled &&
		pollingState.value.count === 0
	) {
		isPollingForNewServers.value = true
		pollingState.value = {
			enabled: true,
			count: 0,
			initialServers: [...response.servers],
		}
	}
})

const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

const canOpenPurchaseModal = computed(() => {
	return (
		Boolean(props.stripePublishableKey) &&
		Boolean(customer.value) &&
		paymentMethods.value !== undefined &&
		Boolean(regions.value) &&
		!customerLoading.value &&
		!paymentMethodsLoading.value &&
		!regionsLoading.value
	)
})

function openCheckoutFromIntent(intent: PendingPurchaseIntent): boolean {
	console.log('Attempting to open checkout from intent', intent)
	if (!purchaseModal.value || !canOpenPurchaseModal.value) return false

	lastSelectedInterval.value = intent.interval
	lastSelectedPlanId.value = intent.planId
	const defaultPlan =
		pyroProducts.value.find((p) => p?.metadata?.type === 'pyro' && p.metadata.ram === 6144) ??
		pyroProducts.value.find((p) => p?.metadata?.type === 'pyro') ??
		pyroProducts.value[0]
	const selectedPlan =
		intent.planId === null
			? null
			: (pyroProducts.value.find((product) => product.id === intent.planId) ?? defaultPlan)

	purchaseModal.value.show(intent.interval, selectedPlan)
	return true
}

function handleError(err: unknown) {
	const error = err as Error & { data?: { description?: string } }
	addNotification({
		title: 'An error occurred',
		type: 'error',
		text: error?.message ?? error?.data?.description ?? String(err),
	})
}

function openPurchaseModal() {
	guestPlanModal.value?.show(lastSelectedInterval.value, lastSelectedPlanId.value)
}

function handleGuestPlanContinue(payload: {
	interval: ServerBillingInterval
	planId: string | null
}) {
	const intent: PendingPurchaseIntent = {
		interval: payload.interval,
		planId: payload.planId,
		source: PURCHASE_INTENT_SOURCE,
		ts: Date.now(),
	}

	lastSelectedInterval.value = payload.interval
	lastSelectedPlanId.value = payload.planId

	if (!loggedIn.value) {
		writePendingPurchaseIntent(intent)
		void auth.requestSignIn('/hosting/manage')
		return
	}

	pendingResumeIntent.value = intent
	if (openCheckoutFromIntent(intent)) {
		pendingResumeIntent.value = null
	} else {
		addNotification({
			title: 'Purchase unavailable',
			text: 'Payment information is still loading. Opening checkout as soon as it is ready.',
			type: 'info',
		})
	}
}

function handleSignIn() {
	void auth.requestSignIn('/hosting/manage')
}

watch(
	loggedIn,
	(isLoggedIn) => {
		if (!isLoggedIn) return

		const pendingIntent = readPendingPurchaseIntent()
		if (!pendingIntent || pendingIntent.source !== PURCHASE_INTENT_SOURCE) return

		clearPendingPurchaseIntent()
		pendingResumeIntent.value = pendingIntent
		lastSelectedInterval.value = pendingIntent.interval
		lastSelectedPlanId.value = pendingIntent.planId
	},
	{ immediate: true },
)

watch(
	() =>
		[
			loggedIn.value,
			canOpenPurchaseModal.value,
			pendingResumeIntent.value,
			purchaseModal.value,
		] as const,
	([isLoggedIn, canOpen, pendingIntent]) => {
		if (!isLoggedIn || !canOpen || !pendingIntent) return
		if (openCheckoutFromIntent(pendingIntent)) {
			pendingResumeIntent.value = null
		}
	},
	{ immediate: true },
)

const { data: subscriptions } = useQuery({
	queryKey: ['billing', 'subscriptions'],
	queryFn: () => client.labrinth.billing_internal.getSubscriptions(),
	enabled: loggedIn,
})

const { data: charges } = useQuery({
	queryKey: ['billing', 'payments'],
	queryFn: () => client.labrinth.billing_internal.getPayments(),
	enabled: loggedIn,
})

const CHARGE_POLL_INTERVAL_MS = 20_000

const hasProvisioningSubscription = computed(() => {
	if (!subscriptions.value || !charges.value) return false
	return subscriptions.value
		.filter((s) => s?.metadata?.type === 'pyro')
		.some((sub) => {
			if (sub.status !== 'unprovisioned') return false
			const charge = charges.value?.find((c) => c.subscription_id === sub.id)
			return charge?.status === 'processing' || charge?.status === 'open'
		})
})

const { pause: pauseChargePoll, resume: resumeChargePoll } = useIntervalFn(
	() => {
		queryClient.invalidateQueries({ queryKey: ['billing', 'payments'] })
		queryClient.invalidateQueries({ queryKey: ['billing', 'subscriptions'] })
		queryClient.invalidateQueries({ queryKey: ['servers'] })
	},
	CHARGE_POLL_INTERVAL_MS,
	{ immediate: false },
)

watch(
	hasProvisioningSubscription,
	(isProvisioning) => {
		if (isProvisioning) {
			resumeChargePoll()
		} else {
			pauseChargePoll()
		}
	},
	{ immediate: true },
)

const { data: serverFullList } = useQuery({
	queryKey: ['servers', 'v1'],
	queryFn: () => client.archon.servers_v1.list(),
	enabled: loggedIn,
})

type ServerBillingInfo = {
	cancellationDate?: string | null
	isProvisioning?: boolean
	onResubscribe?: () => void
	onDownloadBackup?: (() => void) | null
}

type ResubscribeRequest = {
	subscriptionId: string
	wasSuspended: boolean
}

function getLatestBackupDownload(serverId: string): (() => void) | null {
	const serverFull = serverFullList.value?.find((s) => s.id === serverId)
	if (!serverFull) return null

	const activeWorld = serverFull.worlds.find((w) => w.is_active) ?? serverFull.worlds[0]
	if (!activeWorld?.backups?.length) return null

	const latestBackup = activeWorld.backups
		.filter((b) => b.status === 'done')
		.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())[0]
	if (!latestBackup) return null

	return async () => {
		try {
			const server = await client.archon.servers_v0.get(serverId)
			const kyrosUrl = server.node?.instance
			const jwt = server.node?.token
			if (!kyrosUrl || !jwt) {
				addNotification({
					title: 'Download unavailable',
					text: 'Server connection info is not available. Please contact support.',
					type: 'error',
				})
				return
			}

			window.open(
				`https://${kyrosUrl}/modrinth/v0/backups/${latestBackup.id}/download?auth=${jwt}`,
				'_blank',
			)
		} catch {
			addNotification({
				title: 'Download failed',
				text: 'An error occurred while trying to download the backup.',
				type: 'error',
			})
		}
	}
}

function getProductFromPriceId(priceId: string | null | undefined) {
	if (!priceId) return null

	return (
		pyroProducts.value.find((product) => product.prices.some((price) => price.id === priceId)) ??
		null
	)
}

function getPlanName(product: Labrinth.Billing.Internal.Product | null): string {
	if (!product) return 'Medium plan'
	if (product.metadata.type !== 'pyro' && product.metadata.type !== 'medal') return 'Medium plan'

	switch (product.metadata.ram) {
		case 4096:
			return 'Small plan'
		case 6144:
			return 'Medium plan'
		case 8192:
			return 'Large plan'
		default:
			return 'Custom plan'
	}
}

function getRamGb(product: Labrinth.Billing.Internal.Product | null): number | undefined {
	if (!product) return undefined
	if (product.metadata.type !== 'pyro' && product.metadata.type !== 'medal') return undefined

	return product.metadata.ram / 1024
}

function getStorageGb(product: Labrinth.Billing.Internal.Product | null): number | undefined {
	if (!product) return undefined
	if (product.metadata.type !== 'pyro' && product.metadata.type !== 'medal') return undefined

	return product.metadata.storage / 1024
}

function getSharedCpus(product: Labrinth.Billing.Internal.Product | null): number | undefined {
	if (!product) return undefined
	if (product.metadata.type !== 'pyro' && product.metadata.type !== 'medal') return undefined

	return product.metadata.cpu / 2
}

function getRecurringPrice(
	product: Labrinth.Billing.Internal.Product | null,
	interval: Labrinth.Billing.Internal.PriceDuration,
	preferredCurrency?: string,
): { amount: number; currencyCode: string } | null {
	if (!product) return null

	const recurringPrices = product.prices.filter((price) => price.prices.type === 'recurring')
	const preferredPrice = preferredCurrency
		? recurringPrices.find((price) => price.currency_code === preferredCurrency)
		: undefined
	const usdPrice = recurringPrices.find((price) => price.currency_code === 'USD')
	const selectedPrice = preferredPrice ?? usdPrice ?? recurringPrices[0]

	if (!selectedPrice || selectedPrice.prices.type !== 'recurring') return null

	return {
		amount: selectedPrice.prices.intervals[interval],
		currencyCode: selectedPrice.currency_code,
	}
}

function openResubscribeModal(
	serverId: string,
	subscription: Labrinth.Billing.Internal.UserSubscription,
	charge?: Labrinth.Billing.Internal.Charge | null,
) {
	const displayInterval = charge?.subscription_interval ?? subscription.interval
	const displayPriceId = charge?.price_id ?? subscription.price_id
	const product = getProductFromPriceId(displayPriceId)
	const fallbackPrice = getRecurringPrice(product, displayInterval, charge?.currency_code)

	resubscribeModal.value?.show({
		subscriptionId: subscription.id,
		wasSuspended: !!charge?.due && dayjs(charge.due).isBefore(dayjs()),
		serverName:
			serverList.value.find((server) => server.server_id === serverId)?.name ?? 'this server',
		planName: getPlanName(product),
		ramGb: getRamGb(product),
		storageGb: getStorageGb(product),
		sharedCpus: getSharedCpus(product),
		priceCents: charge?.amount ?? fallbackPrice?.amount,
		currencyCode: charge?.currency_code ?? fallbackPrice?.currencyCode,
		interval: displayInterval,
		nextChargeDate: charge?.due,
	})
}

async function handleResubscribeConfirm({ subscriptionId, wasSuspended }: ResubscribeRequest) {
	try {
		await client.labrinth.billing_internal.editSubscription(subscriptionId, {
			cancelled: false,
		})
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['billing'] }),
			queryClient.invalidateQueries({ queryKey: ['servers'] }),
		])
		if (wasSuspended) {
			addNotification({
				title: 'Resubscription request submitted',
				text: 'If the server is currently cancelled, it may take up to 10 minutes for another charge attempt to be made.',
				type: 'success',
			})
		} else {
			addNotification({
				title: 'Success',
				text: 'Server subscription resubscribed successfully',
				type: 'success',
			})
		}
	} catch {
		addNotification({
			title: 'Error resubscribing',
			text: 'An error occurred while resubscribing to your Modrinth server.',
			type: 'error',
		})
	}
}

const serverBillingMap = computed(() => {
	const map = new Map<string, ServerBillingInfo>()
	if (!subscriptions.value || !charges.value) return map

	const pyroSubs = subscriptions.value.filter((s) => s?.metadata?.type === 'pyro')
	for (const sub of pyroSubs) {
		const serverId = (sub.metadata as { id?: string })?.id
		if (!serverId) continue

		const charge = charges.value.find(
			(c) => c.subscription_id === sub.id && c.status !== 'succeeded',
		)

		const info: ServerBillingInfo = {
			isProvisioning:
				sub.status === 'unprovisioned' &&
				(charge?.status === 'processing' || charge?.status === 'open'),
		}

		info.onDownloadBackup = getLatestBackupDownload(serverId)

		if (charge?.status === 'cancelled') {
			info.cancellationDate = charge.due

			info.onResubscribe = () => openResubscribeModal(serverId, sub, charge)
		}

		map.set(serverId, info)
	}

	return map
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}

.list-enter-active,
.list-leave-active {
	transition: all 200ms ease-in-out;
}

.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}
</style>
