<template>
  <section class="universal-card">
    <Breadcrumbs
      current-title="Withdraw"
      :link-stack="[{ href: '/dashboard/revenue', label: 'Revenue' }]"
    />

    <h2>Withdraw</h2>

    <h3>Region</h3>
    <Multiselect
      id="country-multiselect"
      v-model="country"
      class="country-multiselect"
      placeholder="Select country..."
      track-by="id"
      label="name"
      :options="countries"
      :searchable="true"
      :close-on-select="true"
      :show-labels="false"
      :allow-empty="false"
    />

    <h3>Withdraw method</h3>

    <div class="iconified-input">
      <label class="hidden" for="search">Search</label>
      <SearchIcon aria-hidden="true" />
      <input
        id="search"
        v-model="search"
        name="search"
        placeholder="Search options..."
        autocomplete="off"
      />
    </div>
    <div class="withdraw-options-scroll">
      <div class="withdraw-options">
        <button
          v-for="method in payoutMethods.filter((x) =>
            x.name.toLowerCase().includes(search.toLowerCase())
          )"
          :key="method.id"
          class="withdraw-option button-base"
          :class="{ selected: selectedMethodId === method.id }"
          @click="() => (selectedMethodId = method.id)"
        >
          <div class="preview" :class="{ 'show-bg': !method.image_url || method.name === 'ACH' }">
            <template v-if="method.image_url && method.name !== 'ACH'">
              <div class="preview-badges">
                <span class="badge">
                  {{
                    getRangeOfMethod(method)
                      .map($formatMoney)
                      .map((i) => i.replace('.00', ''))
                      .join('–')
                  }}
                </span>
              </div>
              <img
                v-if="method.image_url && method.name !== 'ACH'"
                class="preview-img"
                :src="method.image_url"
                :alt="method.name"
              />
            </template>
            <div v-else class="placeholder">
              <template v-if="method.type === 'venmo'">
                <VenmoIcon class="enlarge" />
              </template>
              <template v-else>
                <PayPalIcon v-if="method.type === 'paypal'" />
                <span>{{ method.name }}</span>
              </template>
            </div>
          </div>
          <div class="label">
            <RadioButtonChecked v-if="selectedMethodId === method.id" class="radio" />
            <RadioButtonIcon v-else class="radio" />
            <span>{{ method.name }}</span>
          </div>
        </button>
      </div>
    </div>

    <h3>Amount</h3>
    <p>
      You are initiating a transfer of your revenue from Modrinth's Creator Monetization Program.
      How much of your
      <strong>{{ $formatMoney(auth.user.payout_data.balance) }}</strong> balance would you like to
      transfer to {{ selectedMethod.name }}?
    </p>
    <div class="confirmation-input">
      <template v-if="selectedMethod.interval.fixed">
        <Chips
          v-model="amount"
          :items="selectedMethod.interval.fixed.values"
          :format-label="(val) => '$' + val"
        />
      </template>
      <template v-else-if="minWithdrawAmount == maxWithdrawAmount">
        <div>
          <p>
            This method has a fixed transfer amount of
            <strong>{{ $formatMoney(minWithdrawAmount) }}</strong
            >.
          </p>
        </div>
      </template>
      <template v-else>
        <div>
          <p>
            This method has a minimum transfer amount of
            <strong>{{ $formatMoney(minWithdrawAmount) }}</strong> and a maximum transfer amount of
            <strong>{{ $formatMoney(maxWithdrawAmount) }}</strong
            >.
          </p>
          <input
            id="confirmation"
            v-model="amount"
            type="text"
            pattern="^\d*(\.\d{0,2})?$"
            autocomplete="off"
            placeholder="Amount to transfer..."
          />
          <p>
            You have entered <strong>{{ $formatMoney(parsedAmount) }}</strong> to transfer.
          </p>
        </div>
      </template>
    </div>

    <div class="confirm-text">
      <template v-if="knownErrors.length === 0 && amount">
        <Checkbox v-if="fees > 0" v-model="agreedFees" description="Consent to fee">
          I acknowledge that an estimated
          {{ $formatMoney(fees) }} will be deducted from the amount I receive to cover
          {{ $formatWallet(selectedMethod.type) }} processing fees.
        </Checkbox>
        <Checkbox v-model="agreedTransfer" description="Confirm transfer">
          <template v-if="selectedMethod.type === 'tremendous'">
            I confirm that I am initiating a transfer and I will receive further instructions on how
            to redeem this payment via email to: {{ withdrawAccount }}
          </template>
          <template v-else>
            I confirm that I am initiating a transfer to the following
            {{ $formatWallet(selectedMethod.type) }} account: {{ withdrawAccount }}
          </template>
        </Checkbox>
        <Checkbox v-model="agreedTerms" class="rewards-checkbox">
          I agree to the
          <nuxt-link to="/legal/cmp" class="text-link">Rewards Program Terms</nuxt-link>
        </Checkbox>
      </template>
      <template v-else>
        <span v-for="(error, index) in knownErrors" :key="index" class="invalid">
          {{ error }}
        </span>
      </template>
    </div>
    <div class="button-group">
      <nuxt-link to="/dashboard/revenue" class="iconified-button">
        <XIcon />
        Cancel
      </nuxt-link>
      <button
        :disabled="
          knownErrors.length > 0 ||
          !amount ||
          !agreedTransfer ||
          !agreedTerms ||
          (fees > 0 && !agreedFees)
        "
        class="iconified-button brand-button"
        @click="withdraw"
      >
        <TransferIcon />
        Withdraw
      </button>
    </div>
  </section>
</template>

<script setup>
import { Multiselect } from 'vue-multiselect'
import {
  PayPalIcon,
  SearchIcon,
  RadioButtonIcon,
  RadioButtonChecked,
  Chips,
  XIcon,
  TransferIcon,
  Checkbox,
  Breadcrumbs,
} from 'omorphia'
import { all } from 'iso-3166-1'
import VenmoIcon from '~/assets/images/external/venmo.svg?component'

const auth = await useAuth()
const data = useNuxtApp()

const countries = computed(() =>
  all().map((x) => ({
    id: x.alpha2,
    name: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
  }))
)
const search = ref('')

const amount = ref('')
const country = ref(
  countries.value.find((x) => x.id === (auth.value.user.payout_data.paypal_region ?? 'US'))
)

const { data: payoutMethods, refresh: refreshPayoutMethods } = await useAsyncData(
  `payout/methods?country=${country.value.id}`,
  () => useBaseFetch(`payout/methods?country=${country.value.id}`, { apiVersion: 3 })
)

const selectedMethodId = ref(payoutMethods.value[0].id)
const selectedMethod = computed(() =>
  payoutMethods.value.find((x) => x.id === selectedMethodId.value)
)

const parsedAmount = computed(() => {
  const regex = /^\$?(\d*(\.\d{2})?)$/gm
  const matches = regex.exec(amount.value)
  return matches && matches[1] ? parseFloat(matches[1]) : 0.0
})
const fees = computed(() => {
  return Math.min(
    Math.max(
      selectedMethod.value.fee.min,
      selectedMethod.value.fee.percentage * parsedAmount.value
    ),
    selectedMethod.value.fee.max ?? Number.MAX_VALUE
  )
})

const getIntervalRange = (intervalType) => {
  if (!intervalType) {
    return []
  }

  const { min, max, values } = intervalType
  if (values) {
    const first = values[0]
    const last = values.slice(-1)[0]
    return first === last ? [first] : [first, last]
  }

  return min === max ? [min] : [min, max]
}

const getRangeOfMethod = (method) => {
  return getIntervalRange(method.interval?.fixed || method.interval?.standard)
}

const maxWithdrawAmount = computed(() => {
  const interval = selectedMethod.value.interval
  return interval?.standard ? interval.standard.max : interval?.fixed?.values.slice(-1)[0] ?? 0
})

const minWithdrawAmount = computed(() => {
  const interval = selectedMethod.value.interval
  return interval?.standard ? interval.standard.min : interval?.fixed?.values?.[0] ?? fees.value
})

const withdrawAccount = computed(() => {
  if (selectedMethod.value.type === 'paypal') {
    return auth.value.user.payout_data.paypal_address
  } else if (selectedMethod.value.type === 'venmo') {
    return auth.value.user.payout_data.venmo_handle
  } else {
    return auth.value.user.email
  }
})
const knownErrors = computed(() => {
  const errors = []
  if (selectedMethod.value.type === 'paypal' && !auth.value.user.payout_data.paypal_address) {
    errors.push('Please link your PayPal account in the dashboard to proceed.')
  }
  if (selectedMethod.value.type === 'venmo' && !auth.value.user.payout_data.venmo_handle) {
    errors.push('Please set your Venmo handle in the dashboard to proceed.')
  }
  if (selectedMethod.value.type === 'tremendous') {
    if (!auth.value.user.email) {
      errors.push('Please set your email address in your account settings to proceed.')
    }
    if (!auth.value.user.email_verified) {
      errors.push('Please verify your email address to proceed.')
    }
  }

  if (!parsedAmount.value && amount.value.length > 0) {
    errors.push(`${amount.value} is not a valid amount`)
  } else if (
    parsedAmount.value > auth.value.user.payout_data.balance ||
    parsedAmount.value > maxWithdrawAmount.value
  ) {
    const maxAmount = Math.min(auth.value.user.payout_data.balance, maxWithdrawAmount.value)
    errors.push(`The amount must be no more than ${data.$formatMoney(maxAmount)}`)
  } else if (parsedAmount.value <= fees.value || parsedAmount.value < minWithdrawAmount.value) {
    const minAmount = Math.max(fees.value + 0.01, minWithdrawAmount.value)
    errors.push(`The amount must be at least ${data.$formatMoney(minAmount)}`)
  }

  return errors
})

const agreedTransfer = ref(false)
const agreedFees = ref(false)
const agreedTerms = ref(false)

watch(country, async () => {
  await refreshPayoutMethods()
  if (payoutMethods.value && payoutMethods.value[0]) {
    selectedMethodId.value = payoutMethods.value[0].id
  }
})

watch(selectedMethod, () => {
  if (selectedMethod.value.interval?.fixed) {
    amount.value = selectedMethod.value.interval.fixed.values[0]
  }
  if (maxWithdrawAmount.value === minWithdrawAmount.value) {
    amount.value = maxWithdrawAmount.value
  }
  agreedTransfer.value = false
  agreedFees.value = false
  agreedTerms.value = false
})

async function withdraw() {
  startLoading()
  try {
    const auth = await useAuth()

    await useBaseFetch(`payout`, {
      method: 'POST',
      body: {
        amount: parsedAmount.value,
        method: selectedMethod.value.type,
        method_id: selectedMethod.value.id,
      },
      apiVersion: 3,
    })
    await useAuth(auth.value.token)
    await navigateTo('/dashboard/revenue')
    data.$notify({
      group: 'main',
      title: 'Withdrawal complete',
      text:
        selectedMethod.value.type === 'tremendous'
          ? 'An email has been sent to your account with further instructions on how to redeem your payout!'
          : `Payment has been sent to your ${data.$formatWallet(
              selectedMethod.value.type
            )} account!`,
      type: 'success',
    })
  } catch (err) {
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
.withdraw-options-scroll {
  max-height: 460px;
  overflow-y: auto;

  &::-webkit-scrollbar {
    width: var(--gap-md);
    border: 3px solid var(--color-bg);
  }

  &::-webkit-scrollbar-track {
    background: var(--color-bg);
    border: 3px solid var(--color-bg);
  }

  &::-webkit-scrollbar-thumb {
    background-color: var(--color-raised-bg);
    border-radius: var(--radius-lg);
    border: 3px solid var(--color-bg);
  }
}

.withdraw-options {
  display: grid;
  grid-template-columns: repeat(1, 1fr);
  gap: var(--gap-lg);
  padding-right: 0.5rem;

  @media screen and (min-width: 300px) {
    grid-template-columns: repeat(2, 1fr);
  }

  @media screen and (min-width: 600px) {
    grid-template-columns: repeat(3, 1fr);
  }
}

.withdraw-option {
  width: 100%;
  border-radius: var(--radius-md);
  padding: 0;
  overflow: hidden;
  border: 1px solid var(--color-divider);
  background-color: var(--color-button-bg);
  color: var(--color-text);

  &.selected {
    color: var(--color-contrast);

    .label svg {
      color: var(--color-brand);
    }
  }

  .preview {
    display: flex;
    justify-content: center;
    aspect-ratio: 30 / 19;
    position: relative;

    .preview-badges {
      // These will float over the image in the bottom right corner
      position: absolute;
      bottom: 0;
      right: 0;
      padding: var(--gap-sm) var(--gap-xs);

      .badge {
        background-color: var(--color-button-bg);
        border-radius: var(--radius-xs);
        padding: var(--gap-xs) var(--gap-sm);
        font-size: var(--font-size-xs);
      }
    }

    &.show-bg {
      background-color: var(--color-bg);
    }

    img {
      -webkit-user-drag: none;
      -khtml-user-drag: none;
      -moz-user-drag: none;
      -o-user-drag: none;
      user-drag: none;
      user-select: none;
      width: 100%;
      height: auto;
      object-fit: cover;
    }

    .placeholder {
      display: flex;
      align-items: center;
      gap: var(--gap-xs);

      svg {
        width: 2rem;
        height: auto;
      }

      span {
        font-weight: var(--font-weight-bold);
        font-size: 2rem;
        font-style: italic;
      }

      .enlarge {
        width: auto;
        height: 1.5rem;
      }
    }
  }

  .label {
    display: flex;
    align-items: center;
    padding: var(--gap-md) var(--gap-lg);

    svg {
      min-height: 1rem;
      min-width: 1rem;
      margin-right: 0.5rem;
    }

    span {
      text-overflow: ellipsis;
      overflow: hidden;
      white-space: nowrap;
    }
  }
}

.invalid {
  color: var(--color-red);
}

.confirm-text {
  margin: var(--spacing-card-md) 0;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
}

.iconified-input {
  margin-bottom: var(--spacing-card-md);
}

.country-multiselect,
.iconified-input {
  max-width: 16rem;
}

.rewards-checkbox {
  a {
    margin-left: 0.5ch;
  }
}
</style>
