<template>
  <div class="experimental-styles-within">
    <section class="universal-card">
      <h2 class="text-2xl">Revenue</h2>
      <div class="grid-display">
        <div class="grid-display__item">
          <div class="label">Available now</div>
          <div class="value">
            {{ $formatMoney(userBalance.available) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">
            Total pending
            <nuxt-link
              v-tooltip="`Click to read about how Modrinth handles your revenue.`"
              class="align-middle text-link"
              to="/legal/cmp-info#pending"
            >
              <UnknownIcon />
            </nuxt-link>
          </div>
          <div class="value">
            {{ $formatMoney(userBalance.pending) }}
          </div>
        </div>
        <div class="grid-display__item">
          <h3 class="label m-0">
            Available soon
            <nuxt-link
              v-tooltip="`Click to read about how Modrinth handles your revenue.`"
              class="align-middle text-link"
              to="/legal/cmp-info#pending"
            >
              <UnknownIcon />
            </nuxt-link>
          </h3>
          <ul class="m-0 list-none p-0">
            <li
              v-for="date in availableSoonDateKeys"
              :key="date"
              class="flex items-center justify-between border-0 border-solid border-b-divider p-0 [&:not(:last-child)]:mb-1 [&:not(:last-child)]:border-b-[1px] [&:not(:last-child)]:pb-1"
            >
              <span
                v-tooltip="
                  availableSoonDateKeys.indexOf(date) === availableSoonDateKeys.length - 1
                    ? `Revenue period is ongoing. \nThis amount is not yet finalized.`
                    : null
                "
                :class="{
                  'cursor-help':
                    availableSoonDateKeys.indexOf(date) === availableSoonDateKeys.length - 1,
                }"
                class="inline-flex items-center gap-1 font-bold"
              >
                {{ $formatMoney(availableSoonDates[date]) }}
                <template
                  v-if="availableSoonDateKeys.indexOf(date) === availableSoonDateKeys.length - 1"
                >
                  <InProgressIcon />
                </template>
              </span>
              <span class="text-sm text-secondary">
                {{ formatDate(dayjs(date)) }}
              </span>
            </li>
          </ul>
        </div>
      </div>
      <div class="input-group mt-4">
        <span :class="{ 'disabled-cursor-wrapper': userBalance.available < minWithdraw }">
          <nuxt-link
            :aria-disabled="userBalance.available < minWithdraw ? 'true' : 'false'"
            :class="{ 'disabled-link': userBalance.available < minWithdraw }"
            :disabled="userBalance.available < minWithdraw ? 'true' : 'false'"
            :tabindex="userBalance.available < minWithdraw ? -1 : undefined"
            class="iconified-button brand-button"
            to="/dashboard/revenue/withdraw"
          >
            <TransferIcon /> Withdraw
          </nuxt-link>
        </span>
        <NuxtLink class="iconified-button" to="/dashboard/revenue/transfers">
          <HistoryIcon />
          View transfer history
        </NuxtLink>
      </div>
      <p class="text-sm text-secondary">
        By uploading projects to Modrinth and withdrawing money from your account, you agree to the
        <nuxt-link class="text-link" to="/legal/cmp">Rewards Program Terms</nuxt-link>. For more
        information on how the rewards system works, see our information page
        <nuxt-link class="text-link" to="/legal/cmp-info">here</nuxt-link>.
      </p>
    </section>
    <section class="universal-card">
      <h2 class="text-2xl">Payout methods</h2>
      <h3>PayPal</h3>
      <template v-if="auth.user.auth_providers.includes('paypal')">
        <p>
          Your PayPal {{ auth.user.payout_data.paypal_country }} account is currently connected with
          email
          {{ auth.user.payout_data.paypal_address }}
        </p>
        <button class="btn mt-4" @click="removeAuthProvider('paypal')">
          <XIcon />
          Disconnect account
        </button>
      </template>
      <template v-else>
        <p>Connect your PayPal account to enable withdrawing to your PayPal balance.</p>
        <a :href="`${getAuthUrl('paypal')}&token=${auth.token}`" class="btn mt-4">
          <PayPalIcon />
          Sign in with PayPal
        </a>
      </template>
      <h3>Tremendous</h3>
      <p>
        Tremendous payments are sent to your Modrinth email. To change/set your Modrinth email,
        visit
        <nuxt-link class="text-link" to="/settings/account">here</nuxt-link>
        .
      </p>
      <h3>Venmo</h3>
      <p>Enter your Venmo username below to enable withdrawing to your Venmo balance.</p>
      <label class="hidden" for="venmo">Venmo address</label>
      <input
        id="venmo"
        v-model="auth.user.payout_data.venmo_handle"
        autocomplete="off"
        class="mt-4"
        name="search"
        placeholder="@example"
        type="search"
      />
      <button class="btn btn-secondary" @click="updateVenmo">
        <SaveIcon />
        Save information
      </button>
    </section>
  </div>
</template>
<script setup>
import {
  HistoryIcon,
  InProgressIcon,
  PayPalIcon,
  SaveIcon,
  TransferIcon,
  UnknownIcon,
  XIcon,
} from "@modrinth/assets";
import { formatDate } from "@modrinth/utils";
import dayjs from "dayjs";
import { computed } from "vue";

const auth = await useAuth();
const minWithdraw = ref(0.01);

const { data: userBalance } = await useAsyncData(`payout/balance`, () =>
  useBaseFetch(`payout/balance`, { apiVersion: 3 }),
);

const deadlineEnding = computed(() => {
  let deadline = dayjs().subtract(2, "month").endOf("month").add(60, "days");
  if (deadline.isBefore(dayjs().startOf("day"))) {
    deadline = dayjs().subtract(1, "month").endOf("month").add(60, "days");
  }
  return deadline;
});

const availableSoonDates = computed(() => {
  // Get the next 3 dates from userBalance.dates that are from now to the deadline + 4 months to make sure we get all the pending ones.
  const dates = Object.keys(userBalance.value.dates)
    .filter((date) => {
      const dateObj = dayjs(date);
      return (
        dateObj.isAfter(dayjs()) && dateObj.isBefore(dayjs(deadlineEnding.value).add(4, "month"))
      );
    })
    .sort((a, b) => dayjs(a).diff(dayjs(b)));

  return dates.reduce((acc, date) => {
    acc[date] = userBalance.value.dates[date];
    return acc;
  }, {});
});

const availableSoonDateKeys = computed(() => Object.keys(availableSoonDates.value));

async function updateVenmo() {
  startLoading();
  try {
    const data = {
      venmo_handle: auth.value.user.payout_data.venmo_handle ?? null,
    };

    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: "PATCH",
      body: data,
      apiVersion: 3,
    });
    await useAuth(auth.value.token);
  } catch (err) {
    const data = useNuxtApp();
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
<style lang="scss" scoped>
strong {
  color: var(--color-text-dark);
  font-weight: 500;
}

.disabled-cursor-wrapper {
  cursor: not-allowed;
}

.disabled-link {
  pointer-events: none;
}

.grid-display {
  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
}
</style>
