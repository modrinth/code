<template>
  <div>
    <section class="universal-card">
      <h2>Revenue</h2>
      <div v-if="auth.user.payout_data.balance >= minWithdraw">
        <p>
          You have
          <strong>{{ $formatMoney(auth.user.payout_data.balance) }}</strong>
          available to withdraw.
        </p>
      </div>
      <p v-else>
        You have made
        <strong>{{ $formatMoney(auth.user.payout_data.balance) }}</strong
        >, which is under the minimum of ${{ minWithdraw }} to withdraw.
      </p>
      <div class="input-group">
        <nuxt-link
          v-if="auth.user.payout_data.balance >= minWithdraw"
          class="iconified-button brand-button"
          to="/dashboard/revenue/withdraw"
        >
          <TransferIcon /> Withdraw
        </nuxt-link>
        <NuxtLink class="iconified-button" to="/dashboard/revenue/transfers">
          <HistoryIcon /> View transfer history
        </NuxtLink>
      </div>
      <p>
        By uploading projects to Modrinth and withdrawing money from your account, you agree to the
        <nuxt-link to="/legal/cmp" class="text-link">Rewards Program Terms</nuxt-link>. For more
        information on how the rewards system works, see our information page
        <nuxt-link to="/legal/cmp-info" class="text-link">here</nuxt-link>.
      </p>
    </section>
    <section class="universal-card">
      <h2>Payout methods</h2>
      <h3>PayPal</h3>
      <template v-if="auth.user.auth_providers.includes('paypal')">
        <p>
          Your PayPal {{ auth.user.payout_data.paypal_country }} account is currently connected with
          email
          {{ auth.user.payout_data.paypal_address }}
        </p>
        <button class="btn" @click="removeAuthProvider('paypal')">
          <XIcon /> Disconnect account
        </button>
      </template>
      <template v-else>
        <p>Connect your PayPal account to enable withdrawing to your PayPal balance.</p>
        <a class="btn" :href="`${getAuthUrl('paypal')}&token=${auth.token}`">
          <PayPalIcon />
          Sign in with PayPal
        </a>
      </template>
      <h3>Tremendous</h3>
      <p>
        Tremendous payments are sent to your Modrinth email. To change/set your Modrinth email,
        visit
        <nuxt-link to="/settings/account" class="text-link">here</nuxt-link>.
      </p>
      <h3>Venmo</h3>
      <p>Enter your Venmo username below to enable withdrawing to your Venmo balance.</p>
      <label class="hidden" for="venmo">Venmo address</label>
      <input
        id="venmo"
        v-model="auth.user.payout_data.venmo_handle"
        type="search"
        name="search"
        placeholder="@example"
        autocomplete="off"
      />
      <button class="btn btn-secondary" @click="updateVenmo"><SaveIcon /> Save information</button>
    </section>
  </div>
</template>
<script setup>
import { TransferIcon, HistoryIcon, PayPalIcon, SaveIcon, XIcon } from 'omorphia'

const auth = await useAuth()
const minWithdraw = ref(0.01)

async function updateVenmo() {
  startLoading()
  try {
    const data = {
      venmo_handle: auth.value.user.payout_data.venmo_handle ?? null,
    }

    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: 'PATCH',
      body: data,
      apiVersion: 3,
    })
    await useAuth(auth.value.token)
  } catch (err) {
    const data = useNuxtApp()
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
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
</style>
