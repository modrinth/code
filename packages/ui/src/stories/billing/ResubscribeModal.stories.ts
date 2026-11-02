import { type Archon, GenericModrinthClient, type Labrinth } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import type Stripe from 'stripe'
import { onMounted, ref } from 'vue'

import { Button } from '../../components/base/buttons'
import ResubscribeModal from '../../components/billing/ResubscribeModal.vue'
import { provideModrinthClient } from '../../providers'

const DAY_MS = 24 * 60 * 60 * 1000

const smallPlan: Labrinth.Billing.Internal.Product = {
	id: 'prod_small',
	metadata: { type: 'pyro', cpu: 4, ram: 4096, swap: 4096, storage: 32768 },
	prices: [
		{
			id: 'price_small_usd',
			product_id: 'prod_small',
			currency_code: 'USD',
			prices: {
				type: 'recurring',
				intervals: { 'five-days': 200, monthly: 1200, quarterly: 3000, yearly: 12000 },
			},
		},
	],
	unitary: false,
}

const regions: Archon.Servers.v1.Region[] = [
	{
		shortcode: 'us-vin',
		country_code: 'us',
		display_name: 'Vint Hill, Virginia',
		lat: 38.75,
		lon: -77.67,
		zone: 'modrinth.gg',
	},
]

const paymentMethods = [
	{
		id: 'pm_visa',
		type: 'card',
		card: { brand: 'visa', last4: '4242' },
		billing_details: { address: { country: 'GB' } },
	},
	{
		id: 'pm_mastercard',
		type: 'card',
		card: { brand: 'mastercard', last4: '5555' },
		billing_details: { address: { country: 'GB' } },
	},
] as unknown as Stripe.PaymentMethod[]

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

/**
 * Creates a client whose resubscribe preview and payment method endpoints return dummy data,
 * so the modal can be previewed without a labrinth that supports `?dry=true` resubscribes.
 */
function createMockClient({ due, fail = false }: { due: Date; fail?: boolean }) {
	const client = new GenericModrinthClient({ userAgent: 'modrinth-storybook/1.0.0' })
	const billing = client.labrinth.billing_internal
	let defaultPaymentMethod = paymentMethods[0]

	billing.previewResubscribe = async (): Promise<Labrinth.Billing.Internal.ResubscribePreview> => {
		await delay(600)
		if (fail) throw new Error('customer has no default payment method')

		const subtotal = 1200
		const tax = defaultPaymentMethod.id === 'pm_visa' ? 240 : 0
		return {
			subtotal,
			tax,
			total: subtotal + tax,
			currency_code: 'USD',
			interval: 'monthly',
			due: due.toISOString(),
			payment_method: defaultPaymentMethod,
		}
	}

	billing.editPaymentMethod = async (id) => {
		await delay(400)
		defaultPaymentMethod = paymentMethods.find((method) => method.id === id) ?? defaultPaymentMethod
	}

	return client
}

function renderStory({
	due,
	fail,
	wasSuspended,
}: {
	due: Date
	fail?: boolean
	wasSuspended: boolean
}) {
	return () => ({
		components: { ResubscribeModal, Button },
		setup() {
			provideModrinthClient(createMockClient({ due, fail }))

			const modalRef = ref<InstanceType<typeof ResubscribeModal> | null>(null)
			const lastEvent = ref<string | null>(null)

			function open() {
				modalRef.value?.show({
					subscriptionId: 'sub_dummy',
					wasSuspended,
					serverName: 'My server',
					plan: smallPlan,
					regionShortcode: 'us-vin',
				})
			}

			onMounted(open)

			return {
				modalRef,
				lastEvent,
				open,
				paymentMethods,
				regions,
				onAddPaymentMethod: () => (lastEvent.value = 'Add payment method clicked'),
			}
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<Button type="colored" color="brand" @click="open">Open resubscribe modal</Button>
				<p v-if="lastEvent" class="m-0 text-sm text-secondary">{{ lastEvent }}</p>
				<ResubscribeModal
					ref="modalRef"
					:payment-methods="paymentMethods"
					:regions="regions"
					:on-add-payment-method="onAddPaymentMethod"
					@resubscribe="lastEvent = 'Resubscribed'"
					@cancel="lastEvent = 'Cancelled'"
				/>
			</div>
		`,
	})
}

const meta = {
	title: 'Billing/ResubscribeModal',
	component: ResubscribeModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ResubscribeModal>

export default meta
type Story = StoryObj<typeof meta>

/** Cancelled but still within the paid period: charged on the next due date. */
export const RenewsOnDate: Story = {
	render: renderStory({ due: new Date(Date.now() + 12 * DAY_MS), wasSuspended: false }),
}

/** Past the due date and suspended: charged immediately on resubscribe. */
export const ChargedToday: Story = {
	render: renderStory({ due: new Date(Date.now() - 3 * DAY_MS), wasSuspended: true }),
}

/** The tax quote couldn't be calculated, e.g. no default payment method or billing address. */
export const PreviewError: Story = {
	render: renderStory({
		due: new Date(Date.now() + 12 * DAY_MS),
		fail: true,
		wasSuspended: false,
	}),
}
