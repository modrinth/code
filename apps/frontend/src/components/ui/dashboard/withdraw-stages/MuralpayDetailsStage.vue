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
				<component class="size-5" :class="getCurrencyColor(selectedRail.currency)"
					:is="getCurrencyIcon(selectedRail.currency)" />
				<span class="font-semibold text-contrast">{{ selectedRail.currency }}</span>
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
				<component class="size-5" :class="getBlockchainColor(selectedRail.blockchain)"
					:is="getBlockchainIcon(selectedRail.blockchain)" />
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
					<span class="font-semibold text-contrast">
						<template v-if="feeLoading">
							<LoaderCircleIcon class="animate-spin size-4" />
						</template>
						<template v-else>-{{ formatMoney(calculatedFee || 0) }}</template>
					</span>
				</div>
				<div class="h-px bg-surface-5" />
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownNetAmount) }}</span>
					<span class="font-semibold text-contrast">{{ formatMoney(netAmount) }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Admonition, ButtonStyled, Combobox } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getBlockchainColor, getBlockchainIcon, getCurrencyColor, getCurrencyIcon } from '@/utils/finance-icons.ts'
import { getRailConfig } from '@/utils/muralpay-rails'
import { LoaderCircleIcon } from '@modrinth/assets'

const withdrawContext = useWithdrawContext()
const { formatMessage } = useVIntl()

const selectedRail = computed(() => {
	const railId = withdrawContext.withdrawData.value.selectedMethod
	return railId ? getRailConfig(railId) : null
})

console.log(selectedRail);

const maxAmount = computed(() => withdrawContext.maxWithdrawAmount.value)

const formData = ref<Record<string, any>>({
	amount: undefined,
	bankName: '',
})

const calculatedFee = ref<number | null>(null)
const feeLoading = ref(false)

const netAmount = computed(() => {
	const amount = formData.value.amount || 0
	const fee = calculatedFee.value || 0
	return Math.max(0, amount - fee)
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

const calculateFees = useDebounceFn(async () => {
	const amount = formData.value.amount
	const rail = selectedRail.value
	const kycData = withdrawContext.withdrawData.value.kycData

	if (!amount || amount <= 0 || !rail || !kycData) {
		calculatedFee.value = null
		return
	}

	feeLoading.value = true
	try {
		let payout_details

		if (rail.type === 'crypto') {
			payout_details = {
				type: 'blockchain',
				wallet_address: formData.value.walletAddress || '0x0000000000000000000000000000000000000000'
			}
		} else {
			const fiatAndRailDetails: Record<string, any> = {
				type: rail.railCode || '',
				symbol: rail.currency || '',
			}

			for (const field of rail.fields) {
				const value = formData.value[field.name]
				if (value !== undefined && value !== null && value !== '') {
					fiatAndRailDetails[field.name] = value
				}
			}

			payout_details = {
				type: 'fiat',
				bank_name: formData.value.bankName || '',
				bank_account_owner: kycData.type === 'individual'
					? `${kycData.firstName} ${kycData.lastName}`
					: kycData.name || '',
				fiat_and_rail_details: fiatAndRailDetails
			}
		}

		const recipient_info = {
			type: kycData.type,
			...(kycData.type === 'individual' ? {
				firstName: kycData.firstName,
				lastName: kycData.lastName,
				dateOfBirth: kycData.dateOfBirth,
			} : {
				name: kycData.name,
			}),
			email: kycData.email,
			physicalAddress: kycData.physicalAddress
		}

		const payload = {
			amount,
			method: 'muralpay',
			method_details: {
				payout_details,
				recipient_info
			},
			method_id: 'muralpay'
		}

		const response = await useBaseFetch('payout/fees', {
			apiVersion: 3,
			method: 'POST',
			body: payload
		}) as { fee: number | null }

		calculatedFee.value = response.fee || 0
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(
	formData,
	() => {
		withdrawContext.withdrawData.value.amount = formData.value.amount ?? 0
		withdrawContext.withdrawData.value.accountDetails = { ...formData.value }

		if (formData.value.amount) {
			calculateFees()
		}
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
