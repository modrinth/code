<template>
	<div class="flex flex-col gap-2">
		<div class="flex items-center gap-2">
			<div class="relative flex-1">
				<input
					ref="amountInput"
					:value="modelValue"
					type="number"
					step="0.01"
					:min="minAmount"
					:max="maxAmount"
					:placeholder="formatMessage(formFieldPlaceholders.amountPlaceholder)"
					class="w-full rounded-[14px] bg-surface-4 py-2.5 pl-4 pr-4 text-contrast placeholder:text-secondary"
					@input="handleInput"
				/>
			</div>
			<ButtonStyled>
				<button class="px-4 py-2" @click="setMaxAmount">
					{{ formatMessage(commonMessages.maxButton) }}
				</button>
			</ButtonStyled>
		</div>
		<span class="my-1 mt-2 text-secondary">
			{{ formatMessage(financialMessages.available, { amount: formatMoney(maxAmount) }) }}
		</span>
	</div>
</template>

<script setup lang="ts">
import {
	ButtonStyled,
	commonMessages,
	financialMessages,
	formFieldPlaceholders,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { nextTick, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		modelValue: number | undefined
		maxAmount: number
		minAmount?: number
	}>(),
	{
		minAmount: 0.01,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: number | undefined]
}>()

const { formatMessage } = useVIntl()
const amountInput = ref<HTMLInputElement | null>(null)

async function setMaxAmount() {
	const maxValue = props.maxAmount
	emit('update:modelValue', maxValue)
	// Force display of 2 decimal places after Vue updates the DOM
	await nextTick()
	if (amountInput.value) {
		amountInput.value.value = maxValue.toFixed(2)
	}
}

function handleInput(event: Event) {
	const input = event.target as HTMLInputElement
	const value = input.value

	if (value && value.includes('.')) {
		const parts = value.split('.')
		if (parts[1] && parts[1].length > 2) {
			const rounded = Math.floor(parseFloat(value) * 100) / 100
			emit('update:modelValue', rounded)
			input.value = rounded.toString()
			return
		}
	}

	const numValue = value === '' ? undefined : parseFloat(value)
	emit('update:modelValue', numValue)
}

watch(
	() => props.modelValue,
	async (newAmount) => {
		if (newAmount !== undefined && newAmount !== null) {
			if (newAmount > props.maxAmount) {
				emit('update:modelValue', props.maxAmount)
				await nextTick()
				if (amountInput.value) {
					amountInput.value.value = props.maxAmount.toFixed(2)
				}
			} else if (newAmount < 0) {
				emit('update:modelValue', 0)
			}
		}
	},
)
</script>
