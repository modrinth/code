<template>
  <Modal ref="modal" :header="title">
    <div class="modal-delete">
      <div class="markdown-body" v-html="renderString(description)" />
      <label v-if="hasToType" for="confirmation" class="confirmation-label">
        <span>
          <strong>To verify, type</strong>
          <em class="confirmation-text">{{ confirmationText }}</em>
          <strong>below:</strong>
        </span>
      </label>
      <div class="confirmation-input">
        <input
          v-if="hasToType"
          id="confirmation"
          v-model="confirmation_typed"
          type="text"
          placeholder="Type here..."
          @input="type"
        />
      </div>
      <div class="button-group">
        <button class="iconified-button" @click="cancel">
          <CrossIcon />
          Cancel
        </button>
        <button class="iconified-button danger-button" :disabled="action_disabled" @click="proceed">
          <TrashIcon />
          {{ proceedLabel }}
        </button>
      </div>
    </div>
  </Modal>
</template>

<script>
import { renderString } from 'omorphia'
import CrossIcon from '~/assets/images/utils/x.svg?component'
import TrashIcon from '~/assets/images/utils/trash.svg?component'
import Modal from '~/components/ui/Modal.vue'

export default {
  components: {
    CrossIcon,
    TrashIcon,
    Modal,
  },
  props: {
    confirmationText: {
      type: String,
      default: '',
    },
    hasToType: {
      type: Boolean,
      default: false,
    },
    title: {
      type: String,
      default: 'No title defined',
      required: true,
    },
    description: {
      type: String,
      default: 'No description defined',
      required: true,
    },
    proceedLabel: {
      type: String,
      default: 'Proceed',
    },
  },
  emits: ['proceed'],
  data() {
    return {
      action_disabled: this.hasToType,
      confirmation_typed: '',
    }
  },
  methods: {
    renderString,
    cancel() {
      this.$refs.modal.hide()
    },
    proceed() {
      this.$refs.modal.hide()
      this.$emit('proceed')
    },
    type() {
      if (this.hasToType) {
        this.action_disabled =
          this.confirmation_typed.toLowerCase() !== this.confirmationText.toLowerCase()
      }
    },
    show() {
      this.$refs.modal.show()
    },
  },
}
</script>

<style scoped lang="scss">
.modal-delete {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;

  .markdown-body {
    margin-bottom: 1rem;
  }

  .confirmation-label {
    margin-bottom: 0.5rem;
  }

  .confirmation-text {
    padding-right: 0.25ch;
    margin: 0 0.25rem;
  }

  .confirmation-input {
    input {
      width: 20rem;
      max-width: 100%;
    }
  }
}
</style>
