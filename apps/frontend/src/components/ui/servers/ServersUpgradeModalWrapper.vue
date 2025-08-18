<template>
	<ModrinthServersPurchaseModal
		v-if="customer"
		ref="purchaseModal"
		:publishable-key="config.public.stripePublishableKey"
		:initiate-payment="async (body) => await initiatePayment(body)"
		:available-products="pyroProducts"
		:on-error="handleError"
		:customer="customer"
		:payment-methods="paymentMethods"
		:currency="selectedCurrency"
		:return-url="`${config.public.siteUrl}/servers/manage`"
		:pings="regionPings"
		:regions="regions"
		:refresh-payment-methods="fetchPaymentData"
		:fetch-stock="fetchStock"
		:plan-stage="true"
		:existing-plan="currentPlanFromSubscription"
		:existing-subscription="subscription || undefined"
		:on-finalize-no-payment-change="finalizeDowngrade"
		@hide="
			() => {
				subscription = null
			}
		"
	/>
</template>

<script setup lang="ts">
import { injectNotificationManager, ModrinthServersPurchaseModal } from '@modrinth/ui'
import type { ServerPlan } from '@modrinth/ui/src/utils/billing'
import type { UserSubscription } from '@modrinth/utils'
import { computed, onMounted, ref } from 'vue'

import { useServersFetch } from '~/composables/servers/servers-fetch.ts'
import { products } from '~/generated/state.json'

const { addNotification } = injectNotificationManager()

const config = useRuntimeConfig()
const purchaseModal = ref<InstanceType<typeof ModrinthServersPurchaseModal> | null>(null)
const customer = ref<any>(null)
const paymentMethods = ref<any[]>([])
const selectedCurrency = ref<string>('USD')
const regions = ref<any[]>([])
const regionPings = ref<any[]>([])

const pyroProducts = (products as any[])
	.filter((p) => p?.metadata?.type === 'pyro')
	.sort((a, b) => (a?.metadata?.ram ?? 0) - (b?.metadata?.ram ?? 0))

function handleError(err: any) {
	console.error('Purchase modal error:', err)
}

async function fetchPaymentData() {
	try {
		const [customerData, paymentMethodsData] = await Promise.all([
			useBaseFetch('billing/customer', { internal: true }),
			useBaseFetch('billing/payment_methods', { internal: true }),
		])
		customer.value = customerData as any
		paymentMethods.value = paymentMethodsData as any[]
	} catch (error) {
		console.error('Error fetching payment data:', error)
	}
}

function fetchStock(region: any, request: any) {
	return useServersFetch(`stock?region=${region.shortcode}`, {
		method: 'POST',
		body: {
			...request,
		},
		bypassAuth: true,
	}).then((res: any) => res.available as number)
}

function pingRegions() {
	useServersFetch('regions', {
		method: 'GET',
		version: 1,
		bypassAuth: true,
	}).then((res: any) => {
		regions.value = res as any[]
		;(regions.value as any[]).forEach((region: any) => {
			runPingTest(region)
		})
	})
}

const PING_COUNT = 20
const PING_INTERVAL = 200
const MAX_PING_TIME = 1000

function runPingTest(region: any, index = 1) {
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

const subscription = ref<UserSubscription | null>(null)
// Dry run state
const dryRunResponse = ref<{
	requires_payment: boolean
	required_payment_is_proration: boolean
} | null>(null)
const pendingDowngradeBody = ref<any | null>(null)
const currentPlanFromSubscription = computed<ServerPlan | undefined>(() => {
	return subscription.value
		? (pyroProducts.find(
				(p) =>
					p.prices.filter((price: { id: string }) => price.id === subscription.value?.price_id)
						.length > 0,
			) ?? undefined)
		: undefined
})

async function initiatePayment(body: any): Promise<any> {
	if (subscription.value) {
		const transformedBody = {
			interval: body.charge?.interval,
			payment_method: body.id,
			product: body.charge?.product_id,
			region: body.metadata?.server_region,
		}

		try {
			const dry = await useBaseFetch(`billing/subscription/${subscription.value.id}?dry=true`, {
				internal: true,
				method: 'PATCH',
				body: transformedBody,
			})

			if (dry && typeof dry === 'object' && 'requires_payment' in dry) {
				dryRunResponse.value = dry as any
				pendingDowngradeBody.value = transformedBody
				if (dry.requires_payment) {
					return await finalizeImmediate(transformedBody)
				} else {
					return null
				}
			} else {
				// Fallback if dry run not supported
				return await finalizeImmediate(transformedBody)
			}
		} catch (e) {
			console.error('Dry run failed, attempting immediate patch', e)
			return await finalizeImmediate(transformedBody)
		}
	} else {
		addNotification({
			title: 'Unable to determine subscription ID.',
			text: 'Please contact support.',
			type: 'error',
		})
		return Promise.reject(new Error('Unable to determine subscription ID.'))
	}
}

async function finalizeImmediate(body: any) {
	const result = await useBaseFetch(`billing/subscription/${subscription.value?.id}`, {
		internal: true,
		method: 'PATCH',
		body,
	})

	return result
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
	if (id) {
		const subscriptions = (await useBaseFetch(`billing/subscriptions`, {
			internal: true,
		})) as any[]
		for (const sub of subscriptions) {
			if (sub?.metadata?.id === id) {
				subscription.value = sub
				break
			}
		}
	} else {
		subscription.value = null
	}

	purchaseModal.value?.show('quarterly')
}

defineExpose({
	open,
})

onMounted(() => {
	fetchPaymentData()
	pingRegions()
})
</script>
