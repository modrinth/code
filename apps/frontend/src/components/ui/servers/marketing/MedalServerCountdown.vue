<template>
  <div
    class="medal-promotion flex w-full flex-row items-center justify-between rounded-2xl p-4 shadow-xl"
  >
    <div class="overlay"></div>
    <MedalPromoBackground class="background-pattern scale-[125%]" />

    <div class="z-10 flex flex-col gap-1">
      <div class="flex items-center gap-2 text-lg font-semibold text-contrast">
        <ClockIcon class="clock-glow text-medal-orange size-5" />
        <span>
          Your <span class="text-medal-orange">Medal</span> powered Modrinth Server will expire in
          <span class="text-medal-orange font-bold">{{ timeLeftCountdown.days }}</span> days
          <span class="text-medal-orange font-bold">{{ timeLeftCountdown.hours }}</span> hours
          <span class="text-medal-orange font-bold">{{ timeLeftCountdown.minutes }}</span> minutes
          <span class="text-medal-orange font-bold">{{ timeLeftCountdown.seconds }}</span> seconds.
        </span>
      </div>
    </div>

    <ButtonStyled color="orange" type="outlined" size="large">
      <button class="z-10 my-auto" @click="openUpgradeModal"><RocketIcon /> Upgrade</button>
    </ButtonStyled>
  </div>
  <ServersUpgradeModalWrapper ref="upgradeModal" />
</template>

<script setup lang="ts">
import { ClockIcon, RocketIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import dayjs from "dayjs";
import dayjsDuration from "dayjs/plugin/duration";
import MedalPromoBackground from "~/assets/images/illustrations/medal_promo_background.svg?component";
import type { ComponentPublicInstance } from "vue";
import ServersUpgradeModalWrapper from "../ServersUpgradeModalWrapper.vue";

// eslint-disable-next-line import/no-named-as-default-member
dayjs.extend(dayjsDuration);

type UpgradeWrapperRef = ComponentPublicInstance<{ open: (id?: string) => void | Promise<void> }>;
const upgradeModal = ref<UpgradeWrapperRef | null>(null);

const props = defineProps<{
  expiryDate?: string | Date;
  serverId?: string;
}>();

function openUpgradeModal() {
  upgradeModal.value?.open(props.serverId);
}

const expiryDate = computed(() => {
  if (props.expiryDate) {
    return dayjs(props.expiryDate);
  }
  return dayjs().add(5, "day");
});

const timeLeftCountdown = ref({ days: 0, hours: 0, minutes: 0, seconds: 0 });

function updateCountdown() {
  const now = dayjs();
  const diff = expiryDate.value.diff(now);

  if (diff <= 0) {
    timeLeftCountdown.value = { days: 0, hours: 0, minutes: 0, seconds: 0 };
    return;
  }

  const duration = dayjs.duration(diff);
  timeLeftCountdown.value = {
    days: duration.days(),
    hours: duration.hours(),
    minutes: duration.minutes(),
    seconds: duration.seconds(),
  };
}

updateCountdown();

const intervalId = ref<NodeJS.Timeout | null>(null);
onMounted(() => {
  intervalId.value = setInterval(updateCountdown, 1000);
});

onUnmounted(() => {
  if (intervalId.value) clearInterval(intervalId.value);
});
</script>

<style scoped lang="scss">
.medal-promotion {
  position: relative;
  border: 1px solid var(--medal-promotion-bg-orange);
  background: inherit; // allows overlay + pattern to take over
  overflow: hidden;
}

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: var(--medal-promotion-bg-gradient);
  z-index: 1;
  border-radius: inherit;
}

.background-pattern {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  background-color: var(--medal-promotion-bg);
  border-radius: inherit;
  color: var(--medal-promotion-text-orange);
}

.clock-glow {
  filter: drop-shadow(0 0 72px var(--color-orange)) drop-shadow(0 0 36px var(--color-orange))
    drop-shadow(0 0 18px var(--color-orange));
}

.text-medal-orange {
  color: var(--medal-promotion-text-orange);
  font-weight: bold;
}
</style>
