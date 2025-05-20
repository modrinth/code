<template>
  <div class="flex justify-center mb-2">
    <div class="flex rounded-full bg-bg p-1">
      <button 
        class="px-4 py-2 rounded-full transition-all duration-200 font-semibold"
        :class="modelValue === 'monthly' 
          ? 'bg-highlight-green text-brand'
          : 'bg-bg text-secondary'"
        @click="$emit('update:modelValue', 'monthly')"
      >
        Pay Monthly
      </button>
      <button 
        class="px-4 py-2 rounded-full transition-all duration-200 flex items-center gap-2 font-semibold"
        :class="modelValue === 'quarterly' 
          ? 'bg-highlight-green text-brand'
          : 'bg-bg text-secondary'"
        @click="$emit('update:modelValue', 'quarterly')"
      >
        Pay Quarterly 
        <span 
          v-if="quarterlyDiscountPercentage > 0"
          class="text-xs px-2 py-0.5 rounded-full font-bold"
          :class="modelValue === 'quarterly' 
            ? 'bg-brand text-brand-inverted'
            : 'bg-accent text-secondary'"
        >
          (Save {{ (quarterlyDiscountPercentage * 100).toFixed(0) }}%)
        </span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
// Toggle component for switching between monthly and quarterly payment options
// Displays a discount badge with percentage when quarterly payment is available
defineProps<{
  // Currently selected payment cycle - supports two-way binding with v-model
  modelValue: 'monthly' | 'quarterly'
  // Percentage discount applied for quarterly payments (e.g. 0.16 for 16%)
  quarterlyDiscountPercentage: number
}>();

defineEmits<{
  (e: 'update:modelValue', value: 'monthly' | 'quarterly'): void
}>();
</script> 