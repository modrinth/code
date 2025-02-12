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
          <div class="label">Total Pending <nuxt-link v-tooltip="`Click to read about pending revenue.`" class="text-link align-middle" to="/legal/cmp-info#pending"><UnknownIcon /></nuxt-link></div>
          <div class="value">
            {{ $formatMoney(userBalance.pending) }}
          </div>
        </div>
        <div class="grid-display__item">
            <div class="label">Pending In Transit <nuxt-link v-tooltip="`Click to read about pending revenue.`" class="text-link align-middle" to="/legal/cmp-info#pending"><UnknownIcon /></nuxt-link>
          </div>
          <div class="value">
            ≈{{ $formatMoney(totalTwoMonthsAgo) }}
          </div>
          <span>
              <span
              >
                accessible on {{ formatDate(deadlineEnding)}}
              </span
              ></span
          >
        </div>
      </div>

      <div v-if="userBalance.available >= minWithdraw">
        <p>
          You have funds available to withdraw.
        </p>
      </div>
      <p v-else>
        Your current balance is under the minimum of <strong>${{ minWithdraw }}</strong> to withdraw.
      </p>
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
      <p>
        By uploading projects to Modrinth and withdrawing money from your account, you agree to the
        <nuxt-link class="text-link" to="/legal/cmp">Rewards Program Terms</nuxt-link>
        . For more
        information on how the rewards system works, see our information page
        <nuxt-link class="text-link" to="/legal/cmp-info">here</nuxt-link>
        .
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
import { HistoryIcon, PayPalIcon, SaveIcon, TransferIcon, XIcon, UnknownIcon } from '@modrinth/assets'
import { formatDate } from "@modrinth/utils"
import dayjs from 'dayjs'

const auth = await useAuth()
const minWithdraw = ref(0.01)

const { data: userBalance } = await useAsyncData(`payout/balance`, () =>
  useBaseFetch(`payout/balance`, { apiVersion: 3 })
)

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

const { data: pendingInTransit } = await useAsyncData('analytics/revenue', () =>
  useBaseFetch('analytics/revenue', {
    apiVersion: 3,
    query: {
      start_date: dayjs().subtract(2, 'month').startOf('month').toISOString(),
      end_date: dayjs().subtract(2, 'month').endOf('month').toISOString(),
      resolution_minutes: 1140
    }
  })
)

const deadlineEnding = dayjs().subtract(2, 'month').endOf('month').add(60, 'days')
const totalTwoMonthsAgo = dayjs().startOf('day').isAfter(deadlineEnding)
  ? 0
  : Object.values(pendingInTransit.value || {}).reduce((acc, project) => {
    return acc + Object.values(project || {}).reduce((acc, value) => acc + parseFloat(value), 0)
  }, 0)
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
</style>
