<template>
	<NewModal
		ref="modal"
		header="Verify Payout"
		width="46rem"
		max-width="calc(100vw - 2rem)"
		:close-on-click-outside="!submitting"
		:on-hide="reset"
	>
		<div class="flex flex-col gap-8">
			<div
				class="grid grid-cols-[auto_minmax(0,1fr)] gap-5 rounded-xl border border-solid border-orange bg-orange-highlight p-6 text-orange"
			>
				<TriangleAlertIcon class="mt-1 size-7" aria-hidden="true" />
				<div class="flex flex-col gap-3">
					<span class="text-xl font-semibold">You are about to initiate a payout</span>
					<span class="text-xl leading-8">
						This will distribute {{ formatCurrency(creatorAmount, { cents: true }) }} to creators
						for {{ formatMonthYear(payout.payouts_date) }}.
					</span>
				</div>
			</div>

			<label class="flex flex-col gap-4">
				<span class="text-xl font-medium text-secondary">
					Enter <span class="font-semibold text-contrast">{{
						formatCurrency(creatorAmount, { cents: true })
					}}</span> to confirm
				</span>
				<StyledInput
					v-model="confirmedAmount"
					type="number"
					inputmode="decimal"
					:step="0.01"
					placeholder="0.00"
					wrapper-class="w-full"
					input-class="!h-16 !text-2xl"
				/>
			</label>

			<label class="flex flex-col gap-6">
				<span class="text-xl font-medium text-secondary">
					Enter 6-digit code from your authenticator app
				</span>
				<div class="flex flex-wrap gap-4">
					<input
						v-for="index in 6"
						:key="index"
						:ref="(element) => setCodeInput(element, index - 1)"
						:value="totpDigits[index - 1]"
						inputmode="numeric"
						maxlength="1"
						class="h-20 w-20 rounded-xl border border-solid border-surface-4 bg-surface-1.5 text-center text-2xl font-semibold text-contrast outline-none transition-shadow focus:ring-4 focus:ring-brand-shadow"
						@input="handleTotpInput($event, index - 1)"
						@keydown="handleTotpKeydown($event, index - 1)"
						@paste.prevent="handleTotpPaste"
					/>
				</div>
			</label>

			<ButtonStyled color="green" size="large">
				<button class="w-full" :disabled="!canSubmit || submitting" @click="submit">
					Verify & Run Payout
					<ChevronRightIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronRightIcon, TriangleAlertIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal, StyledInput } from '@modrinth/ui'
import { computed, nextTick, ref } from 'vue'

import {
	formatCurrency,
	formatMonthYear,
	roundCurrency,
	type DistributionAdjustment,
} from '../utils'

const props = defineProps<{
	payout: Labrinth.Payouts.Internal.HistoryItem
	creatorAmount: number
	amountReceived: number
	adjustments: DistributionAdjustment[]
	submitting?: boolean
}>()

const emit = defineEmits<{
	submit: [request: Labrinth.Payouts.Internal.StartDistributionRequest]
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const confirmedAmount = ref<number | undefined>()
const totpDigits = ref<string[]>(['', '', '', '', '', ''])
const codeInputs = ref<HTMLInputElement[]>([])

const totpCode = computed(() => totpDigits.value.join(''))
const canSubmit = computed(
	() =>
		roundCurrency(Number(confirmedAmount.value ?? Number.NaN)) ===
			roundCurrency(props.creatorAmount) && /^\d{6}$/.test(totpCode.value),
)

function show() {
	modal.value?.show()
	void nextTick(() => codeInputs.value[0]?.focus())
}

function hide() {
	modal.value?.hide()
}

function reset() {
	confirmedAmount.value = undefined
	totpDigits.value = ['', '', '', '', '', '']
	codeInputs.value = []
}

function setCodeInput(element: Element | null, index: number) {
	if (element instanceof HTMLInputElement) {
		codeInputs.value[index] = element
	}
}

function handleTotpInput(event: Event, index: number) {
	const input = event.target as HTMLInputElement
	const digit = input.value.replace(/\D/g, '').slice(-1)
	totpDigits.value[index] = digit
	input.value = digit

	if (digit && index < codeInputs.value.length - 1) {
		codeInputs.value[index + 1]?.focus()
	}
}

function handleTotpKeydown(event: KeyboardEvent, index: number) {
	if (event.key === 'Backspace' && !totpDigits.value[index] && index > 0) {
		codeInputs.value[index - 1]?.focus()
	}
}

function handleTotpPaste(event: ClipboardEvent) {
	const clipboardText = event.clipboardData?.getData('text') ?? ''
	const pastedCode = clipboardText.replace(/\D/g, '').slice(0, 6)
	if (!pastedCode) {
		return
	}

	totpDigits.value = Array.from({ length: 6 }, (_, index) => pastedCode[index] ?? '')
	codeInputs.value[Math.min(pastedCode.length, 5)]?.focus()
}

function submit() {
	if (!canSubmit.value || props.submitting) {
		return
	}

	emit('submit', {
		payouts_date: props.payout.payouts_date,
		totp_code: totpCode.value,
		amount_received: props.amountReceived,
		adjustments: props.adjustments,
	})
}

defineExpose({
	show,
	hide,
})
</script>
