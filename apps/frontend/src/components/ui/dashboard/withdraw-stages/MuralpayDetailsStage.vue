<template>
	<div class="flex flex-col gap-4">
		<Admonition v-if="selectedRail?.warningMessage" type="warning"
			:header="formatMessage(messages.cryptoWarningHeader)">
			{{ selectedRail.warningMessage }}
		</Admonition>


		<div v-if="selectedRail?.type === 'crypto'" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.coin) }}
				</span>
			</label>
			<div class="flex items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5">

				<span class="font-semibold text-contrast">{{ selectedRail.currency }}</span>
				<span class="text-primary">{{ selectedRail.name }}</span>
			</div>
		</div>


		<div v-if="selectedRail?.type === 'fiat'" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.accountOwner) }}
				</span>
			</label>
			<div class="w-full rounded-[14px] bg-surface-2 p-4">
				<div class="flex flex-col gap-1">
					<span class="font-semibold text-contrast">
						{{ accountOwnerName }}
					</span>
					<span class="text-primary">
						{{ accountOwnerAddress }}
					</span>
				</div>
			</div>
		</div>


		<div v-if="selectedRail?.requiresBankName" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.bankName) }}
					<span class="text-brand-red">*</span>
				</span>
			</label>
			<input v-model="formData.bankName" type="text" :placeholder="formatMessage(messages.bankNamePlaceholder)"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
		</div>


		<div v-for="field in selectedRail?.fields" :key="field.name" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ field.label }}
					<span v-if="field.required" class="text-brand-red">*</span>
				</span>
			</label>


			<input v-if="['text', 'email', 'tel'].includes(field.type)" v-model="formData[field.name]" :type="field.type"
				:placeholder="field.placeholder" :pattern="field.pattern"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />


			<Combobox v-else-if="field.type === 'select'" v-model="formData[field.name]" :options="field.options || []"
				:placeholder="field.placeholder" class="h-10" />


			<input v-else-if="field.type === 'date'" v-model="formData[field.name]" type="date"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />


			<span v-if="field.helpText" class="text-sm text-secondary">
				{{ field.helpText }}
			</span>
		</div>


		<div v-if="selectedRail?.blockchain" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.network) }}
				</span>
			</label>
			<div class="flex items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5">

				<span class="font-semibold text-contrast">{{ selectedRail.blockchain }}</span>
			</div>
		</div>


		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.amount) }}
					<span class="text-brand-red">*</span>
				</span>
			</label>
			<div class="flex items-center gap-2">
				<input v-model.number="formData.amount" type="number" step="0.01" min="0" :max="maxAmount"
					:placeholder="formatMessage(messages.amountPlaceholder)"
					class="bg-raised flex-1 rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
				<ButtonStyled>
					<button class="px-4 py-2" @click="setMaxAmount">
						{{ formatMessage(messages.maxButton) }}
					</button>
				</ButtonStyled>
			</div>
			<span class="text-primary">
				{{ formatMoney(maxAmount) }} {{ formatMessage(messages.available) }}
			</span>


			<div class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownAmount) }}</span>
					<span class="font-semibold text-contrast">{{ formatMoney(formData.amount || 0) }}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownFee) }}</span>
					<span class="font-semibold text-contrast">-{{ formatMoney(0) }}</span>
				</div>
				<div class="h-px bg-surface-5" />
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownNetAmount) }}</span>
					<span class="font-semibold text-contrast">{{ formatMoney(formData.amount || 0) }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Admonition, ButtonStyled, Combobox } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getRailConfig } from '@/utils/muralpay-rails'

const withdrawContext = useWithdrawContext()
const { formatMessage } = useVIntl()

const selectedRail = computed(() => {
	const railId = withdrawContext.withdrawData.value.selectedMethod
	return railId ? getRailConfig(railId) : null
})

const maxAmount = computed(() => withdrawContext.maxWithdrawAmount.value)

const formData = ref<Record<string, any>>({
	amount: undefined,
	bankName: '',
})

const accountOwnerName = computed(() => {
	const kycData = withdrawContext.withdrawData.value.kycData
	if (!kycData) return ''

	if (kycData.type === 'individual') {
		return `${kycData.firstName} ${kycData.lastName}`
	} else if (kycData.type === 'business') {
		return kycData.name
	}
	return ''
})

const accountOwnerAddress = computed(() => {
	const kycData = withdrawContext.withdrawData.value.kycData
	if (!kycData || !kycData.physicalAddress) return ''

	const addr = kycData.physicalAddress
	const parts = [
		addr.address1,
		addr.address2,
		addr.city,
		addr.state,
		addr.zip,
		addr.country,
	].filter(Boolean)

	return parts.join(', ')
})

function setMaxAmount() {
	formData.value.amount = maxAmount.value
}

watch(
	formData,
	() => {
		withdrawContext.withdrawData.value.amount = formData.value.amount ?? 0
		withdrawContext.withdrawData.value.accountDetails = { ...formData.value }
	},
	{ deep: true },
)

const messages = defineMessages({
	accountOwner: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.account-owner',
		defaultMessage: 'Account owner',
	},
	bankName: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.bank-name',
		defaultMessage: 'Bank name',
	},
	bankNamePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.bank-name-placeholder',
		defaultMessage: 'Enter bank name',
	},
	amount: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.amount',
		defaultMessage: 'Amount',
	},
	amountPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.amount-placeholder',
		defaultMessage: 'Enter amount',
	},
	maxButton: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.max-button',
		defaultMessage: 'Max',
	},
	available: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.available',
		defaultMessage: 'available.',
	},
	cryptoWarningHeader: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.crypto-warning-header',
		defaultMessage: 'Confirm your wallet address',
	},
	coin: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.coin',
		defaultMessage: 'Coin',
	},
	network: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.network',
		defaultMessage: 'Network',
	},
	feeBreakdownAmount: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.fee-breakdown-amount',
		defaultMessage: 'Amount',
	},
	feeBreakdownFee: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.fee-breakdown-fee',
		defaultMessage: 'Fee',
	},
	feeBreakdownNetAmount: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.fee-breakdown-net-amount',
		defaultMessage: 'Net amount',
	},
})
</script>
