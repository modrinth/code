<template>
  <div>
    <section class="universal-card">
      <h2 class="text-2xl">Revenue</h2>
      <div class="grid-display">
        <div class="grid-display__item">
          <div class="label">Current Balance</div>
          <div class="value">
            {{ $formatMoney(userBalance.available) }}
          </div>
        </div>
        <div class="grid-display__item">
          <div class="label">Total Pending <nuxt-link v-tooltip="`Click to read about how Modrinth handles your revenue.`" class="text-link align-middle" to="/legal/cmp-info#pending"><UnknownIcon /></nuxt-link></div>
          <div class="value">
            {{ $formatMoney(userBalance.pending) }}
          </div>
        </div>
        <div class="grid-display__item available-soon">
          <h3 class="label">Available Soon <nuxt-link v-tooltip="`Click to read about how Modrinth handles your revenue.`" class="text-link align-middle" to="/legal/cmp-info#pending"><UnknownIcon /></nuxt-link></h3>
          <ul class="available-soon-list">
            <li v-for="date in Object.keys(availableSoonDates)" :key="date" class="available-soon-item">
              <span class="amount">{{ $formatMoney(availableSoonDates[date]) }}</span>
              <span class="date">{{ formatDate(dayjs(date)) }}</span>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="userBalance.available >= minWithdraw">
        <p>
          You have funds available to withdraw.
        </p>
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
      <p v-if="!(userBalance.available >= minWithdraw)">
        Your current balance is under the minimum of <strong>${{ minWithdraw }}</strong> to withdraw.
      </p>
      <p>
       <small>By uploading projects to Modrinth and withdrawing money from your account, you agree to the
         <nuxt-link class="text-link" to="/legal/cmp">Rewards Program Terms</nuxt-link>.
         For more information on how the rewards system works, see our information page
         <nuxt-link class="text-link" to="/legal/cmp-info">here</nuxt-link>.</small>
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
  PayPalIcon,
  SaveIcon,
  TransferIcon,
  XIcon,
  UnknownIcon,
  DownloadIcon
} from '@modrinth/assets'
import { formatDate, formatMoney } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'
import { Button } from '@modrinth/ui'

const auth = await useAuth()
const minWithdraw = ref(0.01)

const { data: userBalance } = await useAsyncData(`payout/balance`, () =>
  useBaseFetch(`payout/balance`, { apiVersion: 3 })
)

const deadlineEnding = computed(() => {
  let deadline = dayjs().subtract(2, 'month').endOf('month').add(60, 'days')
  if (deadline.isBefore(dayjs().startOf('day'))) {
    deadline = dayjs().subtract(1, 'month').endOf('month').add(60, 'days')
  }
  return deadline
});

const availableSoonDates = computed(() => {
  // Get the next 3 dates from userBalance.dates that are from now to the deadline + 4 months to make sure we get all the pending ones.
  const dates = Object.keys(userBalance.value.dates).filter(date => {
    const dateObj = dayjs(date)
    return dateObj.isAfter(dayjs()) && dateObj.isBefore(dayjs(deadlineEnding.value).add(4, 'month'))
  }).sort((a, b) => dayjs(a).diff(dayjs(b)));

  return dates.reduce((acc, date) => {
    acc[date] = userBalance.value.dates[date]
    return acc
  }, {})
});

async function updateVenmo() {
  startLoading()
  try {
    const data = {
      venmo_handle: auth.value.user.payout_data.venmo_handle ?? null
    }

    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: 'PATCH',
      body: data,
      apiVersion: 3
    })
    await useAuth(auth.value.token)
  } catch (err) {
    const data = useNuxtApp()
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error'
    })
  }
  stopLoading()
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

.grid-display__item {
  .value {
    flex-grow: 1;
    display: flex;
    align-items: center;
  }
}

.available-soon {
  padding-top: 0;

  .label {
    margin: 0;
  }

  &-list {
    list-style-type: none;
    padding: 0;
    margin: 0;
  }

  &-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.2rem 0 0;
    border-bottom: 1px solid #eee;

    .amount {
      font-weight: 600;
    }

    .date {
      color: #7f8c8d;
      font-size: 0.9em;
    }

    &:last-child {
      border-bottom: none;
    }
  }
}
</style>
