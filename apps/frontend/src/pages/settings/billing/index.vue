<template>
  <section class="universal-card">
    <h2>{{ formatMessage(messages.subscriptionTitle) }}</h2>
    <p>{{ formatMessage(messages.subscriptionDescription) }}</p>
    <div class="universal-card recessed">
      <ConfirmModal
        ref="modal_cancel"
        :title="formatMessage(cancelModalMessages.title)"
        :description="formatMessage(cancelModalMessages.description)"
        :proceed-label="formatMessage(cancelModalMessages.action)"
        @proceed="cancelSubscription(cancelSubscriptionId, true)"
      />
      <div class="flex flex-wrap justify-between gap-4">
        <div class="flex flex-col gap-4">
          <template v-if="midasCharge">
            <span v-if="midasCharge.status === 'open'"> You're currently subscribed to: </span>
            <span v-else-if="midasCharge.status === 'processing'" class="text-orange">
              Your payment is being processed. Perks will activate once payment is complete.
            </span>
            <span v-else-if="midasCharge.status === 'cancelled'">
              You've cancelled your subscription. <br />
              You will retain your perks until the end of the current billing cycle.
            </span>
            <span v-else-if="midasCharge.status === 'failed'" class="text-red">
              Your subscription payment failed. Please update your payment method.
            </span>
          </template>

          <span v-else>Become a subscriber to Modrinth Plus!</span>
          <ModrinthPlusIcon class="h-8 w-min" />
          <div class="flex flex-col gap-2">
            <span class="font-bold">Benefits</span>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span> Ad-free browsing on modrinth.com and Modrinth App </span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Modrinth+ badge on your profile</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Support Modrinth and creators directly</span>
            </div>
          </div>
        </div>
        <div class="flex w-full flex-wrap justify-between gap-4 xl:w-auto xl:flex-col">
          <div class="flex flex-col gap-1 xl:ml-auto xl:text-right">
            <span class="text-2xl font-bold text-dark">
              <template v-if="midasCharge">
                {{
                  formatPrice(
                    vintl.locale,
                    midasSubscriptionPrice.prices.intervals[midasCharge.subscription_interval],
                    midasSubscriptionPrice.currency_code,
                  )
                }}
                /
                {{ midasCharge.subscription_interval }}
              </template>
              <template v-else>
                {{ formatPrice(vintl.locale, price.prices.intervals.monthly, price.currency_code) }}
                / month
              </template>
            </span>
            <template v-if="midasCharge">
              <span class="text-sm text-secondary">
                Since {{ $dayjs(midasSubscription.created).format("MMMM D, YYYY") }}
              </span>
              <span v-if="midasCharge.status === 'open'" class="text-sm text-secondary">
                Renews {{ $dayjs(midasCharge.due).format("MMMM D, YYYY") }}
              </span>
              <span v-else-if="midasCharge.status === 'cancelled'" class="text-sm text-secondary">
                Expires {{ $dayjs(midasCharge.due).format("MMMM D, YYYY") }}
              </span>
            </template>

            <span v-else class="text-sm text-secondary">
              Or
              {{ formatPrice(vintl.locale, price.prices.intervals.yearly, price.currency_code) }} /
              year (save
              {{
                calculateSavings(price.prices.intervals.monthly, price.prices.intervals.yearly)
              }}%)!
            </span>
          </div>
          <div
            v-if="midasCharge && midasCharge.status === 'failed'"
            class="ml-auto flex flex-row-reverse items-center gap-2"
          >
            <button
              v-if="midasCharge && midasCharge.status === 'failed'"
              class="iconified-button raised-button"
              @click="
                () => {
                  purchaseModalStep = 0;
                  $refs.purchaseModal.show();
                }
              "
            >
              <UpdatedIcon />
              Update method
            </button>
            <OverflowMenu
              class="btn icon-only transparent"
              :options="[
                {
                  id: 'cancel',
                  action: () => {
                    cancelSubscriptionId = midasSubscription.id;
                    $refs.modal_cancel.show();
                  },
                },
              ]"
            >
              <MoreVerticalIcon />
              <template #cancel><XIcon /> Cancel</template>
            </OverflowMenu>
          </div>
          <button
            v-else-if="midasCharge && midasCharge.status !== 'cancelled'"
            class="iconified-button raised-button !ml-auto"
            @click="
              () => {
                cancelSubscriptionId = midasSubscription.id;
                $refs.modal_cancel.show();
              }
            "
          >
            <XIcon /> Cancel
          </button>
          <button
            v-else-if="midasCharge && midasCharge.status === 'cancelled'"
            class="btn btn-purple btn-large ml-auto"
            @click="cancelSubscription(midasSubscription.id, false)"
          >
            <RightArrowIcon /> Resubscribe
          </button>
          <button
            v-else
            class="btn btn-purple btn-large ml-auto"
            @click="
              () => {
                purchaseModalStep = 0;
                $refs.purchaseModal.show();
              }
            "
          >
            <RightArrowIcon />
            Subscribe
          </button>
        </div>
      </div>
    </div>
  </section>
  <section class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      :title="formatMessage(deleteModalMessages.title)"
      :description="formatMessage(deleteModalMessages.description)"
      :proceed-label="formatMessage(deleteModalMessages.action)"
      @proceed="removePaymentMethod(removePaymentMethodIndex)"
    />
    <PurchaseModal
      ref="purchaseModal"
      :product="midasProduct"
      :country="country"
      :publishable-key="config.public.stripePublishableKey"
      :send-billing-request="
        async (body) =>
          await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })
      "
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
    <NewModal ref="addPaymentMethodModal">
      <template #title>
        <span class="text-lg font-extrabold text-contrast">
          {{ formatMessage(messages.paymentMethodTitle) }}
        </span>
      </template>
      <div class="min-h-[16rem] md:w-[600px]">
        <div
          v-show="loadingPaymentMethodModal !== 2"
          class="flex min-h-[16rem] items-center justify-center"
        >
          <AnimatedLogo class="w-[80px]" />
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="min-h-[16rem] p-1">
          <div id="address-element"></div>
          <div id="payment-element" class="mt-4"></div>
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="input-group push-right mt-auto pt-4">
          <button class="btn" @click="$refs.addPaymentMethodModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button class="btn btn-primary" :disabled="loadingAddMethod" @click="submit">
            <PlusIcon />
            {{ formatMessage(messages.paymentMethodAdd) }}
          </button>
        </div>
      </div>
    </NewModal>
    <div class="header__row">
      <div class="header__title">
        <h2 class="text-2xl">{{ formatMessage(messages.paymentMethodTitle) }}</h2>
      </div>
      <nuxt-link class="btn" to="/settings/billing/charges">
        <HistoryIcon /> {{ formatMessage(messages.paymentMethodHistory) }}
      </nuxt-link>
      <button class="btn" @click="addPaymentMethod">
        <PlusIcon /> {{ formatMessage(messages.paymentMethodAdd) }}
      </button>
    </div>
    <div
      v-if="!paymentMethods || paymentMethods.length === 0"
      class="universal-card recessed !mb-0"
    >
      {{ formatMessage(messages.paymentMethodNone) }}
    </div>
    <div v-else class="flex flex-col gap-4">
      <div
        v-for="(method, index) in paymentMethods"
        :key="index"
        class="universal-card recessed !mb-0 flex items-center justify-between"
      >
        <div class="flex gap-2">
          <CardIcon v-if="method.type === 'card'" class="h-8 w-8" />
          <CurrencyIcon v-else-if="method.type === 'cashapp'" class="h-8 w-8" />
          <PayPalIcon v-else-if="method.type === 'paypal'" class="h-8 w-8" />
          <div class="flex flex-col">
            <div class="flex items-center gap-2">
              <div class="font-bold text-contrast">
                <template v-if="method.type === 'card'">
                  {{
                    formatMessage(messages.paymentMethodCardDisplay, {
                      card_brand:
                        formatMessage(paymentMethodTypes[method.card.brand]) ??
                        formatMessage(paymentMethodTypes.unknown),
                      last_four: method.card.last4,
                    })
                  }}
                </template>
                <template v-else>
                  {{
                    formatMessage(paymentMethodTypes[method.type]) ??
                    formatMessage(paymentMethodTypes.unknown)
                  }}
                </template>
              </div>
              <div
                v-if="primaryPaymentMethodId === method.id"
                class="border-r-ma rounded-full bg-button-bg px-2 py-0.5 text-sm font-bold text-secondary"
              >
                {{ formatMessage(messages.paymentMethodPrimary) }}
              </div>
            </div>
            <div v-if="method.type === 'card'" class="text-secondary">
              {{
                formatMessage(messages.paymentMethodCardExpiry, {
                  month: method.card.exp_month,
                  year: method.card.exp_year,
                })
              }}
            </div>
            <div v-else-if="method.type === 'cashapp'" class="text-secondary">
              {{ method.cashapp.cashtag }}
            </div>
            <div v-else-if="method.type === 'paypal'" class="text-secondary">
              {{ method.paypal.payer_email }}
            </div>
          </div>
        </div>
        <OverflowMenu
          class="btn icon-only transparent"
          :options="
            [
              {
                id: 'primary',
                action: () => editPaymentMethod(index, true),
              },
              {
                id: 'remove',
                action: () => {
                  removePaymentMethodIndex = index;
                  $refs.modal_confirm.show();
                },
                color: 'red',
                hoverOnly: true,
              },
            ].slice(primaryPaymentMethodId === method.id ? 1 : 0, 2)
          "
        >
          <MoreVerticalIcon />
          <template #primary>
            <StarIcon />
            {{ formatMessage(messages.paymentMethodMakePrimary) }}
          </template>
          <template #edit>
            <EditIcon />
            {{ formatMessage(commonMessages.editButton) }}
          </template>
          <template #remove>
            <TrashIcon />
            {{ formatMessage(commonMessages.deleteLabel) }}
          </template>
        </OverflowMenu>
      </div>
    </div>
  </section>
</template>
<script setup>
import { ConfirmModal, NewModal, OverflowMenu, AnimatedLogo, PurchaseModal } from "@modrinth/ui";
import {
  PlusIcon,
  XIcon,
  CardIcon,
  MoreVerticalIcon,
  TrashIcon,
  EditIcon,
  StarIcon,
  PayPalIcon,
  CurrencyIcon,
  CheckCircleIcon,
  RightArrowIcon,
  ModrinthPlusIcon,
  UpdatedIcon,
  HistoryIcon,
} from "@modrinth/assets";
import { calculateSavings, formatPrice, createStripeElements, getCurrency } from "@modrinth/utils";
import { ref } from "vue";
import { products } from "~/generated/state.json";

definePageMeta({
  middleware: "auth",
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

const data = useNuxtApp();
const config = useRuntimeConfig();

const vintl = useVIntl();
const { formatMessage } = vintl;

const deleteModalMessages = defineMessages({
  title: {
    id: "settings.billing.modal.delete.title",
    defaultMessage: "Are you sure you want to remove this payment method?",
  },
  description: {
    id: "settings.billing.modal.delete.description",
    defaultMessage: "This will remove this payment method forever (like really forever).",
  },
  action: {
    id: "settings.billing.modal.delete.action",
    defaultMessage: "Remove this payment method",
  },
});

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

const messages = defineMessages({
  subscriptionTitle: {
    id: "settings.billing.subscription.title",
    defaultMessage: "Subscriptions",
  },
  subscriptionDescription: {
    id: "settings.billing.subscription.description",
    defaultMessage: "Manage your Modrinth subscriptions.",
  },
  paymentMethodTitle: {
    id: "settings.billing.payment_method.title",
    defaultMessage: "Payment methods",
  },
  paymentMethodNone: {
    id: "settings.billing.payment_method.none",
    defaultMessage: "You have not added any payment methods.",
  },
  paymentMethodHistory: {
    id: "settings.billing.payment_method.action.history",
    defaultMessage: "View past charges",
  },
  paymentMethodAdd: {
    id: "settings.billing.payment_method.action.add",
    defaultMessage: "Add payment method",
  },
  paymentMethodPrimary: {
    id: "settings.billing.payment_method.primary",
    defaultMessage: "Primary",
  },
  paymentMethodMakePrimary: {
    id: "settings.billing.payment_method.action.primary",
    defaultMessage: "Make primary",
  },
  paymentMethodCardDisplay: {
    id: "settings.billing.payment_method.card_display",
    defaultMessage: "{card_brand} ending in {last_four}",
  },
  paymentMethodCardExpiry: {
    id: "settings.billing.payment_method.card_expiry",
    defaultMessage: "Expires {month}/{year}",
  },
});

const paymentMethodTypes = defineMessages({
  visa: {
    id: "settings.billing.payment_method_type.visa",
    defaultMessage: "Visa",
  },
  amex: { id: "settings.billing.payment_method_type.amex", defaultMessage: "American Express" },
  diners: { id: "settings.billing.payment_method_type.diners", defaultMessage: "Diners Club" },
  discover: { id: "settings.billing.payment_method_type.discover", defaultMessage: "Discover" },
  eftpos: { id: "settings.billing.payment_method_type.eftpos", defaultMessage: "EFTPOS" },
  jcb: { id: "settings.billing.payment_method_type.jcb", defaultMessage: "JCB" },
  mastercard: {
    id: "settings.billing.payment_method_type.mastercard",
    defaultMessage: "MasterCard",
  },
  unionpay: { id: "settings.billing.payment_method_type.unionpay", defaultMessage: "UnionPay" },
  paypal: { id: "settings.billing.payment_method_type.paypal", defaultMessage: "PayPal" },
  cashapp: { id: "settings.billing.payment_method_type.cashapp", defaultMessage: "Cash App" },
  amazon_pay: {
    id: "settings.billing.payment_method_type.amazon_pay",
    defaultMessage: "Amazon Pay",
  },
  unknown: {
    id: "settings.billing.payment_method_type.unknown",
    defaultMessage: "Unknown payment method",
  },
});

let stripe = null;
let elements = null;

function loadStripe() {
  try {
    if (!stripe) {
      stripe = Stripe(config.public.stripePublishableKey);
    }
  } catch {}
}

const [
  { data: paymentMethods, refresh: refreshPaymentMethods },
  { data: charges, refresh: refreshCharges },
  { data: customer, refresh: refreshCustomer },
  { data: subscriptions, refresh: refreshSubscriptions },
] = await Promise.all([
  useAsyncData("billing/payment_methods", () =>
    useBaseFetch("billing/payment_methods", { internal: true }),
  ),
  useAsyncData("billing/payments", () => useBaseFetch("billing/payments", { internal: true })),
  useAsyncData("billing/customer", () => useBaseFetch("billing/customer", { internal: true })),
  useAsyncData("billing/subscriptions", () =>
    useBaseFetch("billing/subscriptions", { internal: true }),
  ),
]);

async function refresh() {
  await Promise.all([
    refreshPaymentMethods(),
    refreshCharges(),
    refreshCustomer(),
    refreshSubscriptions(),
  ]);
}

const midasProduct = ref(products.find((x) => x.metadata.type === "midas"));
const midasSubscription = computed(() =>
  subscriptions.value.find(
    (x) => x.status === "provisioned" && midasProduct.value.prices.find((y) => y.id === x.price_id),
  ),
);
const midasSubscriptionPrice = computed(() =>
  midasSubscription.value
    ? midasProduct.value.prices.find((x) => x.id === midasSubscription.value.price_id)
    : null,
);
const midasCharge = computed(() =>
  midasSubscription.value
    ? charges.value.find((x) => x.subscription_id === midasSubscription.value.id)
    : null,
);

const purchaseModal = ref();
const country = useUserCountry();
const price = computed(() =>
  midasProduct.value.prices.find((x) => x.currency_code === getCurrency(country.value)),
);

// Initialize subscription with fake data if redirected from checkout
const route = useRoute();
const router = useRouter();
if (route.query.priceId && route.query.plan && route.query.redirect_status) {
  let status;
  if (route.query.redirect_status === "succeeded") {
    status = "active";
  } else if (route.query.redirect_status === "processing") {
    status = "payment-processing";
  } else {
    status = "payment-failed";
  }

  subscriptions.value.push({
    id: "temp",
    price_id: route.query.priceId,
    interval: route.query.plan,
    created: Date.now(),
    status,
  });

  charges.value.push({
    id: "temp",
    price_id: route.query.priceId,
    subscription_id: "temp",
    status: "open",
    due: Date.now() + (route.query.plan === "yearly" ? 31536000000 : 2629746000),
    subscription_interval: route.query.plan,
  });

  await router.replace({ query: {} });
}

const primaryPaymentMethodId = computed(() => {
  if (
    customer.value &&
    customer.value.invoice_settings &&
    customer.value.invoice_settings.default_payment_method
  ) {
    return customer.value.invoice_settings.default_payment_method;
  } else if (paymentMethods.value && paymentMethods.value[0] && paymentMethods.value[0].id) {
    return paymentMethods.value[0].id;
  } else {
    return null;
  }
});

const addPaymentMethodModal = ref();
const loadingPaymentMethodModal = ref(0);
async function addPaymentMethod() {
  try {
    loadingPaymentMethodModal.value = 0;
    addPaymentMethodModal.value.show();

    const result = await useBaseFetch("billing/payment_method", {
      internal: true,
      method: "POST",
    });

    loadStripe();
    const {
      elements: elementsVal,
      addressElement,
      paymentElement,
    } = createStripeElements(stripe, paymentMethods.value, {
      clientSecret: result.client_secret,
    });

    elements = elementsVal;
    paymentElement.on("ready", () => {
      loadingPaymentMethodModal.value += 1;
    });
    addressElement.on("ready", () => {
      loadingPaymentMethodModal.value += 1;
    });
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

const loadingAddMethod = ref(false);
async function submit() {
  startLoading();
  loadingAddMethod.value = true;

  loadStripe();
  const { error } = await stripe.confirmSetup({
    elements,
    confirmParams: {
      return_url: `${config.public.siteUrl}/settings/billing`,
    },
  });

  if (error && error.type !== "validation_error") {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: error.message,
      type: "error",
    });
  } else if (!error) {
    await refresh();
    addPaymentMethodModal.value.close();
  }
  loadingAddMethod.value = false;
  stopLoading();
}

const removePaymentMethodIndex = ref();

async function editPaymentMethod(index, primary) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "PATCH",
      data: {
        primary,
      },
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

async function removePaymentMethod(index) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "DELETE",
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const cancelSubscriptionId = ref();
async function cancelSubscription(id, cancelled) {
  startLoading();
  try {
    await useBaseFetch(`billing/subscription/${id}`, {
      internal: true,
      method: "PATCH",
      body: {
        cancelled,
      },
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}
</script>
