<template>
	<div class="flex flex-col gap-4">
		<Admonition
			v-if="selectedRail?.warningMessage"
			type="warning"
			:header="formatMessage(messages.cryptoWarningHeader)"
		>
			{{ formatMessage(selectedRail.warningMessage) }}
		</Admonition>

		<div v-if="selectedRail?.type === 'crypto'" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.coin) }}
				</span>
			</label>
			<div class="flex items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5">
				<component
					:is="getCurrencyIcon(selectedRail.currency)"
					class="size-5"
					:class="getCurrencyColor(selectedRail.currency)"
				/>
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
					<span class="text-red">*</span>
				</span>
			</label>
			<input
				v-model="formData.bankName"
				type="text"
				:placeholder="formatMessage(messages.bankNamePlaceholder)"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
			/>
		</div>

		<div v-for="field in selectedRail?.fields" :key="field.name" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(field.label) }}
					<span v-if="field.required" class="text-red">*</span>
				</span>
			</label>

			<input
				v-if="['text', 'email', 'tel'].includes(field.type)"
				v-model="formData[field.name]"
				:type="field.type"
				:placeholder="field.placeholder ? formatMessage(field.placeholder) : undefined"
				:pattern="field.pattern"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
			/>

			<Combobox
				v-else-if="field.type === 'select'"
				v-model="formData[field.name]"
				:options="
					(field.options || []).map((opt) => ({
						value: opt.value,
						label: formatMessage(opt.label),
					}))
				"
				:placeholder="field.placeholder ? formatMessage(field.placeholder) : undefined"
				class="h-10"
			/>

			<input
				v-else-if="field.type === 'date'"
				v-model="formData[field.name]"
				type="date"
				class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
			/>

			<span v-if="field.helpText" class="text-sm text-secondary">
				{{ formatMessage(field.helpText) }}
			</span>
		</div>

		<Transition
			enter-active-class="transition-all duration-300 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-40"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-40"
			leave-to-class="opacity-0 max-h-0"
		>
			<div v-if="dynamicDocumentNumberField" class="overflow-hidden">
				<div class="flex flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ dynamicDocumentNumberField.label }}
							<span v-if="dynamicDocumentNumberField.required" class="text-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.documentNumber"
						:type="dynamicDocumentNumberField.type"
						:placeholder="dynamicDocumentNumberField.placeholder"
						class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
			</div>
		</Transition>

		<div v-if="selectedRail?.blockchain" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.network) }}
				</span>
			</label>
			<div class="flex items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5">
				<component
					:is="getBlockchainIcon(selectedRail.blockchain)"
					class="size-5"
					:class="getBlockchainColor(selectedRail.blockchain)"
				/>
				<span class="font-semibold text-contrast">{{ selectedRail.blockchain }}</span>
			</div>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.amount) }}
					<span class="text-red">*</span>
				</span>
			</label>
			<div class="flex items-center gap-2">
				<div class="relative flex-1">
					<input
						v-model.number="formData.amount"
						type="number"
						step="0.01"
						min="0.01"
						:max="roundedMaxAmount"
						:placeholder="formatMessage(messages.amountPlaceholder)"
						class="bg-raised w-full rounded-[14px] py-2.5 pl-8 pr-4 text-contrast placeholder:text-secondary"
						@input="enforceDecimalPlaces"
					/>
				</div>
				<ButtonStyled>
					<button class="px-4 py-2" @click="setMaxAmount">
						{{ formatMessage(messages.maxButton) }}
					</button>
				</ButtonStyled>
			</div>
			<span class="text-secondary">
				{{ formatMoney(roundedMaxAmount) }} {{ formatMessage(messages.available) }}
			</span>

			<WithdrawFeeBreakdown
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
				:exchange-rate="exchangeRate"
				:local-currency="selectedRail?.currency"
			/>

			<Checkbox v-model="agreedTerms" class="rewards-checkbox">
				<span
					>I agree to the
					<nuxt-link to="/legal/cmp" class="text-link">Rewards Program Terms</nuxt-link></span
				>
			</Checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Admonition, ButtonStyled, Checkbox, Combobox } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import WithdrawFeeBreakdown from '@/components/ui/dashboard/WithdrawFeeBreakdown.vue'
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import {
	getBlockchainColor,
	getBlockchainIcon,
	getCurrencyColor,
	getCurrencyIcon,
} from '@/utils/finance-icons.ts'
import { getRailConfig } from '@/utils/muralpay-rails'

const withdrawContext = useWithdrawContext()
const { formatMessage } = useVIntl()

const selectedRail = computed(() => {
	const railId = withdrawContext.withdrawData.value.selectedMethod
	return railId ? getRailConfig(railId) : null
})

const maxAmount = computed(() => withdrawContext.maxWithdrawAmount.value)
const roundedMaxAmount = computed(() => Math.floor(maxAmount.value * 100) / 100)

// if user has switched stages use what was in withdraw context
const existingAccountDetails = withdrawContext.withdrawData.value.accountDetails
const existingAmount = withdrawContext.withdrawData.value.amount
const formData = ref<Record<string, any>>({
	amount: existingAmount || undefined,
	bankName: existingAccountDetails?.bankName ?? '',
	...existingAccountDetails,
})

const agreedTerms = ref<boolean>(false)

const calculatedFee = ref<number | null>(null)
const exchangeRate = ref<number | null>(null)
const feeLoading = ref(false)

const hasDocumentTypeField = computed(() => {
	const rail = selectedRail.value
	if (!rail) return false
	return rail.fields.some((field) => field.name === 'documentType')
})

const dynamicDocumentNumberField = computed(() => {
	if (!hasDocumentTypeField.value) return null

	const documentType = formData.value.documentType
	if (!documentType) return null

	const labelMap: Record<string, string> = {
		NATIONAL_ID: formatMessage(messages.documentNumberNationalId),
		PASSPORT: formatMessage(messages.documentNumberPassport),
		RESIDENT_ID: formatMessage(messages.documentNumberResidentId),
		RUC: formatMessage(messages.documentNumberRuc),
		TAX_ID: formatMessage(messages.documentNumberTaxId),
	}

	const placeholderMap: Record<string, string> = {
		NATIONAL_ID: formatMessage(messages.documentNumberNationalIdPlaceholder),
		PASSPORT: formatMessage(messages.documentNumberPassportPlaceholder),
		RESIDENT_ID: formatMessage(messages.documentNumberResidentIdPlaceholder),
		RUC: formatMessage(messages.documentNumberRucPlaceholder),
		TAX_ID: formatMessage(messages.documentNumberTaxIdPlaceholder),
	}

	return {
		name: 'documentNumber',
		type: 'text' as const,
		label: labelMap[documentType] || 'Document Number',
		placeholder: placeholderMap[documentType] || 'Enter document number',
		required: true,
	}
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
	formData.value.amount = roundedMaxAmount.value
}

function enforceDecimalPlaces(event: Event) {
	const input = event.target as HTMLInputElement
	const value = input.value

	if (value && value.includes('.')) {
		const parts = value.split('.')
		if (parts[1] && parts[1].length > 2) {
			const rounded = Math.floor(parseFloat(value) * 100) / 100
			formData.value.amount = rounded
			input.value = rounded.toString()
		}
	}
}

const calculateFees = useDebounceFn(async () => {
	const amount = formData.value.amount
	const rail = selectedRail.value
	const kycData = withdrawContext.withdrawData.value.kycData

	if (!amount || amount <= 0 || !rail || !kycData) {
		calculatedFee.value = null
		exchangeRate.value = null
		return
	}

	feeLoading.value = true
	try {
		let payout_details

		if (rail.type === 'crypto') {
			payout_details = {
				type: 'blockchain',
				wallet_address: formData.value.walletAddress || null,
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
				bank_account_owner:
					kycData.type === 'individual'
						? `${kycData.firstName} ${kycData.lastName}`
						: kycData.name || '',
				fiat_and_rail_details: fiatAndRailDetails,
			}
		}

		const recipient_info = {
			type: kycData.type,
			...(kycData.type === 'individual'
				? {
						firstName: kycData.firstName,
						lastName: kycData.lastName,
						dateOfBirth: kycData.dateOfBirth,
					}
				: {
						name: kycData.name,
					}),
			email: kycData.email,
			physicalAddress: kycData.physicalAddress,
		}

		const payload = {
			amount,
			method: 'muralpay',
			method_details: {
				payout_details,
				recipient_info,
			},
			method_id: 'muralpay',
		}

		const response = (await useBaseFetch('payout/fees', {
			apiVersion: 3,
			method: 'POST',
			body: payload,
		})) as { fee: number | null; exchange_rate: number | null }

		calculatedFee.value = response.fee || 0
		exchangeRate.value = response.exchange_rate || null
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
		exchangeRate.value = null
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(
	() => formData.value.amount,
	(newAmount) => {
		if (newAmount !== undefined && newAmount !== null) {
			if (newAmount > roundedMaxAmount.value) {
				formData.value.amount = roundedMaxAmount.value
				return
			}
			if (newAmount < 0) {
				formData.value.amount = 0
				return
			}
		}
	},
)

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

if (formData.value.amount) {
	calculateFees()
}

// reset all the values if the selected method changes
watch(
	() => withdrawContext.withdrawData.value.selectedMethod,
	(newMethod, oldMethod) => {
		if (oldMethod && newMethod !== oldMethod) {
			formData.value = {
				amount: undefined,
				bankName: '',
			}
			withdrawContext.withdrawData.value.accountDetails = {}
			withdrawContext.withdrawData.value.amount = 0
			calculatedFee.value = null
			exchangeRate.value = null
		}
	},
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
	documentNumberNationalId: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-national-id',
		defaultMessage: 'National ID Number',
	},
	documentNumberPassport: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-passport',
		defaultMessage: 'Passport Number',
	},
	documentNumberResidentId: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-resident-id',
		defaultMessage: 'Resident ID Number',
	},
	documentNumberRuc: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-ruc',
		defaultMessage: 'RUC Number',
	},
	documentNumberTaxId: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-tax-id',
		defaultMessage: 'Tax ID Number',
	},
	documentNumberNationalIdPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-national-id-placeholder',
		defaultMessage: 'Enter national ID number',
	},
	documentNumberPassportPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-passport-placeholder',
		defaultMessage: 'Enter passport number',
	},
	documentNumberResidentIdPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-resident-id-placeholder',
		defaultMessage: 'Enter resident ID number',
	},
	documentNumberRucPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-ruc-placeholder',
		defaultMessage: 'Enter RUC number',
	},
	documentNumberTaxIdPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-tax-id-placeholder',
		defaultMessage: 'Enter tax ID number',
	},
})
</script>
