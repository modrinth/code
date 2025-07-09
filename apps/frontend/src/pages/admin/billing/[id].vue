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
        <Toggle id="unprovision" v-model="unprovision" />
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
  <div class="page experimental-styles-within">
    <div
      class="mb-4 flex items-center justify-between border-0 border-b border-solid border-divider pb-4"
    >
      <div class="flex items-center gap-2">
        <Avatar :src="user.avatar_url" :alt="user.username" size="32px" circle />
        <h1 class="m-0 text-2xl font-extrabold">{{ user.username }}'s subscriptions</h1>
      </div>
      <div class="flex items-center gap-2">
        <ButtonStyled>
          <nuxt-link :to="`/user/${user.id}`">
            <UserIcon aria-hidden="true" />
            User profile
            <ExternalIcon class="h-4 w-4" />
          </nuxt-link>
        </ButtonStyled>
      </div>
    </div>
    <div>
      <div v-for="subscription in subscriptionCharges" :key="subscription.id" class="card">
        <div class="mb-4 grid grid-cols-[1fr_auto]">
          <div>
            <span class="flex items-center gap-2 font-semibold text-contrast">
              <template v-if="subscription.product.metadata.type === 'midas'">
                <ModrinthPlusIcon class="h-7 w-min" />
              </template>
              <template v-else-if="subscription.product.metadata.type === 'pyro'">
                <ModrinthServersIcon class="h-7 w-min" />
              </template>
              <template v-else> Unknown product </template>
            </span>
            <div class="mb-4 mt-2 flex w-full items-center gap-1 text-sm text-secondary">
              {{ formatCategory(subscription.interval) }} ⋅ {{ subscription.status }} ⋅
              {{ dayjs(subscription.created).format("MMMM D, YYYY [at] h:mma") }} ({{
                formatRelativeTime(subscription.created)
              }})
            </div>
          </div>
          <div v-if="subscription.metadata?.id" class="flex flex-col items-end gap-2">
            <ButtonStyled v-if="subscription.product.metadata.type === 'pyro'">
              <nuxt-link
                :to="`/servers/manage/${subscription.metadata.id}`"
                target="_blank"
                class="w-fit"
              >
                <ServerIcon /> Server panel <ExternalIcon class="h-4 w-4" />
              </nuxt-link>
            </ButtonStyled>
            <CopyCode :text="subscription.metadata.id" />
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <div
            v-for="(charge, index) in subscription.charges"
            :key="charge.id"
            class="relative overflow-clip rounded-xl bg-bg px-4 py-3"
          >
            <div
              class="absolute bottom-0 left-0 top-0 w-1"
              :class="charge.type === 'refund' ? 'bg-purple' : chargeStatuses[charge.status].color"
            />
            <div class="grid w-full grid-cols-[1fr_auto] items-center gap-4">
              <div class="flex flex-col gap-2">
                <span>
                  <span class="font-bold text-contrast">
                    <template v-if="charge.status === 'succeeded'"> Succeeded </template>
                    <template v-else-if="charge.status === 'failed'"> Failed </template>
                    <template v-else-if="charge.status === 'cancelled'"> Cancelled </template>
                    <template v-else-if="charge.status === 'processing'"> Processing </template>
                    <template v-else-if="charge.status === 'open'"> Upcoming </template>
                    <template v-else> {{ charge.status }} </template>
                  </span>
                  ⋅
                  <span>
                    <template v-if="charge.type === 'refund'"> Refund </template>
                    <template v-else-if="charge.type === 'subscription'">
                      <template v-if="charge.status === 'cancelled'"> Subscription </template>
                      <template v-else-if="index === subscription.charges.length - 1">
                        Started subscription
                      </template>
                      <template v-else> Subscription renewal </template>
                    </template>
                    <template v-else-if="charge.type === 'one-time'"> One-time charge </template>
                    <template v-else-if="charge.type === 'proration'"> Proration charge </template>
                    <template v-else> {{ charge.status }} </template>
                  </span>
                  <template v-if="charge.status !== 'cancelled'">
                    ⋅
                    {{ formatPrice(vintl.locale, charge.amount, charge.currency_code) }}
                  </template>
                </span>
                <span class="text-sm text-secondary">
                  {{ dayjs(charge.due).format("MMMM D, YYYY [at] h:mma") }}
                  <span class="text-secondary">({{ formatRelativeTime(charge.due) }}) </span>
                </span>
                <div class="flex w-full items-center gap-1 text-xs text-secondary">
                  {{ charge.status }}
                  ⋅
                  {{ charge.type }}
                  ⋅
                  {{ formatPrice(vintl.locale, charge.amount, charge.currency_code) }}
                  ⋅
                  {{ dayjs(charge.due).format("YYYY-MM-DD h:mma") }}
                  <template v-if="charge.subscription_interval">
                    ⋅ {{ charge.subscription_interval }}
                  </template>
                </div>
              </div>
              <div class="flex gap-2">
                <ButtonStyled
                  v-if="
                    charges.some((x) => x.type === 'refund' && x.parent_charge_id === charge.id)
                  "
                >
                  <div class="button-like disabled"><CheckIcon /> Charge refunded</div>
                </ButtonStyled>
                <ButtonStyled
                  v-else-if="charge.status === 'succeeded' && charge.type !== 'refund'"
                  color="red"
                  color-fill="text"
                >
                  <button @click="showRefundModal(charge)">
                    <CurrencyIcon />
                    Refund options
                  </button>
                </ButtonStyled>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup>
import {
  Avatar,
  ButtonStyled,
  CopyCode,
  DropdownSelect,
  NewModal,
  Toggle,
  useRelativeTime,
} from "@modrinth/ui";
import { formatCategory, formatPrice } from "@modrinth/utils";
import {
  CheckIcon,
  XIcon,
  UserIcon,
  ModrinthPlusIcon,
  ServerIcon,
  ExternalIcon,
  CurrencyIcon,
} from "@modrinth/assets";
import dayjs from "dayjs";
import { products } from "~/generated/state.json";
import ModrinthServersIcon from "~/components/ui/servers/ModrinthServersIcon.vue";

const route = useRoute();
const data = useNuxtApp();
const vintl = useVIntl();

const { formatMessage } = vintl;
const formatRelativeTime = useRelativeTime();

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
      charges: charges.value
        .filter((charge) => charge.subscription_id === subscription.id)
        .slice()
        .sort((a, b) => dayjs(b.due).diff(dayjs(a.due))),
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
const refundTypes = ref(["full", "partial", "none"]);
const refundAmount = ref(0);
const unprovision = ref(true);

function showRefundModal(charge) {
  selectedCharge.value = charge;
  refundType.value = "full";
  refundAmount.value = 0;
  unprovision.value = true;
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

const chargeStatuses = {
  open: {
    color: "bg-blue",
  },
  processing: {
    color: "bg-orange",
  },
  succeeded: {
    color: "bg-green",
  },
  failed: {
    color: "bg-red",
  },
  cancelled: {
    color: "bg-red",
  },
};
</script>
<style scoped>
.page {
  padding: 1rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}
</style>
