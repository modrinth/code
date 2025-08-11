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
  <ModrinthServersPurchaseModal
    v-if="customer"
    :key="`manage-purchase-modal-${customer?.id}`"
    ref="purchaseModal"
    :publishable-key="config.public.stripePublishableKey"
    :initiate-payment="
      async (body) =>
        (await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })) as any
    "
    :available-products="pyroProducts"
    :on-error="handleError"
    :customer="customer"
    :payment-methods="paymentMethods"
    :currency="selectedCurrency"
    :return-url="`${config.public.siteUrl}/servers/manage`"
    :pings="regionPings"
    :regions="regions"
    :refresh-payment-methods="fetchPaymentData"
    :fetch-stock="fetchStock"
    :plan-stage="true"
  />
</template>

<script setup lang="ts">
import { ClockIcon, RocketIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import dayjs from "dayjs";
import dayjsDuration from "dayjs/plugin/duration";
import MedalPromoBackground from "~/assets/images/illustrations/medal_promo_background.svg?component";
import { ModrinthServersPurchaseModal } from "@modrinth/ui";
import { useServersFetch } from "~/composables/servers/servers-fetch";
import { useBaseFetch } from "#build/imports";
import { products } from "~/generated/state.json";

// eslint-disable-next-line import/no-named-as-default-member
dayjs.extend(dayjsDuration);

// (mirrors servers/index.vue)
const config = useRuntimeConfig();
const purchaseModal = ref<InstanceType<typeof ModrinthServersPurchaseModal> | null>(null);
const customer = ref<any>(null);
const paymentMethods = ref<any[]>([]);
const selectedCurrency = ref<string>("USD");
const regions = ref<any[]>([]);
const regionPings = ref<any[]>([]);

const pyroProducts = (products as any[])
  .filter((p) => p?.metadata?.type === "pyro")
  .sort((a, b) => (a?.metadata?.ram ?? 0) - (b?.metadata?.ram ?? 0));

function handleError(err: any) {
  // todo
  // eslint-disable-next-line no-console
  console.error("Purchase modal error:", err);
}

async function fetchPaymentData() {
  try {
    const [customerData, paymentMethodsData] = await Promise.all([
      useBaseFetch("billing/customer", { internal: true }),
      useBaseFetch("billing/payment_methods", { internal: true }),
    ]);
    customer.value = customerData as any;
    paymentMethods.value = paymentMethodsData as any[];
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Error fetching payment data:", error);
  }
}

function fetchStock(region: any, request: any) {
  return useServersFetch(`stock?region=${region.shortcode}`, {
    method: "POST",
    body: {
      ...request,
    },
    bypassAuth: true,
  }).then((res: any) => res.available as number);
}

function pingRegions() {
  useServersFetch("regions", {
    method: "GET",
    version: 1,
    bypassAuth: true,
  }).then((res: any) => {
    regions.value = res as any[];
    (regions.value as any[]).forEach((region: any) => {
      runPingTest(region);
    });
  });
}

const PING_COUNT = 20;
const PING_INTERVAL = 200;
const MAX_PING_TIME = 1000;

function runPingTest(region: any, index = 1) {
  if (index > 10) {
    regionPings.value.push({
      region: region.shortcode,
      ping: -1,
    });
    return;
  }

  const wsUrl = `wss://${region.shortcode}${index}.${region.zone}/pingtest`;
  try {
    const socket = new WebSocket(wsUrl);
    const pings: number[] = [];

    socket.onopen = () => {
      for (let i = 0; i < PING_COUNT; i++) {
        setTimeout(() => {
          socket.send(String(performance.now()));
        }, i * PING_INTERVAL);
      }
      setTimeout(
        () => {
          socket.close();
          const median = Math.round([...pings].sort((a, b) => a - b)[Math.floor(pings.length / 2)]);
          if (median) {
            regionPings.value.push({
              region: region.shortcode,
              ping: median,
            });
          }
        },
        PING_COUNT * PING_INTERVAL + MAX_PING_TIME,
      );
    };

    socket.onmessage = (event) => {
      const start = Number(event.data);
      pings.push(performance.now() - start);
    };

    socket.onerror = () => {
      runPingTest(region, index + 1);
    };
  } catch {
    // todo
  }
}

function openUpgradeModal() {
  purchaseModal.value?.show("quarterly");
}

onMounted(() => {
  fetchPaymentData();
  pingRegions();
});

const props = defineProps<{
  expiryDate?: string | Date;
}>();

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
