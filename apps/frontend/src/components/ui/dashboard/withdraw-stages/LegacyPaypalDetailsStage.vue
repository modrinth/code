<template>
	<div class="flex flex-col gap-4 sm:gap-5">
		<div v-if="isPayPal" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(messages.paypalAccount) }} <span class="text-red">*</span></span
				>
			</label>

			<div class="flex flex-col gap-2">
				<ButtonStyled v-if="!isPayPalAuthenticated" color="standard">
					<a :href="paypalAuthUrl" class="w-min" @click="handlePayPalAuth">
						<PayPalColorIcon class="size-5" />
						{{ formatMessage(messages.signInWithPaypal) }}
					</a>
				</ButtonStyled>
				<ButtonStyled v-else>
					<button class="w-min" @click="handleDisconnectPaypal">
						<XIcon /> {{ formatMessage(messages.disconnectButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="isPayPal && isPayPalAuthenticated" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">{{
					formatMessage(messages.account)
				}}</span>
			</label>

			<div class="flex flex-col gap-2 rounded-2xl bg-surface-2 px-4 py-2.5">
				<span>{{ paypalEmail }}</span>
			</div>
		</div>

		<div v-if="isVenmo" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(messages.venmoHandle) }} <span class="text-red">*</span></span
				>
			</label>
			<div class="flex flex-row gap-2">
				<input
					v-model="venmoHandle"
					type="text"
					:placeholder="formatMessage(messages.venmoHandlePlaceholder)"
					class="w-full rounded-[14px] bg-surface-4 px-4 py-3 text-contrast placeholder:text-secondary sm:py-2.5"
				/>
				<ButtonStyled color="brand">
					<button
						v-tooltip="!hasVenmoChanged ? 'Change the venmo username to save.' : undefined"
						:disabled="venmoSaving || !hasVenmoChanged"
						@click="saveVenmoHandle"
					>
						<CheckIcon v-if="venmoSaveSuccess" />
						<SaveIcon v-else />
						{{
							venmoSaveSuccess
								? formatMessage(messages.savedButton)
								: formatMessage(messages.saveButton)
						}}
					</button>
				</ButtonStyled>
			</div>
			<span v-if="venmoSaveError" class="text-sm font-bold text-red">
				{{ venmoSaveError }}
			</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(formFieldLabels.amount) }} <span class="text-red">*</span></span
				>
			</label>

			<RevenueInputField
				v-model="formData.amount"
				:max-amount="effectiveMaxAmount"
				:min-amount="selectedMethodDetails?.interval?.standard?.min || 0.01"
			/>

			<WithdrawFeeBreakdown
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
			/>

			<Checkbox v-model="agreedTerms">
				<span>
					<IntlFormatted :message-id="financialMessages.rewardsProgramTermsAgreement">
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/cmp" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</span>
			</Checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, PayPalColorIcon, SaveIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	financialMessages,
	formFieldLabels,
	normalizeChildren,
} from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { useDebounceFn } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'

import RevenueInputField from '@/components/ui/dashboard/RevenueInputField.vue'
import WithdrawFeeBreakdown from '@/components/ui/dashboard/WithdrawFeeBreakdown.vue'
import { getAuthUrl, removeAuthProvider, useAuth } from '@/composables/auth.js'
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const { withdrawData, maxWithdrawAmount, availableMethods, calculateFees, saveStateToStorage } =
	useWithdrawContext()
const { formatMessage } = useVIntl()
const auth = await useAuth()

const isPayPal = computed(() => withdrawData.value.selection.provider === 'paypal')
const isVenmo = computed(() => withdrawData.value.selection.provider === 'venmo')

const selectedMethodDetails = computed(() => {
	const methodId = withdrawData.value.selection.methodId
	if (!methodId) return null
	return availableMethods.value.find((m) => m.id === methodId) || null
})

const isPayPalAuthenticated = computed(() => {
	return (auth.value.user as any)?.auth_providers?.includes('paypal') || false
})

const paypalEmail = computed(() => {
	return (auth.value.user as any)?.payout_data?.paypal_address || ''
})

const paypalAuthUrl = computed(() => {
	const route = useRoute()
	const authToken = useCookie('auth-token')
	const separator = route.fullPath.includes('?') ? '&' : '?'
	const returnUrl = `${route.fullPath}${separator}paypal_auth_return=true`
	return `${getAuthUrl('paypal', returnUrl)}&auth_token=${authToken.value}`
})

function handlePayPalAuth() {
	saveStateToStorage()
}

async function handleDisconnectPaypal() {
	try {
		await removeAuthProvider('paypal')
	} catch (error) {
		console.error('Failed to disconnect PayPal:', error)
	}
}

const venmoHandle = ref<string>((auth.value.user as any)?.venmo_handle || '')
const initialVenmoHandle = ref<string>((auth.value.user as any)?.venmo_handle || '')
const venmoSaving = ref(false)
const venmoSaveSuccess = ref(false)
const venmoSaveError = ref<string | null>(null)

const hasVenmoChanged = computed(() => {
	return venmoHandle.value.trim() !== initialVenmoHandle.value.trim()
})

async function saveVenmoHandle() {
	if (!venmoHandle.value.trim()) {
		venmoSaveError.value = 'Please enter a Venmo handle'
		return
	}

	venmoSaving.value = true
	venmoSaveError.value = null
	venmoSaveSuccess.value = false

	try {
		await useBaseFetch(`user/${(auth.value.user as any)?.id}`, {
			method: 'PATCH',
			body: {
				venmo_handle: venmoHandle.value.trim(),
			},
		})

		// @ts-expect-error auth.js is not typed
		await useAuth(auth.value.token)

		initialVenmoHandle.value = venmoHandle.value.trim()
		venmoSaveSuccess.value = true

		setTimeout(() => {
			venmoSaveSuccess.value = false
		}, 3000)
	} catch (error) {
		console.error('Failed to update Venmo handle:', error)
		venmoSaveError.value = 'Failed to save Venmo handle. Please try again.'
	} finally {
		venmoSaving.value = false
	}
}

const maxAmount = computed(() => maxWithdrawAmount.value)
const roundedMaxAmount = computed(() => Math.floor(maxAmount.value * 100) / 100)

const effectiveMaxAmount = computed(() => {
	const apiMax = selectedMethodDetails.value?.interval?.standard?.max
	if (apiMax) {
		return Math.min(roundedMaxAmount.value, apiMax)
	}
	return roundedMaxAmount.value
})

const formData = ref<Record<string, any>>({
	amount: withdrawData.value.calculation.amount || undefined,
})

const agreedTerms = computed({
	get: () => withdrawData.value.agreedTerms,
	set: (value) => {
		withdrawData.value.agreedTerms = value
	},
})

const isComponentValid = computed(() => {
	const hasAmount = (formData.value.amount || 0) > 0
	const hasAgreed = agreedTerms.value

	if (!hasAmount || !hasAgreed) return false

	if (isPayPal.value) {
		return isPayPalAuthenticated.value
	} else if (isVenmo.value) {
		return venmoHandle.value.trim().length > 0
	}

	return false
})

const calculatedFee = ref<number>(0)
const feeLoading = ref(false)

const calculateFeesDebounced = useDebounceFn(async () => {
	const amount = formData.value.amount
	if (!amount || amount <= 0) {
		calculatedFee.value = 0
		return
	}

	const methodId = withdrawData.value.selection.methodId
	if (!methodId) {
		calculatedFee.value = 0
		return
	}

	feeLoading.value = true
	try {
		await calculateFees()
		calculatedFee.value = withdrawData.value.calculation.fee ?? 0
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(
	() => formData.value.amount,
	() => {
		withdrawData.value.calculation.amount = formData.value.amount ?? 0

		if (formData.value.amount) {
			feeLoading.value = true
			calculateFeesDebounced()
		} else {
			calculatedFee.value = 0
			feeLoading.value = false
		}
	},
	{ deep: true },
)

watch(
	[isComponentValid, venmoHandle, () => formData.value.amount, agreedTerms, isPayPalAuthenticated],
	() => {
		withdrawData.value.stageValidation.paypalDetails = isComponentValid.value
	},
	{ immediate: true },
)

onMounted(async () => {
	if (formData.value.amount) {
		feeLoading.value = true
		calculateFeesDebounced()
	}
})

const messages = defineMessages({
	paymentMethod: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.payment-method',
		defaultMessage: 'Payment method',
	},
	paypalAccount: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.paypal-account',
		defaultMessage: 'PayPal account',
	},
	account: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.account',
		defaultMessage: 'Account',
	},
	signInWithPaypal: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.sign-in-with-paypal',
		defaultMessage: 'Sign in with PayPal',
	},
	paypalAuthDescription: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.paypal-auth-description',
		defaultMessage: 'Connect your PayPal account to receive payments directly.',
	},
	venmoHandle: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.venmo-handle',
		defaultMessage: 'Venmo handle',
	},
	venmoHandlePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.venmo-handle-placeholder',
		defaultMessage: '@username',
	},
	venmoDescription: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.venmo-description',
		defaultMessage: 'Enter your Venmo handle to receive payments.',
	},
	disconnectButton: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.disconnect-account',
		defaultMessage: 'Disconnect account',
	},
	saveButton: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.save-button',
		defaultMessage: 'Save',
	},
	savedButton: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.saved-button',
		defaultMessage: 'Saved',
	},
	saveSuccess: {
		id: 'dashboard.creator-withdraw-modal.paypal-details.save-success',
		defaultMessage: 'Venmo handle saved successfully!',
	},
})
</script>
