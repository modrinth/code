<template>
	<div class="flex flex-row gap-3">
		<div
			class="flex h-12 min-h-12 w-12 min-w-12 justify-center rounded-full border-[1px] border-solid border-button-bg bg-bg-raised !p-0 shadow-md"
		>
			<ArrowUpIcon class="my-auto size-8 text-secondary" />
		</div>
		<div class="flex w-full flex-row justify-between">
			<div class="flex flex-col">
				<span class="text-lg font-semibold text-contrast">{{
					formatMethodName(transaction.method)
				}}</span>
				<span class="text-secondary">
					{{ formatTransactionStatus(transaction.status) }} <BulletDivider />
					{{ $dayjs(transaction.created).format('MMM DD YYYY') }}
					<template v-if="transaction.fee">
						<BulletDivider /> Fee {{ $formatMoney(transaction.fee) }}
					</template>
					<template v-if="transaction.method_address">
						<BulletDivider /> {{ formatWallet(transaction.method) }} ({{
							transaction.method_address
						}})
					</template>
				</span>
			</div>
			<div class="my-auto flex flex-row items-end gap-4">
				<span class="text-lg font-semibold text-contrast">{{
					$formatMoney(transaction.amount)
				}}</span>
				<template v-if="transaction.status === 'in-transit'">
					<Tooltip theme="dismissable-prompt" :triggers="['hover', 'focus']" no-auto-focus>
						<ButtonStyled circular size="small">
							<button @click="cancelPayout">
								<XIcon />
							</button>
						</ButtonStyled>
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
import { ArrowUpIcon, XIcon } from '@modrinth/assets'
import { BulletDivider, ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { capitalizeString, formatWallet } from '@modrinth/utils'
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
		default:
			return capitalizeString(method)
	}
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
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
