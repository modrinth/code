<script setup lang="ts">
import type Stripe from 'stripe'
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'
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
  <div
    v-show="selected === undefined"
    class="min-h-[16rem] flex flex-col gap-2 mt-2 p-4 bg-table-alternateRow rounded-xl justify-center items-center"
  >
    <div v-show="loadingElements">
      <ModalLoadingIndicator :error="loadingElementsFailed">
        Loading...
        <template #error> Error loading Stripe payment UI. </template>
      </ModalLoadingIndicator>
    </div>
    <div class="w-full">
      <div id="address-element"></div>
      <div id="payment-element" class="mt-4"></div>
    </div>
  </div>
</template>
