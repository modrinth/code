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
						<span class="text-orange">{{ formatMoney(usedLimit) }}</span> / {{ formatMoney(600) }}
					</div>
				</div>
				<div class="flex h-2 w-full overflow-hidden rounded-full bg-button-bg">
					<div class="bg-orange" :style="{ width: `${(usedLimit / 600) * 100}%` }"></div>
				</div>
				<template v-if="usedLimit !== 600">
					<span
						>You're nearing the {{ formatMoney(600) }} withdrawal threshold. You can withdraw up to
						<b>{{ formatMoney(usedLimit) }}</b> now, but you'll need to complete a tax form to
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
			</template>
			<template v-if="currentStage === 'withdraw-details'">
				<span class="font-semibold text-contrast">Withdraw from</span>
				<div class="flex flex-col rounded-xl bg-bg p-4">
					<span class="text-sm text-primary">Available balance</span>
					<span class="font-semibold text-contrast">{{ formatMoney(balance?.available) }}</span>
				</div>
				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Region</span><span class="text-brand-red">*</span>
					<UnknownIcon v-tooltip="`something here`" class="my-auto size-4 text-secondary" />
				</div>
				<TeleportDropdownMenu
					id="country-multiselect"
					v-model="countryProxy"
					class="country-multiselect"
					name="country"
					:options="countries"
					:display-name="(o: { id: string; name: string }) => o.name"
					placeholder="Select country..."
				/>
				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Withdraw to</span><span class="text-brand-red">*</span>
				</div>
				<TeleportDropdownMenu
					id="payment-method"
					v-model="paymentMethod"
					class="country-multiselect"
					name="payment-method"
					:options="paymentMethods"
					:display-name="(m: PayoutMethod) => m.name"
					placeholder="Select method..."
				/>
				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Amount</span><span class="text-brand-red">*</span>
				</div>
				<span class="text-secondary"
					>The minimum transfer amount is {{ formatMoney(minWithdrawAmount) }}.</span
				>
			</template>
		</div>
	</NewModal>
	<CreatorTaxFormModal ref="taxFormModal" @on-hide="reopenAfterTaxForm" />
</template>
<script lang="ts" setup>
import { FileTextIcon, UnknownIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal, TeleportDropdownMenu } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { all } from 'iso-3166-1'
import CreatorTaxFormModal from './CreatorTaxFormModal.vue'
type Stage = 'withdraw-limit' | 'withdraw-details' | 'confirmation'

// TODO: Deduplicate in @modrinth/api-client PR.
type FormCompletionStatus = 'unknown' | 'unrequested' | 'unsigned' | 'tin-mismatch' | 'complete'

interface UserBalanceResponse {
	available: number
	withdrawn_lifetime: number
	withdrawn_ytd: number
	pending: number
	// ISO 8601 date string -> amount
	dates: Record<string, number>
	// backend returns null when not applicable
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
const paymentMethod = ref<PayoutMethod | null>(null)

const props = defineProps<{
	balance: UserBalanceResponse | null
	payoutMethods?: PayoutMethod[]
	payoutMethodsPending?: boolean
	country: { id: string; name: string } | null
}>()

const emit = defineEmits<{
	(e: 'update:country', value: { id: string; name: string } | null): void
}>()

const countryProxy = computed({
	get: () => props.country,
	set: (v) => emit('update:country', v),
})

const paymentMethods = computed<PayoutMethod[]>(() => props.payoutMethods ?? [])

const maxWithdrawAmount = computed(() => {
	if (!paymentMethod.value) return props.balance?.available ?? 0

	const interval = paymentMethod.value.interval
	return 'standard' in interval
		? interval.standard.max
		: (interval?.fixed?.values.slice(-1)[0] ?? 0)
})

const minWithdrawAmount = computed(() => {
	if (!paymentMethod.value) return 0.25

	const interval = paymentMethod.value.interval
	return 'standard' in interval ? interval.standard.min : (interval?.fixed?.values?.[0] ?? 0.25)
})

const usedLimit = computed(() => {
	return 600 - (props.balance?.withdrawn_ytd ?? 0)
})

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
	() => paymentMethods.value,
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
