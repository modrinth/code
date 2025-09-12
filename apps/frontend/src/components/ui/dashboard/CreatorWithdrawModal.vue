<template>
	<NewModal
		ref="withdrawModal"
		:header="currentStageLabel"
		:noblur="true"
		:closable="currentStage !== 'confirmation'"
		:merge-header="!currentStageLabel"
		@onHide="currentStage = undefined"
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
						<template v-slot:icon="{ iconClass }">
							<FileTextIcon :class="iconClass" />
						</template>
						<template #actions>
							<ButtonStyled color="blue">
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
					<div class="ml-auto">
						<ButtonStyled color="blue">
							<button @click="showTaxFormModal"><FileTextIcon />Complete tax form</button>
						</ButtonStyled>
					</div>
				</template>
				<div class="mt-2 flex gap-3">
					<ButtonStyled color="standard">
						<button @click="withdrawModal?.hide()"><XIcon />Cancel</button>
					</ButtonStyled>
					<ButtonStyled color="standard">
						<button @click="currentStage = 'withdraw-details'" :disabled="remainingLimit <= 0">
							Continue with limit<ChevronRightIcon />
						</button>
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
					:listbox="true"
					:searchable="true"
					v-model="countryId"
					:options="countryOptions"
					placeholder="Select country..."
					:displayValue="countryProxy?.name ?? 'Select country...'"
				/>

				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Withdraw to</span><span class="text-brand-red">*</span>
				</div>
				<div class="relative">
					<Combobox
						:listbox="true"
						v-model="paymentMethodId"
						class="payment-method-select w-full"
						:options="paymentMethodOptions"
						placeholder="Select method..."
						:displayValue="
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
						<button @click="withdrawAmount = maxWithdrawAmount.toString()">Max</button>
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

			<template v-if="currentStage === 'confirmation'">
				<div class="relative flex min-h-[400px] flex-col items-center justify-center">
					<!-- Animated background particles -->
					<div class="absolute inset-0 overflow-hidden opacity-70">
						<div v-for="i in 20" :key="i" class="particle" :style="getParticleStyle(i)"></div>
					</div>

					<!-- Success content -->
					<div class="relative z-10 flex flex-col items-center gap-6">
						<div class="relative flex h-28 w-28 items-center justify-center">
							<div class="bg-green/10 absolute inset-0 rounded-full"></div>
							<div
								class="h-22 w-22 to-green-dark border-green-dark flex items-center justify-center rounded-full border bg-gradient-to-b from-green"
							>
								<CheckIcon class="h-14 w-14 text-white" />
							</div>
						</div>

						<div class="flex flex-col items-center gap-3 text-center">
							<h2 class="text-3xl font-semibold text-contrast">Withdraw Complete</h2>
							<p class="text-secondary">
								You have successfully withdrawn <b>{{ formatMoney(parsedWithdrawAmount) }}</b> from
								your Modrinth Balance to {{ formatPaymentMethodName(paymentMethod) }}.
							</p>
						</div>
					</div>
				</div>

				<div class="mt-6 flex gap-3">
					<ButtonStyled color="standard" class="flex-1">
						<button @click="goToPayouts">Payouts</button>
					</ButtonStyled>
					<ButtonStyled color="standard" class="flex-1">
						<button @click="goToTransactions">Transactions</button>
					</ButtonStyled>
				</div>
			</template>
		</div>
	</NewModal>
	<CreatorTaxFormModal ref="taxFormModal" @on-hide="reopenAfterTaxForm" />
</template>

<script lang="ts" setup>
import {
	CheckIcon,
	ChevronRightIcon,
	FileTextIcon,
	TransferIcon,
	UnknownIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	NewModal,
	injectNotificationManager,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { all } from 'iso-3166-1'
import type { CSSProperties } from 'vue'
import { getAuthUrl } from '~/composables/auth.js'
import CreatorTaxFormModal from './CreatorTaxFormModal.vue'

const { addNotification, handleError } = injectNotificationManager()

type Stage = 'withdraw-limit' | 'withdraw-details' | 'confirmation'

// TODO: Deduplicate in @modrinth/api-client PR.
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

// FancyDropdown-compatible country options and model binding
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
}>()

const countryProxy = computed({
	get: () => props.country,
	set: (v) => emit('update:country', v),
})

const paymentMethod = ref<PayoutMethod | null>(null)
const withdrawAmount = ref('')

// Linking state
const editingLink = ref(false)
const isVerifying = ref(false)
const paypalEmail = ref(props.userPayoutData?.paypal_address ?? '')
const venmoHandle = ref(props.userPayoutData?.venmo_handle ?? '')

const isPayPalSelected = computed(() => paymentMethod.value?.type === 'paypal')
const isVenmoSelected = computed(() => paymentMethod.value?.type === 'venmo')
const hasLinkedPayPal = computed(() => !!props.userPayoutData?.paypal_address)
const hasLinkedVenmo = computed(() => !!props.userPayoutData?.venmo_handle)

const availablePaymentMethods = computed<PayoutMethod[]>(() => {
	if (!props.payoutMethods) return []
	return props.payoutMethods.filter((m) => {
		// Filter based on country support
		if (countryProxy.value && !m.supported_countries.includes(countryProxy.value.id)) {
			return false
		}
		return true
	})
})

// FancyDropdown-compatible payment method options and model binding
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
	return Math.min(methodMax, props.balance?.available ?? 0, remainingLimit.value)
})

const minWithdrawAmount = computed(() => {
	if (!paymentMethod.value) return 0.25
	const interval = paymentMethod.value.interval
	return 'standard' in interval ? interval.standard.min : (interval?.fixed?.values?.[0] ?? 0.25)
})

const parsedWithdrawAmount = computed(() => {
	const regex = /^\$?(\d*(\.\d{2})?)$/gm
	const matches = regex.exec(withdrawAmount.value)
	return matches && matches[1] ? parseFloat(matches[1]) : 0.0
})

const usedLimit = computed(() => props.balance?.withdrawn_ytd ?? 0)
const remainingLimit = computed(() => Math.max(0, 600 - usedLimit.value))

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
	if (editingLink.value) return false
	if (paymentMethod.value.type === 'paypal') return !!props.userPayoutData?.paypal_address
	if (paymentMethod.value.type === 'venmo') return !!props.userPayoutData?.venmo_handle
	return false
})

const paymentAccountDisplay = computed(() => {
	if (!paymentMethod.value) return ''
	if (paymentMethod.value.type === 'paypal') return props.userPayoutData?.paypal_address ?? ''
	if (paymentMethod.value.type === 'venmo') return props.userPayoutData?.venmo_handle ?? ''
	return ''
})

function formatPaymentMethodName(method: PayoutMethod | null): string {
	if (!method) return ''
	return method.name
}

function getPaymentIcon(method: PayoutMethod) {
	// Return appropriate icon component based on payment method type
	// This would need to be implemented with actual icon imports
	return 'div'
}

function editPaymentMethod() {
	// Switch to inline editing mode
	editingLink.value = true
}

// Validate inputs
const paypalEmailValid = computed(() => {
	if (!isPayPalSelected.value) return false
	const v = paypalEmail.value?.trim()
	return !!v && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v)
})

const venmoHandleValid = computed(() => {
	if (!isVenmoSelected.value) return false
	const v = venmoHandle.value?.trim() ?? ''
	const name = v.startsWith('@') ? v.slice(1) : v
	return name.length >= 3 && name.length <= 30 && /^[A-Za-z0-9_]+$/.test(name)
})

function startEditingLink() {
	editingLink.value = true
}

function cancelEditingLink() {
	editingLink.value = false
	paypalEmail.value = props.userPayoutData?.paypal_address ?? ''
	venmoHandle.value = props.userPayoutData?.venmo_handle ?? ''
}

async function refreshAuthSilently() {
	try {
		await useAuth()
	} catch {
		// ignore
	}
}

async function verifyVenmo() {
	if (!venmoHandleValid.value) return
	isVerifying.value = true
	try {
		const userId = props.auth.user?.id
		if (!userId) throw new Error('Not signed in')
		await useBaseFetch(`user/${userId}`, {
			method: 'PATCH',
			body: {
				venmo_handle: (venmoHandle.value || null) as string | null,
			},
			apiVersion: 3,
		})
		await refreshAuthSilently()
		addNotification({
			title: 'Venmo linked',
			text: 'Your Venmo username has been saved.',
			type: 'success',
		})
		editingLink.value = false
	} catch (err: any) {
		handleError(err)
	} finally {
		isVerifying.value = false
	}
}

const paypalAuthUrl = getAuthUrl('paypal')

async function verifyPayPal() {
	if (!paypalEmailValid.value) return
	isVerifying.value = true
	try {
		const url = new URL(`${paypalAuthUrl}&token=${props.auth.token}`)
		const popup = window.open(url, 'paypal_oauth', 'width=520,height=700')
		if (!popup) throw new Error('Popup blocked')

		await new Promise<void>((resolve) => {
			const timer = setInterval(() => {
				if (popup.closed) {
					clearInterval(timer)
					resolve()
				}
			}, 500)
		})

		await refreshAuthSilently()
		addNotification({
			title: 'PayPal linked',
			text: 'Your PayPal account has been connected.',
			type: 'success',
		})
		editingLink.value = false
	} catch (err: any) {
		handleError(err)
	} finally {
		isVerifying.value = false
	}
}

async function initiateWithdraw() {
	if (!canProceedToWithdraw.value || !paymentMethod.value) return

	// Emit withdraw event to parent
	emit('withdraw', parsedWithdrawAmount.value, paymentMethod.value)

	// Show confirmation
	currentStage.value = 'confirmation'
}

function goToPayouts() {
	navigateTo('/dashboard/revenue/transfers')
}

function goToTransactions() {
	navigateTo('/dashboard/revenue')
}

function getParticleStyle(index: number): CSSProperties {
	const colors = [
		'#ff21fb',
		'#9ded1a',
		'#755ffd',
		'#f44620',
		'#470ba7',
		'#c9fcbe',
		'#fc9e1c',
		'#4bc8ef',
		'#ba3d02',
	]
	const size = Math.random() * 15 + 5
	const x = Math.random() * 100
	const y = Math.random() * 100
	const duration = Math.random() * 20 + 10
	const delay = Math.random() * 5

	return {
		position: 'absolute',
		width: `${size}px`,
		height: `${size}px`,
		left: `${x}%`,
		top: `${y}%`,
		backgroundColor: colors[index % colors.length],
		opacity: Math.random() * 0.6 + 0.2,
		borderRadius: Math.random() > 0.5 ? '50%' : '0',
		transform: `rotate(${Math.random() * 360}deg)`,
		animation: `float ${duration}s ${delay}s infinite ease-in-out`,
	} as CSSProperties
}

const taxFormModal = ref<InstanceType<typeof CreatorTaxFormModal> | null>(null)
function showTaxFormModal(e: MouseEvent) {
	withdrawModal.value?.hide()
	taxFormModal.value?.startTaxForm(e)
}

function reopenAfterTaxForm() {
	withdrawModal.value?.show()
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
		// Initialize linking mode depending on whether method is already linked
		editingLink.value = !showPaymentDetails.value
		// Reset inputs to current stored values
		paypalEmail.value = props.userPayoutData?.paypal_address ?? ''
		venmoHandle.value = props.userPayoutData?.venmo_handle ?? ''
	},
	{ immediate: true },
)

defineExpose({
	show,
})
</script>

<style scoped>
@keyframes float {
	0%,
	100% {
		transform: translateY(0) rotate(0deg);
	}
	50% {
		transform: translateY(-20px) rotate(180deg);
	}
}

.particle {
	pointer-events: none;
}
</style>
