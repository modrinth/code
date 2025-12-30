<template>
	<div class="flex flex-row gap-2 md:gap-3">
		<div
			class="flex h-10 min-h-10 w-10 min-w-10 items-center justify-center rounded-full border-[1px] border-solid border-button-bg bg-bg-raised !p-0 shadow-md md:h-12 md:min-h-12 md:w-12 md:min-w-12"
		>
			<img
				v-if="methodIconUrl"
				:src="methodIconUrl"
				alt=""
				class="size-6 rounded-full object-cover md:size-8"
			/>
			<component
				:is="methodIconComponent"
				v-else-if="methodIconComponent"
				class="size-6 md:size-8"
			/>
			<ArrowDownIcon v-else-if="isIncome" class="size-6 text-secondary md:size-8" />
			<ArrowUpIcon v-else class="size-6 text-secondary md:size-8" />
		</div>
		<div class="flex w-full flex-row justify-between">
			<div class="flex flex-col">
				<span class="text-base font-semibold text-contrast md:text-lg">{{
					transaction.type === 'payout_available'
						? formatPayoutSource(transaction.payout_source)
						: formatMethodName(transaction.method_type || transaction.method, transaction.method_id)
				}}</span>
				<span class="text-xs text-secondary md:text-sm">
					<template v-if="transaction.type === 'withdrawal'">
						<span
							:class="[
								transaction.status === 'cancelling' || transaction.status === 'cancelled'
									? 'text-red'
									: '',
							]"
							>{{ formatTransactionStatus(transaction.status) }} <BulletDivider
						/></span>
					</template>
					{{ dayjs(transaction.created).format('MMM DD YYYY') }}
					<template v-if="transaction.type === 'withdrawal' && transaction.fee">
						<BulletDivider /> Fee {{ formatMoney(transaction.fee) }}
					</template>
				</span>
			</div>
			<div class="my-auto flex flex-row items-center gap-2">
				<span
					class="text-base font-semibold md:text-lg"
					:class="isIncome ? 'text-green' : 'text-contrast'"
					>{{ isIncome ? '' : '-' }}{{ formatMoney(transaction.amount) }}</span
				>
				<template v-if="transaction.type === 'withdrawal' && transaction.status === 'in-transit'">
					<Tooltip theme="dismissable-prompt" :triggers="['hover', 'focus']" no-auto-focus>
						<span class="my-auto align-middle"
							><ButtonStyled circular type="outlined" size="small">
								<button class="align-middle" @click="cancelPayout">
									<XIcon />
								</button> </ButtonStyled
						></span>
						<template #popper>
							<div class="font-semibold text-contrast">Cancel transaction</div>
						</template>
					</Tooltip>
				</template>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ArrowDownIcon,
	ArrowUpIcon,
	LandmarkIcon,
	PayPalColorIcon,
	VenmoColorIcon,
	XIcon,
} from '@modrinth/assets'
import {
	BulletDivider,
	ButtonStyled,
	getCurrencyIcon,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString, formatMoney } from '@modrinth/utils'
import dayjs from 'dayjs'
import { Tooltip } from 'floating-vue'

import { useGeneratedState } from '~/composables/generated'
import { findRail } from '~/utils/muralpay-rails'

type PayoutStatus = 'in-transit' | 'cancelling' | 'cancelled' | 'success' | 'failed'
type PayoutMethodType = 'paypal' | 'venmo' | 'tremendous' | 'muralpay'
type PayoutSource = 'creator_rewards' | 'affilites'

type WithdrawalTransaction = {
	type: 'withdrawal'
	id: string
	status: PayoutStatus
	created: string
	amount: number
	fee?: number | null
	method_type?: PayoutMethodType | null
	method?: string
	method_id?: string
	method_address?: string | null
}

type PayoutAvailableTransaction = {
	type: 'payout_available'
	created: string
	payout_source: PayoutSource
	amount: number
}

type Transaction = WithdrawalTransaction | PayoutAvailableTransaction

const props = defineProps<{
	transaction: Transaction
}>()

const emit = defineEmits<{
	(e: 'cancelled'): void
}>()

const { addNotification } = injectNotificationManager()
const generatedState = useGeneratedState()

const isIncome = computed(() => props.transaction.type === 'payout_available')

const methodIconUrl = computed(() => {
	if (props.transaction.type !== 'withdrawal') return null
	const method = props.transaction.method_type || props.transaction.method
	const methodId = props.transaction.method_id

	if (method === 'tremendous' && methodId) {
		const methodInfo = generatedState.value.tremendousIdMap?.[methodId]
		if (methodInfo?.name?.toLowerCase()?.includes('paypal')) return null
		return methodInfo?.image_url ?? null
	}

	return null
})

const methodIconComponent = computed(() => {
	if (props.transaction.type !== 'withdrawal') return null
	const method = props.transaction.method_type || props.transaction.method
	switch (method) {
		case 'paypal':
			return PayPalColorIcon
		case 'tremendous': {
			const methodId = props.transaction.method_id
			if (methodId) {
				const info = generatedState.value.tremendousIdMap?.[methodId]
				if (info?.name?.toLowerCase()?.includes('paypal')) {
					return PayPalColorIcon
				}
			}
			return null
		}
		case 'venmo':
			return VenmoColorIcon
		case 'muralpay': {
			const methodId = props.transaction.method_id
			if (methodId) {
				const rail = findRail(methodId)
				if (rail) {
					if (rail.type === 'crypto') {
						const currencyIcon = getCurrencyIcon(rail.currency)
						if (currencyIcon) return currencyIcon
					}
					if (rail.type === 'fiat') {
						const currencyIcon = getCurrencyIcon(rail.currency)
						if (currencyIcon) return currencyIcon
						return LandmarkIcon
					}
				}
			}
			return null
		}
		default:
			return null
	}
})

function formatTransactionStatus(status: string): string {
	if (status === 'in-transit') return 'In Transit'
	return capitalizeString(status)
}

const { formatMessage } = useVIntl()

function formatMethodName(method: string | undefined, method_id: string | undefined): string {
	if (!method) return 'Unknown'
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'tremendous':
			if (method_id) {
				const info = generatedState.value.tremendousIdMap?.[method_id]
				if (info) return `${info.name}`
			}
			return 'Tremendous'
		case 'muralpay':
			if (method_id) {
				const rail = findRail(method_id)
				if (rail) {
					return formatMessage(rail.name)
				}
			}
			return 'Mural Pay (Unknown)'
		default:
			return capitalizeString(method)
	}
}

function formatPayoutSource(source: string | undefined): string {
	if (!source) return 'Income'
	return source
		.split('_')
		.map((word: string) => capitalizeString(word))
		.join(' ')
}

async function cancelPayout(): Promise<void> {
	startLoading()
	try {
		const transaction = props.transaction
		if (transaction.type !== 'withdrawal') return

		await useBaseFetch(`payout/${transaction.id}`, {
			method: 'DELETE',
			apiVersion: 3,
		})
		await useAuth()
		emit('cancelled')
	} catch (err: any) {
		addNotification({
			title: 'Failed to cancel transaction',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
