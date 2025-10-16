<template>
	<div class="flex flex-col gap-4">
		<template v-if="withdrawContext.withdrawData.value.selectedMethod === 'bank'">
			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.accountOwner) }}
					</span>
				</label>
				<div class="bg-surface-2 w-full rounded-[14px] p-4">
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

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.bankName) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input v-model="formData.bankName" type="text" :placeholder="formatMessage(messages.bankNamePlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.accountNumber) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input v-model="formData.accountNumber" type="text"
					:placeholder="formatMessage(messages.accountNumberPlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.routingNumber) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input v-model="formData.routingNumber" type="text"
					:placeholder="formatMessage(messages.routingNumberPlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
			</div>
		</template>

		<template v-else-if="withdrawContext.withdrawData.value.selectedMethod === 'crypto'">
			<Admonition type="warning" :header="formatMessage(messages.cryptoWarningHeader)">
				<span>{{ formatMessage(messages.cryptoWarningBody) }}</span>
			</Admonition>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.walletAddress) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input v-model="formData.walletAddress" type="text"
					:placeholder="formatMessage(messages.walletAddressPlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
			</div>
		</template>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.amount) }}
					<span class="text-brand-red">*</span>
				</span>
			</label>
			<div class="flex gap-2 items-center">
				<input v-model.number="formData.amount" type="number" step="0.01" min="0" :max="maxAmount"
					:placeholder="formatMessage(messages.amountPlaceholder)"
					class="bg-raised flex-1 rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary" />
				<ButtonStyled>
					<button @click="setMaxAmount" class="px-4 py-2">
						{{ formatMessage(messages.maxButton) }}
					</button>
				</ButtonStyled>
			</div>
			<span class="text-primary">
				{{ formatMoney(maxAmount) }} {{ formatMessage(messages.available) }}
			</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useWithdrawContext } from '@/providers/creator-withdraw.ts';
import { Admonition, ButtonStyled } from '@modrinth/ui';
import { formatMoney } from '@modrinth/utils';
import { defineMessages, useVIntl } from '@vintl/vintl';
import { computed, ref, watch } from 'vue';

const withdrawContext = useWithdrawContext();
const { formatMessage } = useVIntl();

interface BankAccountDetails {
	bankName: string;
	accountType: string;
	accountNumber: string;
	routingNumber: string;
}

interface CryptoWalletDetails {
	walletAddress: string;
}

interface FormData {
	bankName: string;
	accountType: string;
	accountNumber: string;
	routingNumber: string;
	walletAddress: string;
	amount: number;
}

const formData = ref<FormData>({
	bankName: '',
	accountType: 'CHECKING',
	accountNumber: '',
	routingNumber: '',
	walletAddress: '',
	amount: 0,
});

const maxAmount = computed(() => withdrawContext.maxWithdrawAmount.value);

const isLimited = computed(() => {
	const availableBalance = withdrawContext.balance.value?.available ?? 0;
	return maxAmount.value < availableBalance;
});

const accountTypeOptions = computed(() => [
	{ value: 'CHECKING', label: formatMessage(messages.checking) },
	{ value: 'SAVINGS', label: formatMessage(messages.savings) },
]);

const accountOwnerName = computed(() => {
	const kycData = withdrawContext.withdrawData.value.kycData;
	if (!kycData) return '';

	if (kycData.type === 'individual') {
		return `${kycData.firstName} ${kycData.lastName}`;
	} else if (kycData.type === 'business') {
		return kycData.name;
	}
	return '';
});

const accountOwnerAddress = computed(() => {
	const kycData = withdrawContext.withdrawData.value.kycData;
	if (!kycData || !kycData.physicalAddress) return '';

	const addr = kycData.physicalAddress;
	const parts = [
		addr.address1,
		addr.address2,
		addr.city,
		addr.state,
		addr.zip,
		addr.country,
	].filter(Boolean);

	return parts.join(', ');
});

function setMaxAmount() {
	formData.value.amount = maxAmount.value;
}

watch(
	[
		() => formData.value.bankName,
		() => formData.value.accountType,
		() => formData.value.accountNumber,
		() => formData.value.routingNumber,
		() => formData.value.walletAddress,
		() => formData.value.amount,
		() => withdrawContext.withdrawData.value.selectedMethod,
	],
	() => {
		const method = withdrawContext.withdrawData.value.selectedMethod;

		withdrawContext.withdrawData.value.amount = formData.value.amount;

		if (method === 'bank') {
			const bankDetails: BankAccountDetails = {
				bankName: formData.value.bankName,
				accountType: formData.value.accountType,
				accountNumber: formData.value.accountNumber,
				routingNumber: formData.value.routingNumber,
			};
			withdrawContext.withdrawData.value.accountDetails = {
				bankAccount: bankDetails,
			};
		} else if (method === 'crypto') {
			const cryptoDetails: CryptoWalletDetails = {
				walletAddress: formData.value.walletAddress,
			};
			withdrawContext.withdrawData.value.accountDetails = {
				cryptoWallet: cryptoDetails,
			};
		}
	},
	{ deep: true },
);

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
	accountType: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.account-type',
		defaultMessage: 'Account type',
	},
	accountTypePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.account-type-placeholder',
		defaultMessage: 'Select account type',
	},
	checking: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.checking',
		defaultMessage: 'Checking',
	},
	savings: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.savings',
		defaultMessage: 'Savings',
	},
	accountNumber: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.account-number',
		defaultMessage: 'Account number',
	},
	accountNumberPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.account-number-placeholder',
		defaultMessage: 'Enter account number',
	},
	routingNumber: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.routing-number',
		defaultMessage: 'Routing number',
	},
	routingNumberPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.routing-number-placeholder',
		defaultMessage: 'Enter 9-digit routing number',
	},
	walletAddress: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.wallet-address',
		defaultMessage: 'Wallet Address',
	},
	walletAddressPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.wallet-address-placeholder',
		defaultMessage: 'Enter your wallet address',
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
		defaultMessage: 'Check your wallet address!',
	},
	cryptoWarningBody: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.crypto-warning-body',
		defaultMessage: 'Please ensure your wallet address is correct. Modrinth cannot retrieve funds sent to an invalid or incorrect wallet. Double-check before proceeding.',
	},
	limitedByTaxRequirement: {
		id: 'dashboard.creator-withdraw-modal.muralpay-details.limited-by-tax',
		defaultMessage: 'Due to tax requirements, maximum withdrawal is {limit} without completing tax form.',
	},
});
</script>
