<template>
	<NewModal
		ref="withdrawModal"
		:header="currentStageLabel"
		:noblur="true"
		:closable="currentStage !== 'confirmation'"
		:merge-header="!currentStageLabel"
		@on-hide="currentStage = undefined"
	>
		<div class="flex w-full flex-col gap-4 sm:w-[540px]">
			<template v-if="currentStage === 'withdraw-limit'">
				<div class="flex w-full flex-row justify-between">
					<span class="font-semibold text-contrast">Withdraw Remaining</span>
					<div>
						<span class="text-orange">{{ formatMoney(remainingLimit) }}</span> /
						{{ formatMoney(600) }}
					</div>
				</div>
				<div class="flex h-2 w-full overflow-hidden rounded-full bg-button-bg">
					<div class="bg-orange" :style="{ width: `${(usedLimit / 600) * 100}%` }"></div>
				</div>
				<template v-if="remainingLimit > 0">
					<span
						>You're nearing the {{ formatMoney(600) }} withdrawal threshold. You can withdraw up to
						<b>{{ formatMoney(remainingLimit) }}</b> now, but you'll need to complete a tax form to
						withdraw more.</span
					>
					<Admonition type="warning" show-actions-underneath header="Tax form required">
						To withdraw your full <b>{{ formatMoney(balance?.available) }}</b> available balance
						please complete the form below. It is required for tax reporting and only needs to be
						done once.
						<template #icon="{ iconClass }">
							<FileTextIcon :class="iconClass" />
						</template>
						<template #actions>
							<ButtonStyled color="orange">
								<button @click="showTaxFormModal">Complete tax form</button>
							</ButtonStyled>
						</template>
					</Admonition>
				</template>
				<template v-else>
					<span>
						You've used up your <b>{{ formatMoney(600) }}</b> withdrawal limit. You must complete a
						tax form to withdraw more.
					</span>
				</template>
				<div class="ml-auto flex gap-3">
					<ButtonStyled color="standard">
						<button @click="withdrawModal?.hide()"><XIcon />Cancel</button>
					</ButtonStyled>
					<ButtonStyled v-if="remainingLimit > 0" color="standard">
						<button @click="currentStage = 'withdraw-details'">
							Continue with limit <ChevronRightIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled v-else color="orange">
						<button @click="showTaxFormModal">Complete tax form <ChevronRightIcon /></button>
					</ButtonStyled>
				</div>
			</template>

			<template v-if="currentStage === 'withdraw-details'">
				<span class="font-semibold text-contrast">Withdraw from</span>
				<div class="flex flex-col rounded-xl bg-bg p-4">
					<span class="text-sm text-primary">Available balance</span>
					<span class="font-semibold text-contrast">{{ formatMoney(balance?.available) }}</span>
				</div>

				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Region</span><span class="text-brand-red">*</span>
					<UnknownIcon
						v-tooltip="`Select your country or region`"
						class="my-auto size-4 text-secondary"
					/>
				</div>
				<Combobox
					v-model="countryId"
					:listbox="true"
					:searchable="true"
					:options="countryOptions"
					placeholder="Select country..."
					:display-value="countryProxy?.name ?? 'Select country...'"
				/>

				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Withdraw to</span><span class="text-brand-red">*</span>
				</div>
				<div class="relative">
					<Combobox
						v-model="paymentMethodId"
						:listbox="true"
						class="payment-method-select w-full"
						:options="paymentMethodOptions"
						placeholder="Select method..."
						:display-value="
							paymentMethod ? formatPaymentMethodName(paymentMethod) : 'Select method...'
						"
					/>
				</div>

				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Amount</span><span class="text-brand-red">*</span>
				</div>
				<span class="text-secondary"
					>The minimum transfer amount is {{ formatMoney(minWithdrawAmount) }}.</span
				>
				<div class="flex items-center gap-0">
					<input
						v-model="withdrawAmount"
						type="text"
						pattern="^\d*(\.\d{0,2})?$"
						placeholder="Enter amount..."
						class="flex-1 rounded-l-xl border-y border-l border-divider bg-button-bg px-4 py-2.5 placeholder-secondary focus:outline-none"
					/>
					<ButtonStyled color="standard" class="rounded-l-none">
						<button @click="withdrawAmount = maxWithdrawAmount.toFixed(2)">Max</button>
					</ButtonStyled>
				</div>

				<div v-if="withdrawErrors.length > 0" class="space-y-2">
					<span v-for="error in withdrawErrors" :key="error" class="block text-red">
						{{ error }}
					</span>
				</div>

				<div class="mt-2 flex gap-3">
					<ButtonStyled color="standard">
						<button @click="withdrawModal?.hide()"><XIcon />Cancel</button>
					</ButtonStyled>
					<ButtonStyled color="green" :disabled="!canProceedToWithdraw">
						<button @click="initiateWithdraw"><TransferIcon />Withdraw</button>
					</ButtonStyled>
				</div>
			</template>

			<template v-if="currentStage === 'confirmation'"> </template>
		</div>
	</NewModal>
	<CreatorTaxFormModal
		ref="taxFormModal"
		@success="onTaxFormSuccess"
		@cancelled="onTaxFormCancelled"
	/>
</template>

<script lang="ts" setup>
import { ChevronRightIcon, FileTextIcon, TransferIcon, UnknownIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { all } from 'iso-3166-1'
import { nextTick } from 'vue'

import CreatorTaxFormModal from './CreatorTaxFormModal.vue'

const { addNotification } = injectNotificationManager()

type Stage = 'withdraw-limit' | 'withdraw-details' | 'confirmation'

type FormCompletionStatus = 'unknown' | 'unrequested' | 'unsigned' | 'tin-mismatch' | 'complete'

interface UserBalanceResponse {
	available: number
	withdrawn_lifetime: number
	withdrawn_ytd: number
	pending: number
	dates: Record<string, number>
	requested_form_type: string | null
	form_completion_status: FormCompletionStatus | null
}

type PayoutMethodType = 'venmo' | 'paypal' | 'tremendous' | 'unknown'

type PayoutInterval = { fixed: { values: number[] } } | { standard: { min: number; max: number } }

interface PayoutMethodFee {
	percentage: number
	min: number
	max?: number | null
}

interface PayoutMethod {
	id: string
	name: string
	type: PayoutMethodType
	supported_countries: string[]
	image_url?: string | null
	interval: PayoutInterval
	fee: PayoutMethodFee
}

const countries = computed(() =>
	all().map((x) => ({
		id: x.alpha2,
		name: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)

const countryOptions = computed(() => countries.value.map((c) => ({ value: c.id, label: c.name })))

const countryId = computed<string | null>({
	get: () => countryProxy.value?.id ?? null,
	set: (v) => {
		const selected = countries.value.find((c) => c.id === v) ?? null
		emit('update:country', selected)
	},
})

const props = defineProps<{
	auth: any
	balance: UserBalanceResponse | null
	payoutMethods?: PayoutMethod[]
	payoutMethodsPending?: boolean
	country: { id: string; name: string } | null
	userPayoutData?: {
		paypal_address?: string
		venmo_handle?: string
	}
}>()

const emit = defineEmits<{
	(e: 'update:country', value: { id: string; name: string } | null): void
	(e: 'withdraw', amount: number, method: PayoutMethod): void
	(e: 'refresh-data'): void
}>()

const countryProxy = computed({
	get: () => props.country,
	set: (v) => emit('update:country', v),
})

const paymentMethod = ref<PayoutMethod | null>(null)
const withdrawAmount = ref('')

const availablePaymentMethods = computed<PayoutMethod[]>(() => {
	if (!props.payoutMethods) return []
	return props.payoutMethods.filter((m) => {
		if (countryProxy.value && !m.supported_countries.includes(countryProxy.value.id)) {
			return false
		}
		return true
	})
})

const paymentMethodOptions = computed(() =>
	availablePaymentMethods.value.map((m) => ({ value: m.id, label: formatPaymentMethodName(m) })),
)

const paymentMethodId = computed<string | null>({
	get: () => paymentMethod.value?.id ?? null,
	set: (id) => {
		paymentMethod.value = availablePaymentMethods.value.find((m) => m.id === id) ?? null
	},
})

const maxWithdrawAmount = computed(() => {
	if (!paymentMethod.value) return props.balance?.available ?? 0
	const interval = paymentMethod.value.interval
	const methodMax =
		'standard' in interval ? interval.standard.max : (interval?.fixed?.values.slice(-1)[0] ?? 0)
	const max = Math.min(methodMax, props.balance?.available ?? 0, remainingLimit.value)
	return Math.floor(max * 100) / 100
})

const minWithdrawAmount = computed(() => {
	if (!paymentMethod.value) return 0.25
	const interval = paymentMethod.value.interval
	return 'standard' in interval ? interval.standard.min : (interval?.fixed?.values?.[0] ?? 0.25)
})

const parsedWithdrawAmount = computed(() => {
	const s = (withdrawAmount.value ?? '').trim()
	const match = s.match(/^\$?(\d*(?:\.\d{1,2})?)$/)
	return match && match[1] ? parseFloat(match[1]) : 0.0
})

const usedLimit = computed(() => props.balance?.withdrawn_ytd ?? 0)
const remainingLimit = computed(() => {
	const raw = 600 - usedLimit.value
	if (raw <= 0) return 0
	const cents = Math.floor(raw * 100)
	return cents / 100
})

const withdrawErrors = computed(() => {
	const errors: string[] = []

	if (!parsedWithdrawAmount.value && withdrawAmount.value.length > 0) {
		errors.push(`${withdrawAmount.value} is not a valid amount`)
	} else if (parsedWithdrawAmount.value > maxWithdrawAmount.value) {
		errors.push(`The amount must be no more than ${formatMoney(maxWithdrawAmount.value)}`)
	} else if (
		parsedWithdrawAmount.value < minWithdrawAmount.value &&
		parsedWithdrawAmount.value > 0
	) {
		errors.push(`The amount must be at least ${formatMoney(minWithdrawAmount.value)}`)
	}

	if (paymentMethod.value?.type === 'paypal' && !props.userPayoutData?.paypal_address) {
		errors.push('Please link your PayPal account to proceed.')
	}

	if (paymentMethod.value?.type === 'venmo' && !props.userPayoutData?.venmo_handle) {
		errors.push('Please set your Venmo handle to proceed.')
	}

	return errors
})

const canProceedToWithdraw = computed(() => {
	return (
		withdrawErrors.value.length === 0 &&
		parsedWithdrawAmount.value > 0 &&
		paymentMethod.value !== null &&
		countryProxy.value !== null
	)
})

const showPaymentDetails = computed(() => {
	if (!paymentMethod.value) return false
	if (paymentMethod.value.type === 'paypal') return !!props.userPayoutData?.paypal_address
	if (paymentMethod.value.type === 'venmo') return !!props.userPayoutData?.venmo_handle
	return false
})

function formatPaymentMethodName(method: PayoutMethod | null): string {
	if (!method) return ''
	return method.name
}

async function initiateWithdraw() {
	if (!canProceedToWithdraw.value || !paymentMethod.value) return

	emit('withdraw', parsedWithdrawAmount.value, paymentMethod.value)

	currentStage.value = 'confirmation'
}

const taxFormModal = ref<InstanceType<typeof CreatorTaxFormModal> | null>(null)
function showTaxFormModal(e: MouseEvent) {
	withdrawModal.value?.hide()
	taxFormModal.value?.startTaxForm(e)
}

function onTaxFormSuccess() {
	emit('refresh-data')
	nextTick(() => {
		show('withdraw-details')
	})
}

function onTaxFormCancelled() {
	show('withdraw-limit')
}

const stageLabels = readonly<Record<Stage, string | undefined>>({
	'withdraw-limit': 'Withdraw Limit',
	'withdraw-details': 'Withdraw Details',
	confirmation: undefined,
})

const currentStageLabel = computed<string | undefined>(() => {
	if (!currentStage.value) return undefined
	return stageLabels[currentStage.value]
})

const withdrawModal = ref<InstanceType<typeof NewModal> | null>(null)
const currentStage = ref<Stage | undefined>()

function open(stage: Stage) {
	currentStage.value = stage
	withdrawModal.value?.show()
}

function show(preferred?: Stage) {
	if (preferred) {
		open(preferred)
		return
	}

	const b = props.balance
	if (!b || b.available <= 0) {
		open('withdraw-limit')
		return
	}

	const needsCompliance =
		b.form_completion_status !== null && b.form_completion_status !== 'complete'
	if (needsCompliance) {
		open('withdraw-limit')
		return
	}

	open('withdraw-details')
}

watch(
	() => availablePaymentMethods.value,
	(list) => {
		if (!list?.length) {
			paymentMethod.value = null
			return
		}
		if (!paymentMethod.value || !list.some((m) => m.id === paymentMethod.value?.id)) {
			paymentMethod.value = list[0]
		}
	},
	{ immediate: true },
)

defineExpose({
	show,
})
</script>
