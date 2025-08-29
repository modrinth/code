<template>
  <PurchaseModal
    ref="purchaseModal"
    :product="midasProduct"
    :country="country"
    :publishable-key="config.public.stripePublishableKey"
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
    <div class="flex max-w-(--breakpoint-lg) flex-col items-center gap-4 text-center">
      <ModrinthPlusIcon class="text-contrast h-8 w-max" />
      <h1 class="m-0 text-[4rem]">Support creators and go ad-free</h1>
      <p class="m-0 mb-4 text-[18px] leading-relaxed">
        Subscribe to Modrinth Plus to go ad-free, support Modrinth's development, and get an
        exclusive profile badge! Half your subscription goes directly to Modrinth creators. Cancel
        anytime.
      </p>
      <p class="text-purple m-0 text-[2rem] font-bold">
        {{ formatPrice(vintl.locale, price.prices.intervals.monthly, price.currency_code) }}/mo
      </p>
      <p class="text-secondary m-0 mb-4">
        or save
        {{ calculateSavings(price.prices.intervals.monthly, price.prices.intervals.yearly) }}% with
        annual billing!
      </p>
      <nuxt-link
        v-if="auth.user && isPermission(auth.user.badges, 1 << 0)"
        to="/settings/billing"
        class="btn btn-purple btn-large"
      >
        <SettingsIcon aria-hidden="true" />
        Manage subscription
      </nuxt-link>
      <button v-else-if="auth.user" class="btn btn-purple btn-large" @click="purchaseModal.show()">
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
  <div class="perks-hero">
    <h2>What you get with Modrinth Plus!</h2>
    <div class="mt-8 grid max-w-(--breakpoint-lg) gap-8 lg:grid-cols-3">
      <div class="bg-bg-raised flex flex-col gap-4 rounded-xl p-4">
        <HeartIcon class="text-purple h-8 w-8" />
        <span class="text-lg font-bold">Support Modrinth creators</span>
        <span class="text-secondary leading-5">
          50% of your subscription goes directly to Modrinth creators.
        </span>
      </div>
      <div class="bg-bg-raised flex flex-col gap-4 rounded-xl p-4">
        <SparklesIcon class="text-purple h-8 w-8" />
        <span class="text-lg font-bold">Remove all ads</span>
        <span class="text-secondary leading-5">
          Never see an advertisement again on the Modrinth app or the website.
        </span>
      </div>
      <div class="bg-bg-raised flex flex-col gap-4 rounded-xl p-4">
        <StarIcon class="text-purple h-8 w-8" />
        <span class="text-lg font-bold">Profile badge</span>
        <span class="text-secondary leading-5">Get an exclusive badge on your user page.</span>
      </div>
    </div>
    <span class="text-secondary mt-4">...and much more coming soon!</span>
  </div>
</template>
<script setup>
import {
  ModrinthPlusIcon,
  HeartIcon,
  SparklesIcon,
  StarIcon,
  SettingsIcon,
} from "@modrinth/assets";
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

useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
      defer: true,
      async: true,
    },
  ],
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
  background: linear-gradient(360deg, rgba(199, 138, 255, 0.2) 10.92%, var(--color-bg) 100%),
    var(--color-accent-contrast);
  margin-top: -5rem;
  padding: 11.25rem 1rem 8rem;

  display: flex;
  align-items: center;
  flex-direction: column;
}

.perks-hero {
  background-color: var(--color-accent-contrast);
  display: flex;
  align-items: center;
  flex-direction: column;
  padding: 4rem 1rem;
}
</style>
