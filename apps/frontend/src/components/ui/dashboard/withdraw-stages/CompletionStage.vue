<template>
	<div class="flex flex-col items-center gap-6">
		<div class="flex w-full items-center justify-center gap-2.5">
			<span class="text-nowrap text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</span>
		</div>
		<div class="flex w-full flex-col gap-3">
			<div class="span-4 flex w-full flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.method) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ result?.methodType || 'N/A' }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.recipient) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ result?.recipientDisplay || 'N/A' }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.date) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formattedDate }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.amount) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatCurrency(result?.amount || 0) }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.fee) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatCurrency(result?.fee || 0) }}
					</span>
				</div>
				<div class="border-b-1 h-0 w-full rounded-full border-b border-solid border-divider" />
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.netAmount) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatCurrency(result?.netAmount || 0) }}
					</span>
				</div>
			</div>
		</div>
		<span
			v-if="withdrawData.providerData.type === 'tremendous'"
			class="w-full text-center text-base font-normal text-primary"
		>
			<IntlFormatted
				:message-id="messages.emailConfirmation"
				:values="{ email: withdrawData.result?.recipientDisplay }"
			>
				<template #b="{ children }">
					<strong>
						<component :is="() => normalizeChildren(children)" />
					</strong>
				</template>
			</IntlFormatted>
		</span>
		<div class="flex w-full gap-3">
			<ButtonStyled class="flex-1">
				<button class="w-full text-contrast" @click="handleClose">
					{{ formatMessage(messages.closeButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled class="flex-1">
				<button class="w-full text-contrast" @click="handleViewTransactions">
					{{ formatMessage(messages.transactionsButton) }}
				</button>
			</ButtonStyled>
		</div>
		<Teleport to="body">
			<div
				v-if="showConfetti"
				class="pointer-events-none fixed inset-0 z-[9999] flex items-center justify-center"
			>
				<ConfettiExplosion />
			</div>
		</Teleport>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import dayjs from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import ConfettiExplosion from 'vue-confetti-explosion'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { normalizeChildren } from '@/utils/vue-children.ts'

const { withdrawData } = useWithdrawContext()
const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'close' | 'view-transactions'): void
}>()

const result = computed(() => withdrawData.value.result)

const showConfetti = ref(false)

onMounted(() => {
	showConfetti.value = true
	setTimeout(() => {
		showConfetti.value = false
	}, 5000)
})

const formattedDate = computed(() => {
	if (!result.value?.created) return 'N/A'
	return dayjs(result.value.created).format('MMMM D, YYYY')
})

function formatCurrency(amount: number): string {
	return new Intl.NumberFormat('en-US', {
		style: 'currency',
		currency: 'USD',
		minimumFractionDigits: 2,
		maximumFractionDigits: 2,
	}).format(amount)
}

function handleClose() {
	emit('close')
}

function handleViewTransactions() {
	emit('view-transactions')
}

const messages = defineMessages({
	title: {
		id: 'dashboard.withdraw.completion.title',
		defaultMessage: 'Withdraw complete',
	},
	method: {
		id: 'dashboard.withdraw.completion.method',
		defaultMessage: 'Method',
	},
	recipient: {
		id: 'dashboard.withdraw.completion.recipient',
		defaultMessage: 'Recipient',
	},
	date: {
		id: 'dashboard.withdraw.completion.date',
		defaultMessage: 'Date',
	},
	amount: {
		id: 'dashboard.withdraw.completion.amount',
		defaultMessage: 'Amount',
	},
	fee: {
		id: 'dashboard.withdraw.completion.fee',
		defaultMessage: 'Fee',
	},
	netAmount: {
		id: 'dashboard.withdraw.completion.net-amount',
		defaultMessage: 'Net amount',
	},
	emailConfirmation: {
		id: 'dashboard.withdraw.completion.email-confirmation',
		defaultMessage:
			'You will receive an email at <b>{email}</b> from our partner, Tremendous, with instructions to redeem your withdrawal.',
	},
	closeButton: {
		id: 'dashboard.withdraw.completion.close-button',
		defaultMessage: 'Close',
	},
	transactionsButton: {
		id: 'dashboard.withdraw.completion.transactions-button',
		defaultMessage: 'Transactions',
	},
})
</script>
