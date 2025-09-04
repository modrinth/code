<template>
	<CreatorTaxFormModal ref="taxFormModal" :on-hide="reopenAfterTaxForm" />
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
				<Multiselect
					id="country-multiselect"
					v-model="country"
					class="country-multiselect"
					placeholder="Select country..."
					track-by="id"
					label="name"
					:options="countries"
					:searchable="true"
					:close-on-select="true"
					:show-labels="false"
					:allow-empty="false"
					:use-teleport="true"
					open-direction="below"
				/>
				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Withdraw to</span><span class="text-brand-red">*</span>
				</div>
				<Multiselect
					id="payment-method"
					v-model="paymentMethod"
					class="country-multiselect"
					placeholder="Select method..."
					track-by="id"
					label="name"
					:options="paymentMethods"
					:searchable="true"
					:close-on-select="true"
					:show-labels="false"
					:allow-empty="false"
					:use-teleport="true"
					open-direction="below"
				/>
				<div class="flex gap-1 align-middle text-lg font-semibold text-contrast">
					<span>Amount</span><span class="text-brand-red">*</span>
				</div>
			</template>
		</div>
	</NewModal>
</template>
<script lang="ts" setup>
import { FileTextIcon, UnknownIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { all } from 'iso-3166-1'
import { Multiselect } from 'vue-multiselect'
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

interface PayoutMethod {
	id: string
	name: string
	type: string
}

const countries = computed(() =>
	all().map((x) => ({
		id: x.alpha2,
		name: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)

const country = ref<{ id: string; name: string } | null>(null)
const paymentMethod = ref<PayoutMethod | null>(null)

const {
	data: payoutMethods,
	refresh: refreshPayoutMethods,
	pending: payoutMethodsPending,
} = await useAsyncData(
	'payout-methods',
	() => useBaseFetch(`payout/methods?country=${country.value?.id ?? 'US'}`, { apiVersion: 3 }),
	{ default: () => [] as PayoutMethod[], watch: [country] },
)

const paymentMethods = computed<PayoutMethod[]>(() => (payoutMethods.value as PayoutMethod[]) ?? [])

const props = defineProps<{
	balance: UserBalanceResponse | null
}>()

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

// Initialize default country (fallback to US if available)
if (!country.value) {
	const us = countries.value.find((c) => c.id === 'US')
	country.value = us ?? countries.value[0] ?? null
}

// Keep selected method in sync with fetched options
watch(
	() => paymentMethods.value,
	(list) => {
		if (!list?.length) {
			paymentMethod.value = null
			return
		}
		// Reset selection if current selection is not in the new list
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
