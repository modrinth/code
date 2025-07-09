<script setup lang="ts">
import type Stripe from 'stripe'
import { useVIntl, defineMessages } from '@vintl/vintl'
import PaymentMethodOption from './PaymentMethodOption.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  (e: 'select', paymentMethod: Stripe.PaymentMethod | undefined): void
}>()

defineProps<{
  paymentMethods: Stripe.PaymentMethod[]
  selected?: Stripe.PaymentMethod
  loadingElements: boolean
  loadingElementsFailed: boolean
}>()

const messages = defineMessages({
  prompt: {
    id: 'servers.purchase.step.payment.prompt',
    defaultMessage: 'Select a payment method',
  },
  description: {
    id: 'servers.purchase.step.payment.description',
    defaultMessage: `You won't be charged yet.`,
  },
})
</script>

<template>
  <h2 class="mt-0 mb-1 text-xl font-bold text-contrast">
    {{ formatMessage(messages.prompt) }}
  </h2>
  <p class="mt-0 mb-4 text-secondary">
    {{ formatMessage(messages.description) }}
  </p>
  <div class="flex flex-col gap-1">
    <PaymentMethodOption
      v-for="method in paymentMethods"
      :key="method.id"
      :item="method"
      :selected="selected?.id === method.id"
      @select="emit('select', method)"
    />
    <PaymentMethodOption
      :loading="false"
      :item="undefined"
      :selected="selected === undefined"
      @select="emit('select', undefined)"
    />
  </div>
</template>
