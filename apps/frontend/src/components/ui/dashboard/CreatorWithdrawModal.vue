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
					<Admonition type="info" show-actions-underneath header="Tax form required">
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
		</div>
	</NewModal>
</template>
<script lang="ts" setup>
import { FileTextIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import CreatorTaxFormModal from './CreatorTaxFormModal.vue'

type Stage = 'withdraw-limit' | 'payment-method' | 'withdraw-amount' | 'confirmation'

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
	'payment-method': 'Payment Method',
	'withdraw-amount': 'Withdraw Amount',
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

	open('payment-method')
}

defineExpose({
	show,
})
</script>
