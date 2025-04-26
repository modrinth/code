<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { RightArrowIcon, SparklesIcon, UnknownIcon } from "@modrinth/assets";
import type { MessageDescriptor } from "@vintl/vintl";

const { formatMessage } = useVIntl();

const emit = defineEmits<{
  (e: "select" | "scroll-to-faq"): void;
}>();

type Plan = "small" | "medium" | "large";

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
      id: "servers.plan.small.description",
      defaultMessage:
        "Perfect for vanilla multiplayer, small friend groups, SMPs, and light modding.",
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
      id: "servers.plan.medium.description",
      defaultMessage: "Great for modded multiplayer and small communities.",
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
      id: "servers.plan.large.description",
      defaultMessage: "Ideal for larger communities, modpacks, and heavy modding.",
    }),
  },
};

const props = defineProps<{
  capacity?: number;
  plan: Plan;
  ram: number;
  storage: number;
  cpus: number;
  price: number;
}>();

const outOfStock = computed(() => {
  return !props.capacity || props.capacity === 0;
});

const lowStock = computed(() => {
  return !props.capacity || props.capacity < 8;
});

const formattedRam = computed(() => {
  return props.ram / 1024;
});

const formattedStorage = computed(() => {
  return props.storage / 1024;
});

const sharedCpus = computed(() => {
  return props.cpus / 2;
});
</script>

<template>
  <li class="relative flex w-full flex-col justify-between pt-12 lg:w-1/3">
    <div
      v-if="lowStock"
      class="absolute left-0 right-0 top-[-2px] rounded-t-2xl p-4 text-center font-bold"
      :class="outOfStock ? 'bg-bg-red' : 'bg-bg-orange'"
    >
      <template v-if="outOfStock"> Out of stock! </template>
      <template v-else> Only {{ capacity }} left in stock! </template>
    </div>
    <div
      :style="
        plan === 'medium'
          ? {
              background: `radial-gradient(
                  86.12% 101.64% at 95.97% 94.07%,
                  rgba(27, 217, 106, 0.23) 0%,
                  rgba(14, 115, 56, 0.2) 100%
                )`,
              border: `1px solid rgba(12, 107, 52, 0.55)`,
              'box-shadow': `0px 12px 38.1px rgba(27, 217, 106, 0.13)`,
            }
          : undefined
      "
      class="flex w-full flex-col justify-between gap-4 rounded-2xl bg-bg p-8 text-left"
      :class="{ '!rounded-t-none': lowStock }"
    >
      <div class="flex flex-col gap-4">
        <div class="flex flex-row items-center justify-between">
          <h1 class="m-0">{{ formatMessage(plans[plan].name) }}</h1>
          <div
            class="grid size-8 place-content-center rounded-full text-xs font-bold"
            :class="`${plans[plan].accentBg} ${plans[plan].accentText}`"
          >
            {{ formatMessage(plans[plan].symbol) }}
          </div>
        </div>
        <p class="m-0">{{ formatMessage(plans[plan].description) }}</p>
        <div
          class="flex flex-row flex-wrap items-center gap-2 text-nowrap text-secondary xl:justify-between"
        >
          <p class="m-0">{{ formattedRam }} GB RAM</p>
          <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
          <p class="m-0">{{ formattedStorage }} GB SSD</p>
          <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
          <p class="m-0">{{ sharedCpus }} Shared CPUs</p>
        </div>
        <div class="flex items-center gap-2 text-secondary">
          <SparklesIcon /> Bursts up to {{ cpus }} CPUs
          <nuxt-link
            v-tooltip="
              `CPU bursting allows your server to temporarily use additional threads to help mitigate TPS spikes. Click for more info.`
            "
            to="/servers#cpu-burst"
            @click="() => emit('scroll-to-faq')"
          >
            <UnknownIcon class="h-4 w-4 text-secondary opacity-80" />
          </nuxt-link>
        </div>
        <span class="m-0 text-2xl font-bold text-contrast">
          ${{ price / 100 }}<span class="text-lg font-semibold text-secondary">/month</span>
        </span>
      </div>
      <ButtonStyled
        :color="plans[plan].buttonColor"
        :type="plan === 'medium' ? 'standard' : 'highlight-colored-text'"
        size="large"
      >
        <span v-if="outOfStock" class="button-like disabled"> Out of Stock </span>
        <button v-else @click="() => emit('select')">
          Get Started
          <RightArrowIcon class="shrink-0" />
        </button>
      </ButtonStyled>
    </div>
  </li>
</template>

<style scoped lang="scss"></style>
