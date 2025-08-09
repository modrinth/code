<script setup lang="ts">
import { formatPrice } from '@modrinth/utils'
import { Dropdown } from 'floating-vue'
import { DropdownIcon } from '@modrinth/assets'
import ButtonStyled from '../base/ButtonStyled.vue'
import ServersSpecs from './ServersSpecs.vue'
import type { ServerBillingInterval, ServerPlan } from '../../utils/billing'

const props = defineProps<{
  title: string
  description: string
  plan: ServerPlan
  currency: string
  interval: ServerBillingInterval
  locale: string
  monthlyPrice?: number
}>()
</script>

<template>
  <div class="!bg-bg card !p-4">
    <div class="flex flex-col">
      <span class="text-2xl font-bold text-contrast">{{ title }}</span>
      <span class="m-0 text-lg font-bold text-contrast">
        {{ formatPrice(locale, monthlyPrice, currency, true) }}
        <span class="text-sm font-semibold text-secondary">
          / month<template v-if="interval !== 'monthly'">, billed {{ interval }}</template>
        </span>
      </span>
      <span class="text-sm mb-2">{{ description }}</span>

      <Dropdown
        placement="bottom-start"
        :triggers="['click', 'focus']"
        :popper-triggers="['hover']"
        :distance="8"
      >
        <template #default="{ shown }">
          <ButtonStyled size="small" circular type="outlined">
            <button class="!px-2">
              View plan details
              <DropdownIcon
                class="ml-auto size-5 transition-transform duration-300 shrink-0"
                :class="{ 'rotate-180': shown }"
              />
            </button>
          </ButtonStyled>
        </template>

        <template #popper>
          <div class="w-72 rounded-md border border-contrast/10 bg-bg p-3 shadow-lg">
            <ServersSpecs
              :ram="plan.metadata.ram!"
              :storage="plan.metadata.storage!"
              :cpus="plan.metadata.cpu!"
            />
          </div>
        </template>
      </Dropdown>
    </div>
  </div>
</template>
