<template>
	<div class="relative overflow-clip rounded-xl bg-bg px-4 py-3">
		<div
			class="absolute bottom-0 left-0 top-0 w-1"
			:class="
				charge.type === 'refund' ? 'bg-purple' : (chargeStatuses[charge.status]?.color ?? 'bg-blue')
			"
		/>
		<div class="grid w-full grid-cols-[1fr_auto] items-center gap-4">
			<div class="flex flex-col gap-2">
				<span>
					<span class="font-bold text-contrast">
						<template v-if="charge.status === 'succeeded'"> Succeeded </template>
						<template v-else-if="charge.status === 'failed'"> Failed </template>
						<template v-else-if="charge.status === 'cancelled'"> Cancelled </template>
						<template v-else-if="charge.status === 'processing'"> Processing </template>
						<template v-else-if="charge.status === 'open'"> Upcoming </template>
						<template v-else-if="charge.status === 'expiring'"> Expiring </template>
						<template v-else> {{ charge.status }} </template>
					</span>
					<span class="text-secondary opacity-50">•</span>
					<span>
						<template v-if="charge.type === 'refund'"> Refund </template>
						<template v-else-if="charge.type === 'subscription'">
							<template v-if="charge.status === 'cancelled'"> Subscription </template>
							<template v-else-if="isLatestCharge"> Started subscription </template>
							<template v-else> Subscription renewal </template>
						</template>
						<template v-else-if="charge.type === 'one-time'"> One-time charge </template>
						<template v-else-if="charge.type === 'proration'"> Proration charge </template>
						<template v-else> {{ charge.status }} </template>
					</span>
					<template v-if="charge.status !== 'cancelled'">
						<span class="text-secondary opacity-50">•</span>
						{{ formatPrice(charge.amount, charge.currency_code) }}
					</template>
				</span>
				<span
					v-if="productMetadata && productMetadata.type === 'pyro'"
					class="flex items-center gap-1 text-sm text-secondary"
				>
					<span class="font-bold">Product:</span>
					<span v-if="productMetadata.ram">{{ productMetadata.ram / 1024 }}GB RAM</span>
					<span v-else>Unknown RAM</span>
					<span class="text-secondary opacity-50">•</span>
					<span v-if="productMetadata.cpu">{{ productMetadata.cpu }} vCPU</span>
					<span v-else>Unknown CPU</span>
					<span class="text-secondary opacity-50">•</span>
					<span v-if="productMetadata.storage">{{ productMetadata.storage / 1024 }}GB Storage</span>
					<span v-else>Unknown Storage</span>
					<span class="text-secondary opacity-50">•</span>
					<span v-if="productMetadata.swap">{{ productMetadata.swap }}MB Swap</span>
					<span v-else>Unknown Swap</span>
				</span>
				<span class="text-sm text-secondary">
					<span
						v-if="charge.status === 'cancelled' && dayjs(charge.due).isBefore(dayjs())"
						class="font-bold"
					>
						Ended:
					</span>
					<span v-else-if="charge.status === 'cancelled'" class="font-bold">Ends:</span>
					<span v-else-if="charge.type === 'refund'" class="font-bold">Issued:</span>
					<span v-else class="font-bold">Due:</span>
					{{ formatDateTime(charge.due) }}
					<span class="text-secondary">({{ formatRelativeTime(charge.due) }}) </span>
				</span>
				<span v-if="charge.last_attempt != null" class="text-sm text-secondary">
					<span v-if="charge.status === 'failed'" class="font-bold">Last attempt:</span>
					<span v-else class="font-bold">Charged:</span>
					{{ formatDateTime(charge.last_attempt) }}
					<span class="text-secondary">({{ formatRelativeTime(charge.last_attempt) }}) </span>
				</span>
				<div class="flex w-full items-center gap-1 text-xs text-secondary">
					{{ charge.status }}
					<span class="text-secondary opacity-50">•</span>
					{{ charge.type }}
					<span class="text-secondary opacity-50">•</span>
					{{ formatPrice(charge.amount, charge.currency_code) }}
					<span class="text-secondary opacity-50">•</span>

					{{ formatDateTimeShort(charge.due) }}
					<template v-if="charge.subscription_interval">
						<span class="text-secondary opacity-50">•</span>
						{{ charge.subscription_interval }}
					</template>
				</div>
			</div>
			<div class="flex gap-2">
				<ButtonStyled v-if="isRefunded">
					<div class="button-like disabled"><CheckIcon /> Charge refunded</div>
				</ButtonStyled>
				<ButtonStyled
					v-else-if="charge.status === 'succeeded' && charge.type !== 'refund'"
					color="red"
					color-fill="text"
				>
					<button @click="emit('refund', charge)">
						<CurrencyIcon />
						Refund options
					</button>
				</ButtonStyled>
				<ButtonStyled
					v-else-if="charge.status === 'failed' || charge.status === 'open'"
					color="red"
					color-fill="text"
				>
					<button @click="emit('modify', charge, subscription)">
						<CurrencyIcon />
						Modify charge
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, CurrencyIcon } from '@modrinth/assets'
import { ButtonStyled, useFormatDateTime, useFormatPrice, useRelativeTime } from '@modrinth/ui'
import dayjs from 'dayjs'

import { products } from '~/generated/state.json'

const props = defineProps<{
	charge: Labrinth.Billing.Internal.Charge
	subscription: Labrinth.Billing.Internal.UserSubscription
	allCharges: Labrinth.Billing.Internal.Charge[]
	chargeIndex: number
	chargeCount: number
}>()

const emit = defineEmits<{
	refund: [charge: Labrinth.Billing.Internal.Charge]
	modify: [
		charge: Labrinth.Billing.Internal.Charge,
		subscription: Labrinth.Billing.Internal.UserSubscription,
	]
}>()

const formatPrice = useFormatPrice()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatDateTimeShort = useFormatDateTime({
	year: 'numeric',
	month: '2-digit',
	day: '2-digit',
	hour: 'numeric',
	minute: 'numeric',
})
const formatRelativeTime = useRelativeTime()

const isLatestCharge = computed(() => props.chargeIndex === props.chargeCount - 1)

const isRefunded = computed(() =>
	props.allCharges.some(
		(charge) => charge.type === 'refund' && charge.parent_charge_id === props.charge.id,
	),
)

const productMetadata = computed(
	() =>
		products.find((product) => product.prices.some((price) => price.id === props.charge.price_id))
			?.metadata,
)

const chargeStatuses = {
	open: {
		color: 'bg-blue',
	},
	processing: {
		color: 'bg-orange',
	},
	succeeded: {
		color: 'bg-green',
	},
	failed: {
		color: 'bg-red',
	},
	cancelled: {
		color: 'bg-red',
	},
	expiring: {
		color: 'bg-orange',
	},
}
</script>
