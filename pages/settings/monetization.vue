<template>
  <div>
    <section v-if="enrolled" class="universal-card">
      <h2>Revenue and metrics</h2>
      <p>View your revenue and metrics in the creator dashboard:</p>
      <NuxtLink class="iconified-button" to="/dashboard/revenue">
        <ChartIcon /> Visit creator dashboard
      </NuxtLink>
    </section>
    <section class="universal-card">
      <h2 class="title">Enrollment</h2>
      <template v-if="!enrolled && !auth.user.email">
        <p v-if="!enrolled">
          You are not currently enrolled in Modrinth's Creator Monetization Program. In order to
          enroll, you must first add a valid email address to your account.
        </p>
        <NuxtLink class="iconified-button" to="/settings/account">
          <SettingsIcon /> Visit account settings
        </NuxtLink>
      </template>
      <template v-else-if="editing || !enrolled">
        <p v-if="!enrolled">
          You are not currently enrolled in Modrinth's Creator Monetization Program. Setup a method
          of receiving payments below to enable monetization.
        </p>
        <div class="enroll universal-body">
          <Chips
            v-model="selectedWallet"
            :starting-value="selectedWallet"
            :items="wallets"
            :format-label="$formatWallet"
            @update:model-value="onChangeWallet()"
          />

          <p>
            Enter the information for the
            {{ $formatWallet(selectedWallet) }} account you would like to receive your revenue from
            the Creator Monetization Program:
          </p>
          <div class="input-group">
            <Multiselect
              v-model="accountType"
              :options="getAccountTypes()"
              :custom-label="(value) => formatAccountType(value)"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />

            <label class="hidden" for="account-input"
              >{{ $formatWallet(selectedWallet) }}
              {{ formatAccountType(accountType).toLowerCase() }} input field</label
            >
            <input
              id="account-input"
              v-model="account"
              :placeholder="`Enter your ${$formatWallet(selectedWallet)} ${formatAccountType(
                accountType
              ).toLowerCase()}...`"
              :type="accountType === 'email' ? 'email' : ''"
            />
            <span v-if="accountType === 'phone'"> Format: +18888888888 or +1-888-888-8888 </span>
          </div>
          <div class="input-group">
            <button class="iconified-button brand-button" @click="updatePayoutData(false)">
              <SaveIcon /> Save information
            </button>
            <button
              v-if="enrolled"
              class="iconified-button danger-button"
              @click="updatePayoutData(true)"
            >
              <TrashIcon /> Remove enrollment
            </button>
          </div>
        </div>
      </template>
      <template v-else>
        <p>
          You are currently enrolled in the Creator Monetization Program with a
          {{ $formatWallet(selectedWallet) }} account.
        </p>
        <button class="iconified-button brand-button" @click="editing = true">
          <EditIcon /> Edit information
        </button>
      </template>
    </section>
  </div>
</template>

<script>
import { Multiselect } from 'vue-multiselect'
import Chips from '~/components/ui/Chips.vue'
import SaveIcon from '~/assets/images/utils/save.svg'
import TrashIcon from '~/assets/images/utils/trash.svg'
import EditIcon from '~/assets/images/utils/edit.svg'
import ChartIcon from '~/assets/images/utils/chart.svg'
import SettingsIcon from '~/assets/images/utils/settings.svg'

export default defineNuxtComponent({
  components: {
    Multiselect,
    Chips,
    SaveIcon,
    TrashIcon,
    EditIcon,
    ChartIcon,
    SettingsIcon,
  },
  async setup() {
    definePageMeta({
      middleware: 'auth',
    })
    const auth = await useAuth()
    return { auth }
  },
  data() {
    return {
      editing: false,
      enrolled:
        this.auth.user.payout_data.payout_wallet &&
        this.auth.user.payout_data.payout_wallet_type &&
        this.auth.user.payout_data.payout_address,
      wallets: ['paypal', 'venmo'],
      selectedWallet: this.auth.user.payout_data.payout_wallet ?? 'paypal',
      accountType: this.auth.user.payout_data.payout_wallet_type ?? this.getAccountTypes()[0],
      account: this.auth.user.payout_data.payout_address ?? '',
    }
  },
  head: {
    title: 'Monetization settings - Modrinth',
  },
  methods: {
    getAccountTypes() {
      const types = []
      if (this.selectedWallet === 'venmo') {
        types.push('user_handle')
      }
      types.push('email')
      types.push('phone')
      return types
    },
    formatAccountType(value) {
      switch (value) {
        case 'email':
          return 'Email address'
        case 'phone':
          return 'Phone number'
        case 'user_handle':
          return 'Username'
        default:
          return value.charAt(0).toUpperCase() + value.slice(1)
      }
    },
    onChangeWallet() {
      this.account = ''

      // Set default account type for each wallet
      if (this.selectedWallet === 'paypal') {
        this.accountType = 'email'
      } else if (this.selectedWallet === 'venmo') {
        this.accountType = 'user_handle'
      }
    },
    async updatePayoutData(unenroll) {
      startLoading()
      if (unenroll) {
        this.selectedWallet = 'paypal'
        this.accountType = this.getAccountTypes()[0]
        this.account = ''
      }
      try {
        const data = {
          payout_data: unenroll
            ? null
            : {
                payout_wallet: this.selectedWallet,
                payout_wallet_type: this.accountType,
                payout_address: this.account,
              },
        }

        await useBaseFetch(`user/${this.auth.user.id}`, {
          method: 'PATCH',
          body: data,
          ...this.$defaultHeaders(),
        })
        await useAuth(this.auth.token)

        this.editing = false
        this.enrolled = !unenroll
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data.description,
          type: 'error',
        })
      }
      stopLoading()
    },
  },
})
</script>
<style lang="scss" scoped></style>
