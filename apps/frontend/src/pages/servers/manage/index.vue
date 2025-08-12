<template>
  <div
    data-pyro-server-list-root
    class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
  >
    <ModrinthServersPurchaseModal
      v-if="customer"
      ref="purchaseModal"
      :publishable-key="config.public.stripePublishableKey"
      :initiate-payment="async (body) => await initiatePayment(body)"
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
      :existing-plan="currentPlanFromSubscription"
      :existing-subscription="subscription || undefined"
      @hide="() => (subscription = null)"
    />
    <div
      v-if="hasError || fetchError"
      class="mx-auto flex h-full min-h-[calc(100vh-4rem)] flex-col items-center justify-center gap-4 text-left"
    >
      <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
        <div class="flex flex-col items-center text-center">
          <div class="flex flex-col items-center gap-4">
            <div class="grid place-content-center rounded-full bg-bg-blue p-4">
              <HammerIcon class="size-12 text-blue" />
            </div>
            <h1 class="m-0 w-fit text-3xl font-bold">Servers could not be loaded</h1>
          </div>
          <p class="text-lg text-secondary">We may have temporary issues with our servers.</p>
          <ul class="m-0 list-disc space-y-4 p-0 pl-4 text-left text-sm leading-[170%]">
            <li>
              Our systems automatically alert our team when there's an issue. We are already working
              on getting them back online.
            </li>
            <li>
              If you recently purchased your Modrinth Server, it is currently in a queue and will
              appear here as soon as it's ready. <br />
              <span class="font-medium text-contrast"
                >Do not attempt to purchase a new server.</span
              >
            </li>
            <li>
              If you require personalized support regarding the status of your server, please
              contact Modrinth Support.
            </li>

            <li v-if="fetchError" class="text-red">
              <p>Error details:</p>
              <CopyCode
                :text="(fetchError as ModrinthServersFetchError).message || 'Unknown error'"
                :copyable="false"
                :selectable="false"
                :language="'json'"
              />
            </li>
          </ul>
        </div>
        <ButtonStyled size="large" type="standard" color="brand">
          <a class="mt-6 !w-full" href="https://support.modrinth.com">Contact Modrinth Support</a>
        </ButtonStyled>
        <ButtonStyled size="large" @click="() => reloadNuxtApp()">
          <button class="mt-3 !w-full">Reload</button>
        </ButtonStyled>
      </div>
    </div>

    <LazyUiServersServerManageEmptyState
      v-else-if="serverList.length === 0 && !isPollingForNewServers && !hasError"
    />

    <template v-else>
      <div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
        <h1 class="w-full text-4xl font-bold text-contrast">Servers</h1>
        <div class="mb-4 flex w-full flex-row items-center justify-end gap-2 md:mb-0 md:gap-4">
          <div class="relative w-full text-sm md:w-72">
            <label class="sr-only" for="search">Search</label>
            <SearchIcon
              class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
              aria-hidden="true"
            />
            <input
              id="search"
              v-model="searchInput"
              class="w-full border-[1px] border-solid border-button-border pl-9"
              type="search"
              name="search"
              autocomplete="off"
              placeholder="Search servers..."
            />
          </div>
          <ButtonStyled type="standard">
            <NuxtLink
              class="!h-10 whitespace-pre !border-[1px] !border-solid !border-button-border text-sm !font-medium"
              :to="{ path: '/servers', hash: '#plan' }"
            >
              <PlusIcon class="size-4" />
              New server
            </NuxtLink>
          </ButtonStyled>
        </div>
      </div>

      <ul
        v-if="filteredData.length > 0 || isPollingForNewServers"
        class="m-0 flex flex-col gap-4 p-0"
      >
        <UiServersServerListing
          v-for="server in filteredData.filter((s) => !s.is_medal)"
          :key="server.server_id"
          v-bind="server"
        />
        <MedalServerListing
          v-for="server in filteredData.filter((s) => s.status !== 'suspended')"
          :key="server.server_id"
          v-bind="server"
          @upgrade="openUpgradeModal(server.server_id)"
        />
        <LazyUiServersServerListingSkeleton v-if="isPollingForNewServers" />
      </ul>
      <div v-else class="flex h-full items-center justify-center">
        <p class="text-contrast">No servers found.</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import Fuse from "fuse.js";
import { HammerIcon, PlusIcon, SearchIcon } from "@modrinth/assets";
import { ButtonStyled, CopyCode, ModrinthServersPurchaseModal } from "@modrinth/ui";
import type { Server, ModrinthServersFetchError, UserSubscription } from "@modrinth/utils";
import { reloadNuxtApp } from "#app";
import { useServersFetch } from "~/composables/servers/servers-fetch.ts";
import MedalServerListing from "~/components/ui/servers/marketing/MedalServerListing.vue";
import { products } from "~/generated/state.json";
import type { ServerPlan } from "@modrinth/ui/src/utils/billing";

definePageMeta({
  middleware: "auth",
});

useHead({
  title: "Servers - Modrinth",
});

interface ServerResponse {
  servers: Server[];
}

type LocalServer = Server & { is_preview?: boolean; is_medal?: boolean };

const router = useRouter();
const route = useRoute();
const hasError = ref(false);
const isPollingForNewServers = ref(false);

const {
  data: serverResponse,
  error: fetchError,
  refresh,
} = await useAsyncData<ServerResponse>("ServerList", () =>
  useServersFetch<ServerResponse>("servers"),
);

watch([fetchError, serverResponse], ([error, response]) => {
  hasError.value = !!error || !response;
});

const serverList = computed<LocalServer[]>(() => {
  if (!serverResponse.value) return [];
  return serverResponse.value.servers;
});

const searchInput = ref("");

const fuse = computed(() => {
  if (serverList.value.length === 0) return null;
  return new Fuse(serverList.value, {
    keys: ["name", "loader", "mc_version", "game", "state"],
    includeScore: true,
    threshold: 0.4,
  });
});

function introToTop(array: Server[]): Server[] {
  return array.slice().sort((a, b) => {
    return Number(b.flows?.intro) - Number(a.flows?.intro);
  });
}

const filteredData = computed<LocalServer[]>(() => {
  if (!searchInput.value.trim()) {
    return introToTop(serverList.value);
  }
  return fuse.value
    ? introToTop(fuse.value.search(searchInput.value).map((result) => result.item))
    : [];
});

const previousServerList = ref<Server[]>([]);
const refreshCount = ref(0);

const checkForNewServers = async () => {
  await refresh();
  refreshCount.value += 1;
  if (JSON.stringify(previousServerList.value) !== JSON.stringify(serverList.value)) {
    isPollingForNewServers.value = false;
    clearInterval(intervalId);
    router.replace({ query: {} });
  } else if (refreshCount.value >= 5) {
    isPollingForNewServers.value = false;
    clearInterval(intervalId);
  }
};

let intervalId: ReturnType<typeof setInterval> | undefined;

onMounted(() => {
  if (route.query.redirect_status === "succeeded") {
    isPollingForNewServers.value = true;
    previousServerList.value = [...serverList.value];
    intervalId = setInterval(checkForNewServers, 5000);
  }
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});

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

const subscription = ref<UserSubscription | null>(null);
const currentPlanFromSubscription = computed<ServerPlan | undefined>(() => {
  console.log("Current plan from subscription:", {
    subscription: subscription.value,
    pyroProducts,
  });
  return subscription.value
    ? (pyroProducts.find(
        (p) =>
          p.prices.filter((price: { id: string }) => price.id === subscription.value?.price_id)
            .length > 0,
      ) ?? undefined)
    : undefined;
});

async function initiatePayment(body: any): Promise<any> {
  if (subscription.value) {
    // Transform the POST billing/payment payload to PATCH subscription/{id} format
    const transformedBody = {
      interval: body.charge?.interval,
      payment_method: body.id,
      product: body.charge?.product_id,
      // TODO: Uncomment when server region is available to PATCH.
      // region: body.metadata?.server_region,
    };

    return await useBaseFetch(`billing/subscription/${subscription.value.id}`, {
      internal: true,
      method: "PATCH",
      body: transformedBody,
    });
  } else {
    addNotification({
      title: "Unable to determine subscription ID.",
      text: "Please contact support.",
      type: "error",
    });
    return Promise.reject(new Error("Unable to determine subscription ID."));
  }
}

async function openUpgradeModal(serverId: string) {
  const subscriptions = (await useBaseFetch(`billing/subscriptions`, { internal: true })) as any[];
  for (const sub of subscriptions) {
    console.log(sub);
    if (sub?.metadata?.type === "pyro" && sub?.metadata?.id === serverId) {
      subscription.value = sub;
      break;
    }
  }

  purchaseModal.value?.show("quarterly");
}

onMounted(() => {
  fetchPaymentData();
  pingRegions();
});
</script>
