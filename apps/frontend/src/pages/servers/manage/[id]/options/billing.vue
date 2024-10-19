<template>
  <div class="relative h-full w-full">
    <div
      v-if="status === 'pending'"
      class="flex h-full w-full items-center justify-center gap-6 p-20"
    >
      <AnimatedLogo />
    </div>
    <div
      v-else-if="subscription && subscription.length > 0"
      data-pyro-billing
      class="flex h-full w-full flex-col justify-between gap-6"
    >
      <div class="card flex h-full w-full flex-col gap-4 rounded-xl p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <h2 class="m-0 text-lg font-bold">Linked subscription</h2>
          </div>

          {{
            currentCharge?.status === "open"
              ? "Subscription Active"
              : currentCharge?.status === "processing"
                ? "Processing"
                : currentCharge?.status === "cancelled"
                  ? "Cancelled"
                  : currentCharge?.status === "failed"
                    ? "Failed"
                    : ""
          }}
        </div>

        <div
          data-pyro-charge-card
          class="card flex w-full flex-wrap justify-between gap-4 xl:w-auto xl:flex-col [&&]:bg-bg"
        >
          <div class="flex justify-between">
            <div class="flex flex-col gap-4">
              <LazyUiServersModrinthServersIcon class="flex h-8 w-fit" />

              <div class="flex flex-col gap-2">
                <div class="flex flex-col gap-2">
                  <h1 class="m-0 text-xl font-semibold leading-none text-contrast">
                    {{ product ? getProductSize(product) : "" }} Plan
                  </h1>
                  <!-- <span class="text-sm text-secondary">
                    {{ product ? getProductDescription(product) : "" }}
                  </span> -->
                </div>
                <div class="mt-2 flex flex-col gap-2">
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span> {{ product?.metadata?.cpu }} vCores (CPU) </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{ product?.metadata?.ram ? product.metadata.ram / 1024 + " GB RAM" : "" }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{ product?.metadata?.swap ? product.metadata.swap / 1024 + " GB Swap" : "" }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{
                        product?.metadata?.storage
                          ? product?.metadata?.storage / 1024 + " GB SSD"
                          : ""
                      }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
            <div class="flex flex-col items-end justify-between">
              <div class="flex flex-col items-end gap-2">
                <div class="flex text-2xl font-bold text-contrast">
                  <span class="text-contrast">
                    {{
                      formatPrice(
                        vintl.locale,
                        getProductPrice(product!, subscription[0].interval as "yearly" | "monthly")
                          .prices.intervals[subscription[0].interval as "yearly" | "monthly"] ?? 0,
                        getProductPrice(product!, subscription[0].interval as "yearly" | "monthly")
                          .currency_code,
                      )
                    }}
                  </span>
                  <span>/{{ subscription[0].interval.replace("ly", "") }}</span>
                </div>
                <div v-if="currentCharge">
                  <span v-if="currentCharge.status === 'open'" class="text-sm text-secondary">
                    Renews {{ $dayjs(currentCharge.due).format("MMMM D, YYYY") }}
                  </span>
                  <span
                    v-else-if="currentCharge.status === 'processing'"
                    class="text-sm text-orange"
                  >
                    Your payment is being processed. Perks will activate once payment is complete.
                  </span>
                  <span
                    v-else-if="currentCharge.status === 'cancelled'"
                    class="text-sm text-secondary"
                  >
                    Expires {{ $dayjs(currentCharge.due).format("MMMM D, YYYY") }}
                  </span>
                  <span v-else-if="currentCharge.status === 'failed'" class="text-sm text-red">
                    Your subscription payment failed. Please update your payment method.
                  </span>
                </div>
              </div>
              <div class="flex gap-2">
                <ButtonStyled>
                  <NuxtLink to="/settings/billing">
                    <span class="whitespace-pre text-contrast">Manage Payment Method</span>
                  </NuxtLink>
                </ButtonStyled>
                <ButtonStyled
                  v-if="currentCharge && currentCharge.status !== 'cancelled'"
                  type="standard"
                  color="red"
                  @click="showCancelModal"
                >
                  <button class="text-contrast">Cancel</button>
                </ButtonStyled>
                <ButtonStyled
                  v-else-if="currentCharge && currentCharge.status === 'cancelled'"
                  type="standard"
                  color="green"
                  @click="resubscribe"
                >
                  <button class="text-contrast">Resubscribe</button>
                </ButtonStyled>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="flex h-full w-full items-center justify-center gap-6 p-20">
      An error occurred and your subscription could not be found. Report this to Modrinth support
      with this information:
      <div class="flex flex-col gap-2">
        <LazyUiCopyCode :text="serverId" />
        <LazyUiCopyCode :text="JSON.stringify(subscription)" />
      </div>
    </div>
    <ConfirmModal
      ref="cancelModal"
      :title="formatMessage(cancelModalMessages.title)"
      :description="formatMessage(cancelModalMessages.description)"
      :proceed-label="formatMessage(cancelModalMessages.action)"
      @proceed="cancelSubscription"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { AnimatedLogo, ButtonStyled, ConfirmModal } from "@modrinth/ui";
import { CheckCircleIcon } from "@modrinth/assets";
import { formatPrice, getCurrency } from "@modrinth/utils";

interface Subscription {
  id: string;
  user_id: string;
  price_id: string;
  interval: string;
  status: string;
  created: string;
  metadata: {
    type: string;
    id: string;
  };
}

interface Product {
  id: string;
  metadata: {
    type: string;
    cpu: number;
    ram: number;
    swap: number;
    storage: number;
  };
  prices: Array<{
    id: string;
    product_id: string;
    prices: {
      type: string;
      intervals: {
        yearly?: number;
        monthly?: number;
      };
    };
    currency_code: string;
  }>;
  unitary: boolean;
}

interface Charge {
  id: string;
  user_id: string;
  price_id: string;
  amount: number;
  currency_code: string;
  status: "open" | "processing" | "cancelled" | "failed" | "succeeded";
  due: string;
  last_attempt: string | null;
  type: string;
  subscription_id: string;
  subscription_interval: "monthly" | "yearly";
}

const route = useNativeRoute();
const serverId = route.params.id as string;
const country = useUserCountry();
const vintl = useVIntl();
const { formatMessage } = vintl;

const { data: productsData } = useAsyncData<Product[]>("products", async () => {
  const data = (await useBaseFetch("billing/products", { internal: true })) as Product[];
  return data;
});

const { data: subscription, status } = useAsyncData<Subscription[]>(
  "billing/subscriptions",
  async () => {
    const data = (await useBaseFetch("billing/subscriptions", {
      internal: true,
    })) as Subscription[];
    const filteredSubscriptions = data.filter(
      (s: Subscription) => s.metadata?.type === "pyro" && s.metadata?.id === serverId,
    );
    return filteredSubscriptions;
  },
);

const { data: charges } = useAsyncData<Charge[]>("billing/payments", async () => {
  const data = (await useBaseFetch("billing/payments", { internal: true })) as Charge[];
  return data;
});

const product = computed(() => {
  if (!subscription.value || subscription.value.length === 0) {
    return null;
  }
  const foundProduct = productsData.value?.find(
    (p) =>
      subscription.value &&
      subscription.value.length > 0 &&
      p.prices.some((x) => x.id === subscription.value![0]?.price_id),
  );
  return foundProduct;
});

const currentCharge = computed(() => {
  if (!subscription.value || subscription.value.length === 0 || !charges.value) {
    return null;
  }
  return charges.value.find(
    (charge) =>
      charge.subscription_id === subscription.value![0].id && charge.status !== "succeeded",
  );
});

const getProductSize = (product: Product) => {
  const ramSize = product.metadata.ram;
  if (ramSize === 4096) return "Small";
  if (ramSize === 6144) return "Medium";
  if (ramSize === 8192) return "Large";
  return "Custom";
};

const getProductPrice = (product: Product, interval: "yearly" | "monthly") => {
  return (
    product.prices.find(
      (p) => p.currency_code === getCurrency(country.value) && p.prices.intervals[interval],
    ) ??
    product.prices.find((p) => p.currency_code === "USD" && p.prices.intervals[interval]) ??
    product.prices[0]
  );
};

// const getProductDescription = (product: Product) => {
//   const ramSize = product.metadata.ram;
//   if (ramSize === 4096)
//     return "Perfect for small modpacks and friend groups looking to play together.";
//   if (ramSize === 6144)
//     return "The best value for most players. Add more mods and friends to your server with ease.";
//   if (ramSize === 8192) return "For the biggest modpacks. Play with hundreds of mods at once.";
//   return "Custom server configuration.";
// };

const cancelModal = ref<InstanceType<typeof ConfirmModal> | null>(null);

const cancelModalMessages = defineMessages({
  title: {
    id: "settings.billing.modal.cancel.title",
    defaultMessage: "Are you sure you want to cancel your subscription?",
  },
  description: {
    id: "settings.billing.modal.cancel.description",
    defaultMessage:
      "This will cancel your subscription. You will retain your perks until the end of the current billing cycle.",
  },
  action: {
    id: "settings.billing.modal.cancel.action",
    defaultMessage: "Cancel subscription",
  },
});

const showCancelModal = () => {
  cancelModal.value?.show();
};

const cancelSubscription = async () => {
  if (!subscription.value || subscription.value.length === 0) return;

  const id = subscription.value[0].id;
  try {
    await useBaseFetch(`billing/subscription/${id}`, {
      internal: true,
      method: "PATCH",
      body: {
        cancelled: true,
      },
    });
    await refreshNuxtData("billing/subscriptions");
    await refreshNuxtData("billing/payments");
  } catch (err: any) {
    console.error(
      "An error occurred while cancelling the subscription:",
      err.data?.description || err.message,
    );
    addNotification({
      group: "billing",
      title: "Error cancelling subscription",
      text: "An error occurred while cancelling your subscription.",
      type: "error",
    });
  }
};

const resubscribe = async () => {
  if (!subscription.value || subscription.value.length === 0) return;

  const id = subscription.value[0].id;
  try {
    await useBaseFetch(`billing/subscription/${id}`, {
      internal: true,
      method: "PATCH",
      body: {
        cancelled: false,
      },
    });
    await refreshNuxtData("billing/subscriptions");
    await refreshNuxtData("billing/payments");
  } catch (err: any) {
    console.error("An error occurred while resubscribing:", err.data?.description || err.message);
    addNotification({
      group: "billing",
      title: "Error resubscribing",
      text: "An error occurred while resubscribing to your Modrinth server.",
      type: "error",
    });
  }
};
</script>
