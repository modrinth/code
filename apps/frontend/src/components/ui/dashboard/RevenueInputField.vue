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
					:max="safeMaxAmount"
					:disabled="isDisabled"
					:placeholder="formatMessage(formFieldPlaceholders.amountPlaceholder)"
					class="w-full rounded-[14px] bg-surface-4 py-2.5 pl-4 pr-4 text-contrast placeholder:text-secondary"
					@input="handleInput"
				/>
			</div>
			<Combobox
				v-if="showCurrencySelector"
				:model-value="selectedCurrency"
				:options="currencyOptions"
				class="w-min"
				@update:model-value="$emit('update:selectedCurrency', $event)"
			>
				<template v-for="option in currencyOptions" :key="option.value" #[`option-${option.value}`]>
					<span class="font-semibold leading-tight">{{ option.label }}</span>
				</template>
			</Combobox>
			<ButtonStyled>
				<button class="px-4 py-2" :disabled="isDisabled" @click="setMaxAmount">
					{{ formatMessage(commonMessages.maxButton) }}
				</button>
			</ButtonStyled>
		</div>
		<div>
			<span class="my-1 mt-0 text-secondary">{{ formatMoney(safeMaxAmount) }} available.</span>
			<Transition name="fade">
				<span v-if="isBelowMinimum" class="text-red">
					Amount must be at least {{ formatMoney(minAmount) }}.
				</span>
			</Transition>
			<Transition name="fade">
				<span v-if="isAboveMaximum" class="text-red">
					Amount cannot exceed {{ formatMoney(safeMaxAmount) }}.
				</span>
			</Transition>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled, Combobox, commonMessages, formFieldPlaceholders } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { computed, nextTick, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		modelValue: number | undefined
		maxAmount: number
		minAmount?: number
		showCurrencySelector?: boolean
		selectedCurrency?: string
		currencyOptions?: Array<{ value: string; label: string }>
	}>(),
	{
		minAmount: 0.01,
		showCurrencySelector: false,
		currencyOptions: () => [],
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: number | undefined]
	'update:selectedCurrency': [value: string]
}>()

const { formatMessage } = useVIntl()
const amountInput = ref<HTMLInputElement | null>(null)

const safeMaxAmount = computed(() => {
	return Math.max(0, props.maxAmount)
})

const isDisabled = computed(() => {
	return safeMaxAmount.value < 0.01
})

const isBelowMinimum = computed(() => {
	return (
		props.modelValue !== undefined && props.modelValue > 0 && props.modelValue < props.minAmount
	)
})

const isAboveMaximum = computed(() => {
	return props.modelValue !== undefined && props.modelValue > safeMaxAmount.value
})

async function setMaxAmount() {
	const maxValue = safeMaxAmount.value
	emit('update:modelValue', maxValue)

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
			if (newAmount > safeMaxAmount.value) {
				emit('update:modelValue', safeMaxAmount.value)
				await nextTick()
				if (amountInput.value) {
					amountInput.value.value = safeMaxAmount.value.toFixed(2)
				}
			} else if (newAmount < 0) {
				emit('update:modelValue', 0)
			}
		}
	},
)
</script>

<style scoped>
.fade-enter-active {
	transition: opacity 200ms ease-out;
}

.fade-leave-active {
	transition: opacity 150ms ease-in;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
