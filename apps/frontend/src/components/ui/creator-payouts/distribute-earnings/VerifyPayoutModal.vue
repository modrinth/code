<template>
	<NewModal
		ref="modal"
		header="Verify Payout"
		width="32rem"
		max-width="calc(100vw - 2rem)"
		:close-on-click-outside="!submitting"
		:on-hide="reset"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" header="You are about to initiate a payout">
				This will distribute {{ formatCurrency(creatorAmount, { cents: true }) }} to creators for
				{{ formatMonthYear(payout.payouts_date) }}.
			</Admonition>

			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					Enter
					{{ creatorAmount }}
					to confirm
				</span>
				<StyledInput
					v-model="confirmedAmount"
					type="number"
					inputmode="decimal"
					:step="0.01"
					placeholder="0.00"
					wrapper-class="w-full"
				/>
			</label>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					Enter 6-digit code from your authenticator app
				</span>
				<label
					class="flex w-fit flex-wrap gap-1.5"
					@pointerdown.prevent="focusFirstUnfilledCodeInput"
				>
					<input
						v-for="index in 6"
						:key="index"
						:ref="(element) => setCodeInput(element, index - 1)"
						:value="totpDigits[index - 1]"
						inputmode="numeric"
						maxlength="1"
						class="h-12 w-11 appearance-none rounded-xl border-none bg-surface-4 p-1 text-center text-base font-medium text-primary outline-none focus:text-primary focus:ring-4 focus:ring-brand-shadow"
						@input="handleTotpInput($event, index - 1)"
						@keydown="handleTotpKeydown($event, index - 1)"
						@paste.prevent="handleTotpPaste"
					/>
				</label>
			</div>

			<ButtonStyled color="green">
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
import { ChevronRightIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal, StyledInput } from '@modrinth/ui'
import { type ComponentPublicInstance,computed, nextTick, ref } from 'vue'

import {
	type DistributionAdjustment,
	formatCurrency,
	formatMonthYear,
	roundCurrency,
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
	void nextTick(focusFirstUnfilledCodeInput)
}

function hide() {
	modal.value?.hide()
}

function reset() {
	confirmedAmount.value = undefined
	totpDigits.value = ['', '', '', '', '', '']
	codeInputs.value = []
}

function setCodeInput(element: Element | ComponentPublicInstance | null, index: number) {
	if (element instanceof HTMLInputElement) {
		codeInputs.value[index] = element
	}
}

function focusFirstUnfilledCodeInput() {
	const firstUnfilledIndex = totpDigits.value.findIndex((digit) => !digit)
	const inputIndex = firstUnfilledIndex === -1 ? totpDigits.value.length - 1 : firstUnfilledIndex
	const input = codeInputs.value[inputIndex]
	input?.focus()
	input?.setSelectionRange(input.value.length, input.value.length)
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
	void nextTick(focusFirstUnfilledCodeInput)
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
