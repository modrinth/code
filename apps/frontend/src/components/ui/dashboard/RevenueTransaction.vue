<template>
	<div class="flex flex-row gap-2 md:gap-3">
		<div
			class="flex h-10 min-h-10 w-10 min-w-10 justify-center rounded-full border-[1px] border-solid border-button-bg bg-bg-raised !p-0 shadow-md md:h-12 md:min-h-12 md:w-12 md:min-w-12"
		>
			<ArrowDownIcon v-if="isIncome" class="my-auto size-6 text-secondary md:size-8" />
			<ArrowUpIcon v-else class="my-auto size-6 text-secondary md:size-8" />
		</div>
		<div class="flex w-full flex-row justify-between">
			<div class="flex flex-col">
				<span class="text-base font-semibold text-contrast md:text-lg">{{
					isIncome
						? formatPayoutSource(transaction.payout_source)
						: formatMethodName(transaction.method_type || transaction.method)
				}}</span>
				<span class="text-xs text-secondary md:text-sm">
					<template v-if="!isIncome">
						<span
							:class="[
								transaction.status === 'cancelling' || transaction.status === 'cancelled'
									? 'text-red'
									: '',
							]"
							>{{ formatTransactionStatus(transaction.status) }} <BulletDivider
						/></span>
					</template>
					{{ $dayjs(transaction.created).format('MMM DD YYYY') }}
					<template v-if="!isIncome && transaction.fee">
						<BulletDivider /> Fee {{ formatMoney(transaction.fee) }}
					</template>
				</span>
			</div>
			<div class="my-auto flex flex-row items-center gap-2">
				<span
					class="text-base font-semibold md:text-lg"
					:class="isIncome ? 'text-green' : 'text-contrast'"
					>{{ formatMoney(transaction.amount) }}</span
				>
				<template v-if="!isIncome && transaction.status === 'in-transit'">
					<Tooltip theme="dismissable-prompt" :triggers="['hover', 'focus']" no-auto-focus>
						<span class="my-auto align-middle"
							><ButtonStyled circular size="small">
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

<script setup>
import { ArrowDownIcon, ArrowUpIcon, XIcon } from '@modrinth/assets'
import { BulletDivider, ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { capitalizeString, formatMoney } from '@modrinth/utils'
import { Tooltip } from 'floating-vue'

const props = defineProps({
	transaction: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits(['cancelled'])

const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const isIncome = computed(() => props.transaction.type === 'payout_available')

function formatTransactionStatus(status) {
	if (status === 'in-transit') return 'In Transit'
	return capitalizeString(status)
}

function formatMethodName(method) {
	if (!method) return 'Unknown'
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'tremendous':
			return 'Tremendous'
		case 'muralpay':
			return 'Muralpay'
		default:
			return capitalizeString(method)
	}
}

function formatPayoutSource(source) {
	if (!source) return 'Income'
	return source
		.split('_')
		.map((word) => capitalizeString(word))
		.join(' ')
}

async function cancelPayout() {
	startLoading()
	try {
		await useBaseFetch(`payout/${props.transaction.id}`, {
			method: 'DELETE',
			apiVersion: 3,
		})
		await useAuth(auth.value.token)
		emit('cancelled')
	} catch (err) {
		addNotification({
			title: 'Failed to cancel transaction',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
