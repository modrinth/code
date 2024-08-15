<template>
  <PurchaseModal
    ref="purchaseModal"
    :product="midasProduct"
    :country="country"
    publishable-key="pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b"
    :send-billing-request="
      async (body) =>
        await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })
    "
    :fetch-payment-data="fetchPaymentData"
    :on-error="
      (err) =>
        data.$notify({
          group: 'main',
          title: 'An error occurred',
          type: 'error',
          text: err.message ?? (err.data ? err.data.description : err),
        })
    "
    :customer="customer"
    :payment-methods="paymentMethods"
    :return-url="`${config.public.siteUrl}/settings/billing`"
  />
  <div class="main-hero">
    <div class="flex max-w-screen-lg flex-col items-center gap-4 text-center">
      <ModrinthPlusIcon class="h-8 w-max" />
      <h1 class="hidden">Modrinth Plus</h1>
      <h1 class="m-0 text-[4rem]">Support creators and go ad-free</h1>
      <p class="m-0 mb-4 text-[18px] leading-relaxed">
        Subscribe to Modrinth Plus to go ad-free, support Modrinth's development, and get an
        exclusive profile badge! Half your subscription goes directly to Modrinth creators. Cancel
        anytime.
      </p>
      <p class="m-0 text-[2rem] font-bold text-purple">
        {{ formatPrice(vintl.locale, price.prices.intervals.monthly, price.currency_code) }}/mo
      </p>
      <p class="m-0 mb-4 text-secondary">
        or save
        {{ calculateSavings(price.prices.intervals.monthly, price.prices.intervals.yearly) }}% with
        annual billing!
      </p>
      <button v-if="auth.user" class="btn btn-purple btn-large" @click="purchaseModal.show()">
        Subscribe
      </button>
      <nuxt-link
        v-else
        :to="`/auth/sign-in?redirect=${encodeURIComponent('/plus?showModal=true')}`"
        class="btn btn-purple btn-large"
      >
        Subscribe
      </nuxt-link>
    </div>
  </div>
</template>
<script setup>
import { ModrinthPlusIcon } from "@modrinth/assets";
import { PurchaseModal } from "@modrinth/ui";
import { calculateSavings, formatPrice, getCurrency } from "@modrinth/utils";
import { products } from "~/generated/state.json";

const title = "Subscribe to Modrinth Plus!";
const description =
  "Subscribe to Modrinth Plus to go ad-free, support Modrinth's development, and get an exclusive profile badge! Half your subscription goes directly to Modrinth creators.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

const vintl = useVIntl();

const data = useNuxtApp();
const config = useRuntimeConfig();

const auth = await useAuth();
const purchaseModal = ref();
const midasProduct = ref(products.find((x) => x.metadata.type === "midas"));
const country = useUserCountry();
const price = computed(() =>
  midasProduct.value.prices.find((x) => x.currency_code === getCurrency(country.value)),
);
const customer = ref();
const paymentMethods = ref([]);

async function fetchPaymentData() {
  [customer.value, paymentMethods.value] = await Promise.all([
    useBaseFetch("billing/customer", { internal: true }),
    useBaseFetch("billing/payment_methods", { internal: true }),
  ]);
}

const route = useRoute();
onMounted(() => {
  if (route.query.showModal) {
    purchaseModal.value.show();
  }
});
</script>
<style lang="scss" scoped>
.main-hero {
  background-color: #000;
  margin-top: -4rem;
  padding: 11.25rem 1rem 12rem;

  display: flex;
  justify-content: center;
  align-items: center;
  text-align: center;
  flex-direction: column;

  svg {
    color: var(--color-contrast);
  }
}
</style>
