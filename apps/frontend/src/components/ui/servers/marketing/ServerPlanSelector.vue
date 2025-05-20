<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { RightArrowIcon, SparklesIcon, UnknownIcon } from "@modrinth/assets";
import type { MessageDescriptor } from "@vintl/vintl";
import type { PropType } from "vue";

const { formatMessage } = useVIntl();

const emit = defineEmits<{
  (e: "select" | "scroll-to-faq"): void;
}>();

type Plan = "small" | "medium" | "large";

// Define plan configurations including colors, text styles, and localized messages
const plans: Record<
  Plan,
  {
    buttonColor: "blue" | "green" | "purple";
    accentText: string;
    accentBg: string;
    name: MessageDescriptor;
    symbol: MessageDescriptor;
    description: MessageDescriptor;
  }
> = {
  small: {
    buttonColor: "blue",
    accentText: "text-blue",
    accentBg: "bg-bg-blue",
    name: defineMessage({
      id: "servers.plan.small.name",
      defaultMessage: "Small",
    }),
    symbol: defineMessage({
      id: "servers.plan.small.symbol",
      defaultMessage: "S",
    }),
    description: defineMessage({
      id: "servers.plan.small.description.new",
      defaultMessage: "Perfect for 1-5 friends with a few light mods.",
    }),
  },
  medium: {
    buttonColor: "green",
    accentText: "text-green",
    accentBg: "bg-bg-green",
    name: defineMessage({
      id: "servers.plan.medium.name",
      defaultMessage: "Medium",
    }),
    symbol: defineMessage({
      id: "servers.plan.medium.symbol",
      defaultMessage: "M",
    }),
    description: defineMessage({
      id: "servers.plan.medium.description.new",
      defaultMessage: "Great for 6-15 players and multiple mods.",
    }),
  },
  large: {
    buttonColor: "purple",
    accentText: "text-purple",
    accentBg: "bg-bg-purple",
    name: defineMessage({
      id: "servers.plan.large.name",
      defaultMessage: "Large",
    }),
    symbol: defineMessage({
      id: "servers.plan.large.symbol",
      defaultMessage: "L",
    }),
    description: defineMessage({
      id: "servers.plan.large.description.new",
      defaultMessage: "Ideal for 15-25 players, modpacks, or heavy modding.",
    }),
  },
};

// Component props with validation for plan sizes and payment cycle options
const props = defineProps({
  capacity: Number,
  plan: {
    type: String as PropType<Plan>,
    required: true,
  },
  ram: {
    type: Number,
    required: true,
  },
  storage: {
    type: Number,
    required: true,
  },
  cpus: {
    type: Number,
    required: true,
  },
  price: {
    type: Number,
    required: true,
  },
  paymentCycle: {
    type: String as PropType<'monthly' | 'quarterly'>,
    default: 'monthly',
    validator: (value: string) => ['monthly', 'quarterly'].includes(value),
  },
});

// Reactive computed properties for stock status display
const outOfStock = computed(() => {
  return !props.capacity || props.capacity === 0;
});

const lowStock = computed(() => {
  return !props.capacity || props.capacity < 8;
});

// Format resources for display
const formattedRam = computed(() => {
  return props.ram / 1024;
});

const formattedStorage = computed(() => {
  return props.storage / 1024;
});

// Calculate shared CPUs (half of total CPU count)
const sharedCpus = computed(() => {
  return props.cpus / 2;
});
</script>

<template>
  <li class="relative flex w-full flex-col justify-between pt-12 lg:w-1/3">
    <div
      v-if="lowStock"
      class="absolute left-0 right-0 top-0 rounded-t-2xl p-3 text-center font-bold text-sm"
      :class="outOfStock ? 'bg-bg-red text-red' : 'bg-bg-orange text-orange'"
    >
      <template v-if="outOfStock"> Out of stock </template>
      <template v-else> Only {{ capacity }} left in stock </template>
    </div>
    <div
      :style="plan === 'medium' ? undefined : undefined"
      class="flex h-full w-full flex-col justify-between gap-6 rounded-2xl bg-bg p-8 text-left min-h-[400px]"
      :class="{ 
        '!rounded-t-none': lowStock, 
        'medium-plan-highlight': plan === 'medium', 
        'pt-10': lowStock 
      }"
    >
      <div class="flex flex-col gap-3">
        <div class="flex flex-row items-center justify-between">
          <h1 class="m-0 text-3xl font-bold">{{ formatMessage(plans[plan].name) }}</h1>
          <div
            v-if="plan === 'medium'" 
            class="text-xs px-2 py-1 rounded-full bg-highlight-green text-brand font-semibold"
          >
            Most Popular
          </div>
        </div>
        <span class="m-0 text-2xl font-bold text-contrast">
          ${{ price > 0 ? (price / 100).toFixed(2) : '0.00' }}<span class="text-lg font-semibold text-secondary">
            /month
          </span>
        </span>
        <p class="m-0 text-base text-secondary leading-normal max-w-xs">{{ formatMessage(plans[plan].description) }}</p>
      </div>

      <ButtonStyled
        :color="plans[plan].buttonColor"
        :type="plan === 'medium' ? 'standard' : 'highlight-colored-text'"
        size="large"
        class="w-full mt-auto mb-4" 
      >
        <span v-if="outOfStock" class="button-like disabled w-full"> Out of Stock </span>
        <button v-else class="w-full" @click="() => emit('select')">
          Select Plan
        </button>
      </ButtonStyled>

      <ul class="m-0 flex flex-col gap-2 p-0 text-sm text-secondary">
        <li class="flex items-center gap-2">
          <SparklesIcon class="size-4 text-brand" /> {{ formattedRam }} GB RAM
        </li>
        <li class="flex items-center gap-2">
          <SparklesIcon class="size-4 text-brand" /> {{ formattedStorage }} GB SSD
        </li>
        <li class="flex items-center gap-2">
          <SparklesIcon class="size-4 text-brand" /> {{ sharedCpus }} Shared CPUs
        </li>
        <li class="flex items-center gap-2">
          <SparklesIcon class="size-4 text-brand" /> Bursts up to {{ cpus }} CPUs
          <nuxt-link
            v-tooltip="
              `CPU bursting allows your server to temporarily use additional threads to help mitigate TPS spikes. Click for more info.`
            "
            to="/servers#cpu-burst"
            class="flex items-center"
            @click="() => emit('scroll-to-faq')"
          >
            <UnknownIcon class="size-4 text-secondary opacity-60 hover:opacity-100 transition-opacity" />
          </nuxt-link>
        </li>
      </ul>
    </div>
  </li>
</template>

<style scoped lang="scss">
.button-like.disabled {
  display: flex;
  justify-content: center;
  align-items: center;
}

.medium-plan-highlight {
  background: radial-gradient(
    86.12% 101.64% at 95.97% 94.07%,
    rgba(27, 217, 106, 0.23) 0%,
    rgba(14, 115, 56, 0.2) 100%
  );
  border: 1px solid rgba(12, 107, 52, 0.55);
  box-shadow: 0px 12px 38.1px rgba(27, 217, 106, 0.13);
}
</style>
