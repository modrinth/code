<template>
  <NewModal ref="refundModal">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">Refund charge</span>
    </template>
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2">
        <label for="visibility" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Refund type
            <span class="text-brand-red">*</span>
          </span>
          <span> The type of refund to issue. </span>
        </label>
        <DropdownSelect
          id="refund-type"
          v-model="refundType"
          :options="refundTypes"
          name="Refund type"
        />
      </div>
      <div v-if="refundType === 'partial'" class="flex flex-col gap-2">
        <label for="amount" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Amount
            <span class="text-brand-red">*</span>
          </span>
          <span>
            Enter the amount in cents of USD. For example for $2, enter 200. (net
            {{ selectedCharge.net }})
          </span>
        </label>
        <input id="amount" v-model="refundAmount" type="number" autocomplete="off" />
      </div>
      <div class="flex flex-col gap-2">
        <label for="unprovision" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Unprovision
            <span class="text-brand-red">*</span>
          </span>
          <span> Whether or not the subscription should be unprovisioned on refund. </span>
        </label>
        <Toggle
          id="unprovision"
          :model-value="unprovision"
          :checked="unprovision"
          @update:model-value="() => (unprovision = !unprovision)"
        />
      </div>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button :disabled="refunding" @click="refundCharge">
            <CheckIcon aria-hidden="true" />
            Refund charge
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="refundModal.hide()">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
  <div class="normal-page no-sidebar">
    <h1>{{ user.username }}'s subscriptions</h1>
    <div class="normal-page__content">
      <div v-for="subscription in subscriptionCharges" :key="subscription.id" class="card">
        <span class="font-extrabold text-contrast">
          <template v-if="subscription.product.metadata.type === 'midas'"> Modrinth Plus </template>
          <template v-else-if="subscription.product.metadata.type === 'pyro'">
            Modrinth Servers
          </template>
          <template v-else> Unknown product </template>
          <template v-if="subscription.interval">
            {{ subscription.interval }}
          </template>
        </span>
        <div class="mb-4 mt-2 flex items-center gap-1">
          {{ subscription.status }} ⋅ {{ $dayjs(subscription.created).format("YYYY-MM-DD") }}
          <template v-if="subscription.metadata?.id"> ⋅ {{ subscription.metadata.id }}</template>
        </div>
        <div
          v-for="charge in subscription.charges"
          :key="charge.id"
          class="universal-card recessed flex items-center justify-between gap-4"
        >
          <div class="flex w-full items-center justify-between gap-4">
            <div class="flex items-center gap-1">
              <Badge
                :color="charge.status === 'succeeded' ? 'green' : 'red'"
                :type="charge.status"
              />
              ⋅
              {{ charge.type }}
              ⋅
              {{ $dayjs(charge.due).format("YYYY-MM-DD") }}
              ⋅
              <span>{{ formatPrice(vintl.locale, charge.amount, charge.currency_code) }}</span>
              <template v-if="subscription.interval"> ⋅ {{ subscription.interval }} </template>
            </div>
            <button
              v-if="charge.status === 'succeeded' && charge.type !== 'refund'"
              class="btn"
              @click="showRefundModal(charge)"
            >
              Refund charge
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup>
import { Badge, NewModal, ButtonStyled, DropdownSelect, Toggle } from "@modrinth/ui";
import { formatPrice } from "@modrinth/utils";
import { CheckIcon, XIcon } from "@modrinth/assets";
import { products } from "~/generated/state.json";

const route = useRoute();
const data = useNuxtApp();
const vintl = useVIntl();
const { formatMessage } = vintl;

const messages = defineMessages({
  userNotFoundError: {
    id: "admin.billing.error.not-found",
    defaultMessage: "User not found",
  },
});

const { data: user } = await useAsyncData(`user/${route.params.id}`, () =>
  useBaseFetch(`user/${route.params.id}`),
);

if (!user.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

let subscriptions, charges, refreshCharges;
try {
  [{ data: subscriptions }, { data: charges, refresh: refreshCharges }] = await Promise.all([
    useAsyncData(`billing/subscriptions?user_id=${route.params.id}`, () =>
      useBaseFetch(`billing/subscriptions?user_id=${user.value.id}`, {
        internal: true,
      }),
    ),
    useAsyncData(`billing/payments?user_id=${route.params.id}`, () =>
      useBaseFetch(`billing/payments?user_id=${user.value.id}`, {
        internal: true,
      }),
    ),
  ]);
} catch {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

const subscriptionCharges = computed(() => {
  return subscriptions.value.map((subscription) => {
    return {
      ...subscription,
      charges: charges.value.filter((charge) => charge.subscription_id === subscription.id),
      product: products.find((product) =>
        product.prices.some((price) => price.id === subscription.price_id),
      ),
    };
  });
});

const refunding = ref(false);
const refundModal = ref();
const selectedCharge = ref(null);
const refundType = ref("full");
const refundTypes = ref(["full", "partial"]);
const refundAmount = ref(0);
const unprovision = ref(false);

function showRefundModal(charge) {
  selectedCharge.value = charge;
  refundType.value = "full";
  refundAmount.value = 0;
  unprovision.value = false;
  refundModal.value.show();
}

async function refundCharge() {
  refunding.value = true;
  try {
    await useBaseFetch(`billing/charge/${selectedCharge.value.id}/refund`, {
      method: "POST",
      body: JSON.stringify({
        type: refundType.value,
        amount: refundAmount.value,
        unprovision: unprovision.value,
      }),
      internal: true,
    });
    await refreshCharges();
    refundModal.value.hide();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "Error refunding",
      text: err.data?.description ?? err,
      type: "error",
    });
  }
  refunding.value = false;
}
</script>
