<template>
  <Popup :show-popup="display">
    <div class="popup-delete">
      <span class="title">{{ title }}</span>
      <span class="description">
        {{ description }}
      </span>
      <label v-if="hasToType" for="confirmation" class="confirmation-label">
        <span>
          To confirm your action, please type
          <span class="confirmation-text">{{ confirmationText }}</span>
          to continue
        </span>
      </label>
      <input
        v-if="hasToType"
        id="confirmation"
        v-model="confirmation_typed"
        type="text"
        placeholder="Type the input needed to continue"
        @input="type"
      />
      <div class="actions">
        <button class="button" @click="cancel">Cancel</button>
        <button
          class="button warn-button"
          :disabled="action_disabled"
          @click="proceed"
        >
          {{ proceedLabel }}
        </button>
      </div>
    </div>
  </Popup>
</template>

<script>
import Popup from '~/components/ui/Popup'

export default {
  name: 'ConfirmPopup',
  components: {
    Popup,
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
  data() {
    return {
      action_disabled: this.hasToType,
      confirmation_typed: '',
      display: false,
    }
  },
  methods: {
    cancel() {
      this.display = false
    },
    proceed() {
      this.display = false
      this.$emit('proceed')
    },
    type() {
      if (this.hasToType) {
        this.action_disabled =
          this.confirmation_typed.toLowerCase() !==
          this.confirmationText.toLowerCase()
      }
    },
    show() {
      this.display = true
    },
  },
}
</script>

<style scoped lang="scss">
.popup-delete {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;

  @media screen and (min-width: 900px) {
  }

  @media screen and (min-width: 1024px) {
    max-width: 40vw;
  }

  .title {
    font-size: 1.25rem;
    align-self: stretch;
    font-weight: bold;
    text-align: center;
    margin-bottom: 1.5rem;
  }

  .description {
    word-wrap: break-word;
    padding-bottom: 1rem;
  }

  .confirmation-label {
    margin-bottom: 0.5rem;
  }

  .confirmation-text {
    font-weight: bold;
    padding-right: 0;
  }

  .actions {
    display: flex;
    flex-direction: row;
    margin-top: 1.5rem;

    button {
      flex-grow: 1;
      width: 100%;
      margin: 0.75rem 1rem;
      padding: 0.75rem 0;
    }

    .warn-button {
      transition: background-color 1s, color 1s;
      color: var(--color-brand-inverted);
      background-color: var(--color-badge-red-bg);

      &:disabled {
        background-color: var(--color-button-bg);
        color: var(--color-button-text-disabled);
      }
    }
  }
}
</style>
