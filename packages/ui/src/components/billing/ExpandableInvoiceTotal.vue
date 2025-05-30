<script setup lang="ts">
import Accordion from '../base/Accordion.vue'
import { formatPrice } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

const { locale } = useVIntl()

export type BillingItem = {
  title: string
  amount: number
}

const props = defineProps<{
  period?: string
  currency: string
  total: number
  billingItems: BillingItem[]
  loading?: boolean
}>()

const periodSuffix = computed(() => {
  return props.period ? ` / ${props.period}` : ''
})
</script>
<template>
  <Accordion
    class="rounded-2xl overflow-hidden bg-bg"
    button-class="bg-transparent p-0 w-full p-4 active:scale-[0.98] transition-transform duration-100"
  >
    <template #title>
      <div class="w-full flex items-center justify-between">
        <div class="flex items-center gap-2 text-contrast font-bold">Total</div>
        <div class="text-right mr-1">
          <span class="text-primary font-bold">
            <template v-if="loading">
              <SpinnerIcon class="animate-spin size-4" />
            </template>
            <template v-else> {{ formatPrice(locale, total, currency) }} </template
            ><span class="text-xs text-secondary">{{ periodSuffix }}</span>
          </span>
        </div>
      </div>
    </template>
    <div class="p-4 flex flex-col gap-4 bg-table-alternateRow">
      <div
        v-for="{ title, amount } in billingItems"
        :key="title"
        class="flex items-center justify-between"
      >
        <div class="font-semibold">
          {{ title }}
        </div>
        <div class="text-right">
          <template v-if="loading">
            <SpinnerIcon class="animate-spin size-4" />
          </template>
          <template v-else> {{ formatPrice(locale, amount, currency) }} </template
          ><span class="text-xs text-secondary">{{ periodSuffix }}</span>
        </div>
      </div>
    </div>
  </Accordion>
</template>
