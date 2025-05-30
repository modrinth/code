<script setup lang="ts">
import { RadioButtonIcon, RadioButtonCheckedIcon, SpinnerIcon } from '@modrinth/assets'
import type Stripe from 'stripe'
import FormattedPaymentMethod from './FormattedPaymentMethod.vue'

const emit = defineEmits<{
  (e: 'select'): void
}>()

withDefaults(
  defineProps<{
    item: Stripe.PaymentMethod | undefined
    selected: boolean
    loading?: boolean
  }>(),
  {
    loading: false,
  },
)
</script>

<template>
  <button
    class="flex items-center w-full gap-2 border-none p-3 text-primary rounded-xl transition-all duration-200 hover:bg-button-bg hover:brightness-[--hover-brightness] active:scale-[0.98] hover:cursor-pointer"
    :class="selected ? 'bg-button-bg' : 'bg-transparent'"
    @click="emit('select')"
  >
    <RadioButtonCheckedIcon v-if="selected" class="size-6 text-brand" />
    <RadioButtonIcon v-else class="size-6 text-secondary" />

    <template v-if="item === undefined">
      <span>New payment method</span>
    </template>
    <FormattedPaymentMethod v-else-if="item" :method="item" />
    <SpinnerIcon v-if="loading" class="ml-auto size-4 text-secondary animate-spin" />
  </button>
</template>
