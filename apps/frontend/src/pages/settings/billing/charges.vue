<template>
  <div>
    <section class="card">
      <Breadcrumbs
        current-title="Past charges"
        :link-stack="[{ href: '/settings/billing', label: 'Billing and subscriptions' }]"
      />
      <h2>Past charges</h2>
      <p>All of your past charges to your Modrinth account will be listed here:</p>
      <div
        v-for="charge in charges"
        :key="charge.id"
        class="universal-card recessed flex items-center justify-between gap-4"
      >
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-1">
            <span class="font-bold text-primary">
              <template v-if="charge.product.metadata.type === 'midas'"> Modrinth Plus </template>
              <template v-else-if="charge.product.metadata.type === 'pyro'">
                Modrinth Servers
              </template>
              <template v-else> Unknown product </template>
              <template v-if="charge.subscription_interval">
                {{ charge.subscription_interval }}
              </template>
            </span>
            ⋅
            <span>{{ formatPrice(vintl.locale, charge.amount, charge.currency_code) }}</span>
          </div>
          <div class="flex items-center gap-1">
            <Badge :color="charge.status === 'succeeded' ? 'green' : 'red'" :type="charge.status" />
            ⋅
            {{ $dayjs(charge.due).format("YYYY-MM-DD") }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
<script setup>
import { Breadcrumbs, Badge } from "@modrinth/ui";
import { formatPrice } from "@modrinth/utils";
import { products } from "~/generated/state.json";

definePageMeta({
  middleware: "auth",
});

const vintl = useVIntl();

const { data: charges } = await useAsyncData(
  "billing/payments",
  () => useBaseFetch("billing/payments", { internal: true }),
  {
    transform: (charges) => {
      return charges
        .filter((charge) => charge.status !== "open" && charge.status !== "cancelled")
        .map((charge) => {
          const product = products.find((product) =>
            product.prices.some((price) => price.id === charge.price_id),
          );

          charge.product = product;

          return charge;
        });
    },
  },
);
</script>
