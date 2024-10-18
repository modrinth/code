<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div
      v-if="status === 'pending'"
      class="flex h-full w-full items-center justify-center gap-6 p-20"
    >
      <AnimatedLogo />
    </div>
    <div v-else-if="subscription" class="flex h-full w-full flex-col justify-between gap-6">
      <div class="card flex h-full w-full flex-col gap-4 rounded-xl p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <h2 class="m-0 text-lg font-bold">Linked subscription</h2>
          </div>
        </div>

        <div
          class="card flex w-full flex-wrap justify-between gap-4 xl:w-auto xl:flex-col [&&]:bg-bg"
        >
          <div class="flex justify-between">
            <div class="flex flex-col gap-4">
              <div class="flex items-center gap-1 text-xl font-bold text-contrast">
                <ModrinthIcon class="h-8 w-8" /> Modrinth <span class="text-brand">Server</span>
              </div>

              <div class="flex flex-col gap-2">
                <div>
                  <h1 class="m-0 text-xl font-semibold leading-none text-primary">
                    {{ product ? getProductSize(product) : "" }} Plan
                  </h1>
                  <span class="text-sm text-secondary">
                    {{ product ? getProductDescription(product) : "" }}
                  </span>
                </div>
                <div class="ml-2 mt-2 flex flex-col gap-2">
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span> {{ product?.metadata?.cpu }} V-CPU cores </span>
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
              <div class="flex gap-2 text-xl font-bold text-contrast">
                <span class="text-primary">
                  {{
                    formatPrice(
                      vintl.locale,
                      getProductPrice(product).prices.intervals.monthly,
                      getProductPrice(product).currency_code,
                    )
                  }}
                </span>
                <span class="text-secondary">/month</span>
              </div>
              <NuxtLink to="/settings/billing">
                <Button>
                  <span class="text-contrast">Manage</span>
                </Button>
              </NuxtLink>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="flex h-full w-full items-center justify-center gap-6 p-20">
      Sorry, we couldn't find your server subscription. Please contact support if this issue
      persists.
    </div>
  </div>
</template>

<script setup lang="ts">
import { AnimatedLogo, Button } from "@modrinth/ui";
import { CheckCircleIcon, ModrinthIcon } from "@modrinth/assets";
// @ts-ignore
import { formatPrice, getCurrency } from "@modrinth/utils";
import { products } from "~/generated/state.json";

const route = useNativeRoute();
const serverId = route.params.id as string;
const country = useUserCountry();
const vintl = useVIntl();

const pyroProducts = products.filter((p) => p.metadata.type === "pyro");

const { data: subscription, status } = useAsyncData("billing/subscriptions", async () => {
  const data = (await useBaseFetch("billing/subscriptions", { internal: true })) as any;
  return data.filter((s: any) => s.metadata.type === "pyro" && s.metadata.id === serverId);
});

const product = computed(() =>
  pyroProducts.find((p) => p.prices.find((x) => x.id === subscription.value?.price_id) !== null),
);

const getProductSize = (product: any) => {
  const ramSize = parseInt(product.metadata.ram);
  if (ramSize === 4096) return "Small";
  if (ramSize === 6144) return "Medium";
  if (ramSize === 8192) return "Large";
  return "Custom";
};

const getProductPrice = (product: any) => {
  return (
    product.prices.find((p: any) => p.currency_code === getCurrency(country.value)) ??
    product.prices.find((p: any) => p.currency_code === "USD") ??
    product.prices[0]
  );
};

const getProductDescription = (product: any) => {
  const ramSize = parseInt(product.metadata.ram);
  if (ramSize === 4096)
    return "Perfect for small modpacks and friend groups looking to play together.";
  if (ramSize === 6144)
    return "The best value for most players. Add more mods and friends to your server with ease.";
  if (ramSize === 8192) return "For the biggest modpacks. Play with hundreds of mods at once.";
  return "Custom server configuration.";
};
</script>
