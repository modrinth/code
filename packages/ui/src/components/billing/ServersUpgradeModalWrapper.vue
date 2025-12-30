<template>
	<ModrinthServersPurchaseModal
		v-if="customer && regionsData"
		ref="purchaseModal"
		:publishable-key="props.stripePublishableKey"
		:initiate-payment="async (body) => await initiatePayment(body)"
		:available-products="pyroProducts"
		:on-error="handleError"
		:customer="customer"
		:payment-methods="paymentMethods"
		:currency="selectedCurrency"
		:return-url="`${props.siteUrl}/hosting/manage`"
		:pings="regionPings"
		:regions="regionsData"
		:refresh-payment-methods="fetchPaymentData"
		:fetch-stock="fetchStock"
		:plan-stage="true"
		:existing-plan="currentPlanFromSubscription"
		:existing-subscription="subscription || undefined"
		:on-finalize-no-payment-change="finalizeDowngrade"
		@hide="
			() => {
				debug('modal hidden, resetting subscription')
				subscription = null
			}
		"
	/>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	injectModrinthClient,
	injectNotificationManager,
	ModrinthServersPurchaseModal,
	useDebugLogger,
} from '@modrinth/ui'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

const props = defineProps<{
	stripePublishableKey: string
	siteUrl: string
	products: Labrinth.Billing.Internal.Product[]
}>()

const { addNotification } = injectNotificationManager()
const { labrinth, archon } = injectModrinthClient()
const debug = useDebugLogger('ServersUpgradeModalWrapper')
const purchaseModal = ref<InstanceType<typeof ModrinthServersPurchaseModal> | null>(null)

// stripe type
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const customer = ref<any>(null)

// stripe type
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const paymentMethods = ref<any[]>([])
const selectedCurrency = ref<string>('USD')

const regionPings = ref<
	{
		region: string
		ping: number
	}[]
>([])

const pyroProducts = (props.products as Labrinth.Billing.Internal.Product[])
	.filter((p) => p?.metadata?.type === 'pyro' || p?.metadata?.type === 'medal')
	.sort((a, b) => {
		const aRam = a?.metadata?.type === 'pyro' || a?.metadata?.type === 'medal' ? a.metadata.ram : 0
		const bRam = b?.metadata?.type === 'pyro' || b?.metadata?.type === 'medal' ? b.metadata.ram : 0
		return aRam - bRam
	})

function handleError(err: unknown) {
	debug('Purchase modal error:', err)
}

const { data: customerData } = useQuery({
	queryKey: ['billing', 'customer'],
	queryFn: () => labrinth.billing_internal.getCustomer(),
})

const { data: paymentMethodsData, refetch: refetchPaymentMethods } = useQuery({
	queryKey: ['billing', 'payment-methods'],
	queryFn: () => labrinth.billing_internal.getPaymentMethods(),
})

const { data: regionsData } = useQuery({
	queryKey: ['servers', 'regions'],
	queryFn: () => archon.servers_v1.getRegions(),
})

watch(customerData, (newCustomer) => {
	if (newCustomer) customer.value = newCustomer
})

watch(paymentMethodsData, (newMethods) => {
	if (newMethods) paymentMethods.value = newMethods
})

watch(regionsData, (newRegions) => {
	if (newRegions) {
		newRegions.forEach((region) => {
			runPingTest(region)
		})
	}
})

async function fetchPaymentData() {
	await refetchPaymentMethods()
}

async function fetchStock(
	region: Archon.Servers.v1.Region,
	request: Archon.Servers.v0.StockRequest,
): Promise<number> {
	const result = await archon.servers_v0.checkStock(region.shortcode, request)
	return result.available
}

const PING_COUNT = 20
const PING_INTERVAL = 200
const MAX_PING_TIME = 1000

function runPingTest(region: Archon.Servers.v1.Region, index = 1) {
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
		const pings: number[] = []

		socket.onopen = () => {
			for (let i = 0; i < PING_COUNT; i++) {
				setTimeout(() => {
					socket.send(String(performance.now()))
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
			const start = Number(event.data)
			pings.push(performance.now() - start)
		}

		socket.onerror = () => {
			runPingTest(region, index + 1)
		}
	} catch {
		// ignore
	}
}

const subscription = ref<Labrinth.Billing.Internal.UserSubscription | null>(null)
// Dry run state
const dryRunResponse = ref<{
	requires_payment: boolean
	required_payment_is_proration: boolean
} | null>(null)
const pendingDowngradeBody = ref<Labrinth.Billing.Internal.EditSubscriptionRequest | null>(null)
const currentPlanFromSubscription = computed<Labrinth.Billing.Internal.Product | undefined>(() => {
	return subscription.value
		? pyroProducts.find((p) =>
				p.prices.some((price) => price.id === subscription.value?.price_id),
			) ?? undefined
		: undefined
})

const currentInterval = computed<'monthly' | 'quarterly'>(() => {
	const interval = subscription.value?.interval

	if (interval === 'monthly' || interval === 'quarterly') {
		return interval
	}

	return 'monthly'
})

const editSubscriptionMutation = useMutation({
	mutationFn: async ({
		id,
		body,
		dry,
	}: {
		id: string
		body: Labrinth.Billing.Internal.EditSubscriptionRequest
		dry: boolean
	}) => {
		return await labrinth.billing_internal.editSubscription(id, body, dry)
	},
})

async function initiatePayment(
	body: Labrinth.Billing.Internal.InitiatePaymentRequest,
): Promise<Labrinth.Billing.Internal.EditSubscriptionResponse | null> {
	debug('initiatePayment called', {
		hasSubscription: !!subscription.value,
		subscriptionId: subscription.value?.id,
		body,
	})
	if (subscription.value) {
		const transformedBody: Labrinth.Billing.Internal.EditSubscriptionRequest = {
			interval: body.charge.type === 'new' ? body.charge.interval : undefined,
			payment_method: body.type === 'confirmation_token' ? body.token : body.id,
			product: body.charge.type === 'new' ? body.charge.product_id : undefined,
			region: body.metadata?.server_region,
		}

		try {
			const dry = await editSubscriptionMutation.mutateAsync({
				id: subscription.value.id,
				body: transformedBody,
				dry: true,
			})

			if (dry && typeof dry === 'object' && 'payment_intent_id' in dry) {
				dryRunResponse.value = {
					requires_payment: !!dry.payment_intent_id,
					required_payment_is_proration: true,
				}
				pendingDowngradeBody.value = transformedBody
				if (dry.payment_intent_id) {
					return await finalizeImmediate(transformedBody)
				} else {
					return null
				}
			} else {
				// Fallback if dry run not supported
				return await finalizeImmediate(transformedBody)
			}
		} catch (e) {
			debug('Dry run failed, attempting immediate patch', e)
			return await finalizeImmediate(transformedBody)
		}
	} else {
		debug('subscription.value is null/undefined', {
			subscriptionValue: subscription.value,
		})
		addNotification({
			title: 'Unable to determine subscription ID.',
			text: 'Please contact support.',
			type: 'error',
		})
		return Promise.reject(new Error('Unable to determine subscription ID.'))
	}
}

async function finalizeImmediate(body: Labrinth.Billing.Internal.EditSubscriptionRequest) {
	if (!subscription.value) return null

	const result = await editSubscriptionMutation.mutateAsync({
		id: subscription.value.id,
		body,
		dry: false,
	})

	return result ?? null
}

async function finalizeDowngrade() {
	if (!subscription.value || !pendingDowngradeBody.value) return
	try {
		await finalizeImmediate(pendingDowngradeBody.value)
		addNotification({
			title: 'Subscription updated',
			text: 'Your plan has been downgraded and will take effect next billing cycle.',
			type: 'success',
		})
	} catch (e) {
		addNotification({
			title: 'Failed to apply subscription changes',
			text: 'Please try again or contact support.',
			type: 'error',
		})
		throw e
	} finally {
		dryRunResponse.value = null
		pendingDowngradeBody.value = null
	}
}

async function open(id?: string) {
	debug('open called', { id })
	if (id) {
		const subscriptions = await labrinth.billing_internal.getSubscriptions()
		debug('fetched subscriptions', {
			count: subscriptions.length,
			subscriptions: subscriptions.map((s) => ({
				id: s.id,
				metadataType: s.metadata?.type,
				metadataId: s.metadata?.id,
			})),
		})
		for (const sub of subscriptions) {
			if (
				(sub?.metadata?.type === 'pyro' || sub?.metadata?.type === 'medal') &&
				sub.metadata.id === id
			) {
				subscription.value = sub
				debug('found matching subscription', {
					subscriptionId: sub.id,
				})
				break
			}
		}
		if (!subscription.value) {
			debug('no matching subscription found for id', id)
		}
	} else {
		debug('no id provided, resetting subscription')
		subscription.value = null
	}

	purchaseModal.value?.show(currentInterval.value)
}

defineExpose({
	open,
})
</script>
