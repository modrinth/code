<template>
	<div class="flex flex-col gap-3 sm:gap-4">
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
			<div
				class="flex min-h-[44px] items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5 sm:min-h-0"
			>
				<component
					:is="getCurrencyIcon(selectedRail.currency)"
					class="size-5 shrink-0"
					:class="getCurrencyColor(selectedRail.currency)"
				/>
				<span class="text-sm font-semibold text-contrast sm:text-[1rem]">{{
					selectedRail.currency
				}}</span>
			</div>
		</div>

		<div v-if="selectedRail?.type === 'fiat'" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.accountOwner) }}
				</span>
			</label>
			<div class="w-full rounded-[14px] bg-surface-2 p-3 sm:p-4">
				<div class="flex flex-col gap-1">
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ accountOwnerName }}
					</span>
					<span class="break-words text-xs text-primary sm:text-sm">
						{{ accountOwnerAddress }}
					</span>
				</div>
			</div>
		</div>

		<div v-if="selectedRail?.requiresBankName" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(formFieldLabels.bankName) }}
					<span class="text-red">*</span>
				</span>
			</label>

			<Combobox
				v-if="shouldShowBankNameDropdown"
				v-model="formData.bankName"
				:options="bankNameOptions"
				:searchable="true"
				:placeholder="formatMessage(formFieldPlaceholders.bankNamePlaceholderDropdown)"
				class="h-10"
			/>

			<input
				v-else
				v-model="formData.bankName"
				type="text"
				:placeholder="formatMessage(formFieldPlaceholders.bankNamePlaceholder)"
				autocomplete="off"
				class="w-full rounded-[14px] bg-surface-4 px-4 py-3 text-contrast placeholder:text-secondary sm:py-2.5"
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
				:autocomplete="field.autocomplete || 'off'"
				class="w-full rounded-[14px] bg-surface-4 px-4 py-3 text-contrast placeholder:text-secondary sm:py-2.5"
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
				class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
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
						autocomplete="off"
						class="w-full rounded-[14px] bg-surface-4 px-4 py-3 text-contrast placeholder:text-secondary sm:py-2.5"
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
			<div
				class="flex min-h-[44px] items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5 sm:min-h-0"
			>
				<component
					:is="getBlockchainIcon(selectedRail.blockchain)"
					class="size-5 shrink-0"
					:class="getBlockchainColor(selectedRail.blockchain)"
				/>
				<span class="text-sm font-semibold text-contrast sm:text-[1rem]">{{
					selectedRail.blockchain
				}}</span>
			</div>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(formFieldLabels.amount) }}
					<span class="text-red">*</span>
				</span>
			</label>
			<RevenueInputField
				v-model="formData.amount"
				:min-amount="effectiveMinAmount"
				:max-amount="effectiveMaxAmount"
			/>

			<WithdrawFeeBreakdown
				v-if="allRequiredFieldsFilled"
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
				:exchange-rate="exchangeRate"
				:local-currency="selectedRail?.currency"
			/>

			<Checkbox v-model="agreedTerms">
				<span
					><IntlFormatted :message-id="financialMessages.rewardsProgramTermsAgreement">
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/cmp" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template> </IntlFormatted
				></span>
			</Checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	Admonition,
	Checkbox,
	Combobox,
	financialMessages,
	formFieldLabels,
	formFieldPlaceholders,
	getBlockchainColor,
	getBlockchainIcon,
	getCurrencyColor,
	getCurrencyIcon,
	normalizeChildren,
} from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import RevenueInputField from '@/components/ui/dashboard/RevenueInputField.vue'
import WithdrawFeeBreakdown from '@/components/ui/dashboard/WithdrawFeeBreakdown.vue'
import { useGeneratedState } from '@/composables/generated'
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getRailConfig } from '@/utils/muralpay-rails'

const { withdrawData, maxWithdrawAmount, availableMethods, calculateFees } = useWithdrawContext()
const { formatMessage } = useVIntl()
const generatedState = useGeneratedState()

const selectedRail = computed(() => {
	const railId = withdrawData.value.selection.method
	return railId ? getRailConfig(railId) : null
})

const selectedMethodDetails = computed(() => {
	const methodId = withdrawData.value.selection.methodId
	if (!methodId) return null
	return availableMethods.value.find((m) => m.id === methodId) || null
})

const maxAmount = computed(() => maxWithdrawAmount.value)
const roundedMaxAmount = computed(() => Math.floor(maxAmount.value * 100) / 100)

const effectiveMinAmount = computed(
	() => selectedMethodDetails.value?.interval?.standard?.min || 0.01,
)
const effectiveMaxAmount = computed(() => {
	const apiMax = selectedMethodDetails.value?.interval?.standard?.max
	if (apiMax) {
		return Math.min(roundedMaxAmount.value, apiMax)
	}
	return roundedMaxAmount.value
})

const availableBankNames = computed(() => {
	const rail = selectedRail.value
	if (!rail || !rail.railCode) return []

	const bankDetails = generatedState.value.muralBankDetails?.[rail.railCode]
	return bankDetails?.bankNames || []
})

const shouldShowBankNameDropdown = computed(() => {
	return availableBankNames.value.length > 0
})

const bankNameOptions = computed(() => {
	return availableBankNames.value.map((name) => ({
		value: name,
		label: name,
	}))
})

const providerData = withdrawData.value.providerData
const existingAccountDetails = providerData.type === 'muralpay' ? providerData.accountDetails : {}
const existingAmount = withdrawData.value.calculation.amount
const formData = ref<Record<string, any>>({
	amount: existingAmount || undefined,
	bankName: existingAccountDetails?.bankName ?? '',
	...existingAccountDetails,
})

const agreedTerms = computed({
	get: () => withdrawData.value.agreedTerms,
	set: (value) => {
		withdrawData.value.agreedTerms = value
	},
})

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
		label: labelMap[documentType] || 'Document number',
		placeholder: placeholderMap[documentType] || 'Enter document number',
		required: true,
	}
})

const accountOwnerName = computed(() => {
	const providerDataValue = withdrawData.value.providerData
	if (providerDataValue.type !== 'muralpay') return ''
	const kycData = providerDataValue.kycData
	if (!kycData) return ''

	if (kycData.type === 'individual') {
		return `${kycData.firstName} ${kycData.lastName}`
	} else if (kycData.type === 'business') {
		return kycData.name
	}
	return ''
})

const accountOwnerAddress = computed(() => {
	const providerDataValue = withdrawData.value.providerData
	if (providerDataValue.type !== 'muralpay') return ''
	const kycData = providerDataValue.kycData
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

const allRequiredFieldsFilled = computed(() => {
	const rail = selectedRail.value
	if (!rail) return false

	const amount = formData.value.amount
	if (!amount || amount <= 0) return false

	if (rail.requiresBankName && !formData.value.bankName) return false

	const requiredFields = rail.fields.filter((f) => f.required)
	const allRequiredPresent = requiredFields.every((f) => {
		const value = formData.value[f.name]
		return value !== undefined && value !== null && value !== ''
	})

	if (!allRequiredPresent) return false

	if (dynamicDocumentNumberField.value?.required && !formData.value.documentNumber) return false

	return true
})

const calculateFeesDebounced = useDebounceFn(async () => {
	const amount = formData.value.amount
	const rail = selectedRail.value
	const providerDataValue = withdrawData.value.providerData
	const kycData = providerDataValue.type === 'muralpay' ? providerDataValue.kycData : null

	if (!amount || amount <= 0 || !rail || !kycData) {
		calculatedFee.value = null
		exchangeRate.value = null
		return
	}

	feeLoading.value = true
	try {
		await calculateFees()
		calculatedFee.value = withdrawData.value.calculation.fee
		exchangeRate.value = withdrawData.value.calculation.exchangeRate
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
		exchangeRate.value = null
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(
	formData,
	() => {
		withdrawData.value.calculation.amount = formData.value.amount ?? 0
		if (withdrawData.value.providerData.type === 'muralpay') {
			withdrawData.value.providerData.accountDetails = { ...formData.value }
		}

		if (allRequiredFieldsFilled.value) {
			feeLoading.value = true
			calculateFeesDebounced()
		} else {
			calculatedFee.value = null
			exchangeRate.value = null
			feeLoading.value = false
		}
	},
	{ deep: true },
)

if (allRequiredFieldsFilled.value) {
	feeLoading.value = true
	calculateFeesDebounced()
}

watch(
	() => withdrawData.value.selection.method,
	(newMethod, oldMethod) => {
		if (oldMethod && newMethod !== oldMethod) {
			formData.value = {
				amount: undefined,
				bankName: '',
			}
			if (withdrawData.value.providerData.type === 'muralpay') {
				withdrawData.value.providerData.accountDetails = {}
			}
			withdrawData.value.calculation.amount = 0
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
		defaultMessage: 'National ID number',
	},
	documentNumberPassport: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-passport',
		defaultMessage: 'Passport number',
	},
	documentNumberResidentId: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-resident-id',
		defaultMessage: 'Resident ID number',
	},
	documentNumberRuc: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-ruc',
		defaultMessage: 'RUC number',
	},
	documentNumberTaxId: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.document-number-tax-id',
		defaultMessage: 'Tax ID number',
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
