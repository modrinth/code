<template>
  <Modal ref="modal" :header="'Transfer to ' + $formatWallet(wallet)">
    <div class="modal-transfer">
      <span
        >You are initiating a transfer of your revenue from Modrinth's Creator Monetization Program.
        How much of your <strong>{{ $formatMoney(balance) }}</strong> balance would you like to
        transfer?</span
      >
      <div class="confirmation-input">
        <input
          id="confirmation"
          v-model="amount"
          type="text"
          pattern="^\d*(\.\d{0,2})?$"
          autocomplete="off"
          placeholder="Amount to transfer..."
        />
      </div>
      <div class="confirm-text">
        <Checkbox
          v-if="isValidInput() && parseInput() >= minWithdraw && parseInput() <= balance"
          v-model="consentedFee"
          description="Consent to fee"
        >
          <template v-if="wallet === 'venmo'">
            I acknowledge that $0.25 will be deducted from the amount I receive to cover
            {{ $formatWallet(wallet) }} processing fees.
          </template>
          <template v-else>
            I acknowledge that an estimated
            {{ $formatMoney(calcProcessingFees()) }} will be deducted from the amount I receive to
            cover {{ $formatWallet(wallet) }} processing fees and that any excess will be returned
            to my Modrinth balance.
          </template>
        </Checkbox>
        <Checkbox
          v-if="isValidInput() && parseInput() >= minWithdraw && parseInput() <= balance"
          v-model="consentedAccount"
          description="Confirm transfer"
        >
          I confirm that I an initiating a transfer to the following
          {{ $formatWallet(wallet) }} account: {{ account }}
        </Checkbox>
        <span v-else-if="validInput && parseInput() < minWithdraw" class="invalid">
          The amount must be at least {{ $formatMoney(minWithdraw) }}</span
        >
        <span v-else-if="validInput && parseInput() > balance" class="invalid">
          The amount must be no more than {{ $formatMoney(balance) }}</span
        >
        <span v-else-if="amount.length > 0" class="invalid">
          {{ amount }} is not a valid amount</span
        >
      </div>
      <div class="button-group">
        <NuxtLink class="iconified-button" to="/settings/monetization">
          <SettingsIcon /> Monetization settings
        </NuxtLink>
        <button class="iconified-button" @click="cancel">
          <CrossIcon />
          Cancel
        </button>
        <button
          class="iconified-button brand-button"
          :disabled="!consentedFee || !consentedAccount"
          @click="proceed"
        >
          <TransferIcon />
          Transfer
        </button>
      </div>
    </div>
  </Modal>
</template>

<script>
import CrossIcon from '~/assets/images/utils/x.svg'
import TransferIcon from '~/assets/images/utils/transfer.svg'
import SettingsIcon from '~/assets/images/utils/settings.svg'
import Modal from '~/components/ui/Modal.vue'
import Checkbox from '~/components/ui/Checkbox.vue'

export default {
  components: {
    Checkbox,
    CrossIcon,
    SettingsIcon,
    TransferIcon,
    Modal,
  },
  props: {
    wallet: {
      type: String,
      required: true,
    },
    accountType: {
      type: String,
      required: true,
    },
    account: {
      type: String,
      required: true,
    },
    balance: {
      type: Number,
      required: true,
    },
    minWithdraw: {
      type: Number,
      required: true,
    },
  },
  data() {
    return {
      consentedFee: false,
      consentedAccount: false,
      amount: '',
      validInput: false,
    }
  },
  methods: {
    cancel() {
      this.amount = ''
      this.consentedFee = false
      this.consentedAccount = false
      this.validInput = false
      this.$refs.modal.hide()
    },
    async proceed() {
      startLoading()
      try {
        await useBaseFetch(`user/${this.$auth.user.id}/payouts`, {
          method: 'POST',
          body: {
            amount: Number(this.amount.replace('$', '')),
          },
          ...this.$defaultHeaders(),
        })
        await useAuth(this.$auth.token)

        this.$refs.modal.hide()
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
    show() {
      this.$refs.modal.show()
    },
    isValidInput() {
      const regex = /^\$?(\d*(\.\d{2})?)$/gm
      this.validInput = regex.test(this.amount) && this.amount.length > 0
      return this.validInput
    },
    parseInput() {
      const regex = /^\$?(\d*(\.\d{2})?)$/gm
      const matches = regex.exec(this.amount)
      return parseFloat(matches[1])
    },
    calcProcessingFees() {
      if (this.wallet === 'venmo') {
        return 0.25
      } else {
        return Math.max(0.25, Math.min(this.parseInput() * 0.02, 20))
      }
    },
  },
}
</script>

<style scoped lang="scss">
.modal-transfer {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);

  .confirmation-input {
    input {
      width: 14rem;
      max-width: 100%;
    }
  }

  .button-group {
    margin-top: var(--spacing-card-md);
    justify-content: right;
  }

  strong {
    color: var(--color-text-dark);
    font-weight: 500;
  }

  .invalid {
    color: var(--color-special-red);
  }

  .confirm-text {
    margin-top: var(--spacing-card-sm);
    min-height: 7rem;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-card-sm);
  }
}
</style>
