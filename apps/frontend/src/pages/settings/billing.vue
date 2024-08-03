<template>
  <section class="universal-card">
    <h2>Subscriptions</h2>
    <p>Manage your Modrinth subscriptions.</p>
    <div class="universal-card recessed">
      <div class="flex justify-between">
        <div class="flex flex-col gap-4">
          <span v-if="false">You're currently subscribed to:</span>
          <span v-else>Become a subscriber to Modrinth Plus!</span>
          <ModrinthPlusIcon class="h-8 w-min" />
          <div class="flex flex-col gap-2">
            <span class="font-bold">Benefits</span>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Ad-free browsing on modrinth.com and Modrinth App</span>
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
        <div class="flex flex-col justify-between text-right">
          <div class="flex flex-col gap-1">
            <span class="text-2xl font-bold text-dark">$5/month</span>
            <span v-if="false" class="text-sm text-secondary">Since June 21, 2024</span>
            <span v-if="false" class="text-sm text-secondary">Renews July 21, 2024</span>
            <span v-else class="text-sm text-secondary">Or $50/year (save 20%)!</span>
          </div>
          <button v-if="false" class="btn ml-auto"><SettingsIcon /> Manage</button>
          <button v-else class="btn btn-purple btn-large ml-auto">
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
    <Modal ref="purchaseModal">
      <div class="test"></div>
    </Modal>
    <Modal ref="addPaymentMethodModal" header="Add payment method">
      <div class="universal-modal min-h-[16rem]">
        <div
          v-show="loadingPaymentMethodModal !== 2"
          class="flex min-h-[16rem] items-center justify-center"
        >
          <AnimatedLogo class="w-[80px]" />
        </div>
        <div v-show="loadingPaymentMethodModal === 2" id="address-element"></div>
        <div v-show="loadingPaymentMethodModal === 2" id="payment-element" class="mt-4"></div>
        <div v-show="loadingPaymentMethodModal === 2" class="input-group push-right mt-auto pt-4">
          <button class="btn" @click="$refs.addPaymentMethodModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button class="btn btn-primary" :disabled="loadingAddMethod" @click="submit">
            <PlusIcon />
            Add method
          </button>
        </div>
      </div>
    </Modal>
    <div class="header__row">
      <div class="header__title">
        <h2 class="text-2xl">Payment methods</h2>
      </div>
      <button class="btn" @click="addPaymentMethod"><PlusIcon /> Add payment method</button>
    </div>
    <div v-if="paymentMethods.length === 0" class="universal-card recessed !mb-0">
      You have not added any payment methods.
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
              <div class="text-contrast font-bold">
                <template v-if="method.type === 'card'">
                  <span v-if="method.card.brand === 'visa'">Visa</span>
                  <span v-else-if="method.card.brand === 'amex'">American Express</span>
                  <span v-else-if="method.card.brand === 'diners'">Diners Club</span>
                  <span v-else-if="method.card.brand === 'discover'">Discover</span>
                  <span v-else-if="method.card.brand === 'eftpos'">Eftpos</span>
                  <span v-else-if="method.card.brand === 'jcb'">JCB</span>
                  <span v-else-if="method.card.brand === 'mastercard'">MasterCard</span>
                  <span v-else-if="method.card.brand === 'unionpay'">UnionPay</span>
                  <span v-else>Unknown card</span>
                  ending in {{ method.card.last4 }}
                </template>
                <template v-else-if="method.type === 'cashapp'">Cash App Pay</template>
                <template v-else-if="method.type === 'amazon_pay'">Amazon Pay</template>
                <template v-else-if="method.type === 'alipay'">AliPay</template>
                <template v-else-if="method.type === 'wechat_pay'">WeChat Pay</template>
                <template v-else-if="method.type === 'paypal'">PayPal</template>
                <template v-else>Unknown payment method</template>
              </div>
              <div
                v-if="primaryPaymentMethodId === method.id"
                class="border-r-ma rounded-full bg-button-bg px-2 py-0.5 text-sm font-bold text-secondary"
              >
                Primary
              </div>
            </div>
            <div v-if="method.type === 'card'" class="text-secondary">
              Expires {{ method.card.exp_month }}/{{ method.card.exp_year }}
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
            Make primary
          </template>
          <template #edit>
            <EditIcon />
            Edit
          </template>
          <template #remove>
            <TrashIcon />
            Remove
          </template>
        </OverflowMenu>
      </div>
    </div>
  </section>
</template>
<script setup>
import { ConfirmModal, Modal, OverflowMenu, AnimatedLogo } from "@modrinth/ui";
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
  SettingsIcon,
  RightArrowIcon,
  ModrinthPlusIcon,
} from "@modrinth/assets";

const data = useNuxtApp();

const { formatMessage } = useVIntl();

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

let stripe = null;
let elements = null;

function loadStripe() {
  stripe = Stripe(
    "pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b",
  );
}

useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
      onload: loadStripe,
    },
  ],
});

onMounted(() => {
  loadStripe();
});

const [
  { data: paymentMethods, refresh: refreshPaymentMethods },
  { data: customer, refresh: refreshCustomer },
] = await Promise.all([
  useAsyncData("billing/payment_methods", () =>
    useBaseFetch("billing/payment_methods", { internal: true }),
  ),
  useAsyncData("billing/customer", () => useBaseFetch("billing/customer", { internal: true })),
]);

async function refresh() {
  await Promise.all([refreshPaymentMethods(), refreshCustomer()]);
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

    const styles = getComputedStyle(document.body);

    elements = stripe.elements({
      theme: "night",
      clientSecret: result.client_secret,
      appearance: {
        variables: {
          colorPrimary: styles.getPropertyValue("--color-brand"),
          colorBackground: styles.getPropertyValue("--color-bg"),
          colorText: styles.getPropertyValue("--color-base"),
          colorTextPlaceholder: styles.getPropertyValue("--color-secondary"),
          colorDanger: styles.getPropertyValue("--color-red"),
          fontFamily: styles.getPropertyValue("--font-standard"),
          spacingUnit: "0.25rem",
          borderRadius: "1rem",
        },
      },
      loader: "never",
    });

    const paymentElement = elements.create("payment");
    paymentElement.mount("#payment-element");
    paymentElement.on("ready", () => {
      loadingPaymentMethodModal.value += 1;
    });

    const addressElement = elements.create("address", {
      mode: "billing",
      contacts: [
        ...new Set(
          paymentMethods.value.map((x) => ({
            address: x.billing_details.address,
            email: x.billing_details.email,
            name: x.billing_details.name,
          })),
        ),
      ],
    });
    addressElement.mount("#address-element");
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
  const { error } = await stripe.confirmSetup({
    elements,
    confirmParams: {
      return_url: "http://localhost:3000/settings/billing",
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

const purchaseModal = ref();
</script>
