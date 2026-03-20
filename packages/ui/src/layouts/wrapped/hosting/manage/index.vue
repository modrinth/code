<template>
	<div
		data-pyro-server-list-root
		class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
	>
		<ModrinthServersPurchaseModal
			v-if="customer && regions"
			:key="`purchase-modal-${customer.id}`"
			ref="purchaseModal"
			:publishable-key="props.stripePublishableKey"
			:initiate-payment="
				async (body) => await client.labrinth.billing_internal.initiatePayment(body)
			"
			:available-products="pyroProducts"
			:on-error="handleError"
			:customer="customer"
			:payment-methods="paymentMethods ?? []"
			:currency="selectedCurrency"
			:return-url="`${props.siteUrl}/hosting/manage`"
			:pings="regionPings"
			:regions="regions"
			:refresh-payment-methods="fetchPaymentData"
			:fetch-stock="fetchStock"
			:affiliate-code="affiliateCode"
			plan-stage
		/>

		<div
			v-if="hasError || fetchError"
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
				class="flex h-full flex-col items-center justify-center gap-8"
			>
				<img
					src="https://cdn.modrinth.com/servers/excitement.webp"
					alt=""
					class="max-w-[360px]"
					style="
						mask-image: radial-gradient(97% 77% at 50% 25%, #d9d9d9 0, hsla(0, 0%, 45%, 0) 100%);
					"
				/>
				<h1 class="m-0 text-contrast">You don't have any servers yet!</h1>
				<p class="m-0">Modrinth Hosting is a new way to play modded Minecraft with your friends.</p>
				<ButtonStyled size="large" type="standard" color="brand">
					<AutoLink v-if="isNuxt" to="/servers#plan">Create a server</AutoLink>
					<button v-else :disabled="!canOpenPurchaseModal" @click="openPurchaseModal">
						Create a server
					</button>
				</ButtonStyled>
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
							<button v-else :disabled="!canOpenPurchaseModal" @click="openPurchaseModal">
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
						:on-resubscribe="serverBillingMap.get(server.server_id)?.onResubscribe"
						:on-download-backup="serverBillingMap.get(server.server_id)?.onDownloadBackup"
					/>
				</TransitionGroup>
				<div v-else class="flex h-full items-center justify-center">
					<p class="text-contrast"><LoaderCircleIcon class="size-5 animate-spin" /></p>
				</div>
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
	injectModrinthClient,
	injectNotificationManager,
	ModrinthServersPurchaseModal,
	StyledInput,
} from '@modrinth/ui'
import type { ModrinthServersFetchError } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
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
const client = injectModrinthClient()

const isNuxt = computed(() => client instanceof NuxtModrinthClient)

const hasError = ref(false)
const isPollingForNewServers = ref(false)
const pollingState = ref({
	enabled: false,
	count: 0,
	initialServers: [] as Archon.Servers.v0.Server[],
})

const purchaseModal = ref<InstanceType<typeof ModrinthServersPurchaseModal> | null>(null)
const affiliateCode = ref<string | null>(null)
const selectedCurrency = ref<string>('USD')
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

const {
	data: customer,
	refetch: refetchCustomer,
	isLoading: customerLoading,
} = useQuery({
	queryKey: ['billing', 'customer'],
	queryFn: () => client.labrinth.billing_internal.getCustomer() as Promise<Stripe.Customer>,
})

const {
	data: paymentMethods,
	refetch: refetchPaymentMethods,
	isLoading: paymentMethodsLoading,
} = useQuery({
	queryKey: ['billing', 'payment-methods'],
	queryFn: () =>
		client.labrinth.billing_internal.getPaymentMethods() as Promise<Stripe.PaymentMethod[]>,
})

const { data: regions, isLoading: regionsLoading } = useQuery({
	queryKey: ['servers', 'regions'],
	queryFn: () => client.archon.servers_v1.getRegions(),
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
})

watch([fetchError, serverResponse], ([error, response]) => {
	hasError.value = !!error || !response
})

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

const filteredData = computed<Archon.Servers.v0.Server[]>(() => {
	if (!searchInput.value.trim()) {
		return introToTop(serverList.value)
	}
	return fuse.value
		? introToTop(fuse.value.search(searchInput.value).map((result) => result.item))
		: []
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

function handleError(err: unknown) {
	const error = err as Error & { data?: { description?: string } }
	addNotification({
		title: 'An error occurred',
		type: 'error',
		text: error?.message ?? error?.data?.description ?? String(err),
	})
}

function openPurchaseModal() {
	if (!canOpenPurchaseModal.value || !purchaseModal.value) {
		addNotification({
			title: 'Purchase unavailable',
			text: 'Payment information is still loading. Please try again in a moment.',
			type: 'warning',
		})
		return
	}

	purchaseModal.value.show('quarterly')
}

const { data: subscriptions } = useQuery({
	queryKey: ['billing', 'subscriptions'],
	queryFn: () => client.labrinth.billing_internal.getSubscriptions(),
})

const { data: charges } = useQuery({
	queryKey: ['billing', 'payments'],
	queryFn: () => client.labrinth.billing_internal.getPayments(),
})

const { data: serverFullList } = useQuery({
	queryKey: ['servers', 'v1'],
	queryFn: () => client.archon.servers_v1.list(),
})

type ServerBillingInfo = {
	cancellationDate?: string | null
	onResubscribe?: () => void
	onDownloadBackup?: (() => void) | null
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

		const info: ServerBillingInfo = {}

		info.onDownloadBackup = getLatestBackupDownload(serverId)

		if (charge?.status === 'cancelled') {
			info.cancellationDate = charge.due

			const subId = sub.id
			const wasSuspended = dayjs(charge.due).isBefore(dayjs())
			info.onResubscribe = async () => {
				try {
					await client.labrinth.billing_internal.editSubscription(subId, {
						cancelled: false,
					})
					await Promise.all([
						queryClient.invalidateQueries({ queryKey: ['billing'] }),
						queryClient.invalidateQueries({ queryKey: ['servers'] }),
					])
					if (wasSuspended) {
						addNotification({
							title: 'Resubscription request submitted',
							text: 'If the server is currently suspended, it may take up to 10 minutes for another charge attempt to be made.',
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
