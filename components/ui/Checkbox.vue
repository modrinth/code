<template>
  <div class="checkbox-outer" :class="{ disabled }" @click="toggle">
    <button class="checkbox" :disabled="disabled" :class="{ border }">
      <CheckIcon v-if="value" />
    </button>
    <p>{{ label }}</p>
  </div>
</template>

<script>
import CheckIcon from '~/assets/images/utils/check.svg?inline'

export default {
  name: 'Checkbox',
  components: {
    CheckIcon,
  },
  props: {
    label: {
      type: String,
      default: '',
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    border: {
      type: Boolean,
      default: true,
    },
    value: Boolean,
  },
  methods: {
    toggle() {
      if (!this.disabled) {
        this.$emit('input', !this.value)
      }
    },
  },
}
</script>

<style lang="scss" scoped>
.checkbox-outer {
  display: flex;
  align-items: center;
  cursor: pointer;

  &.disabled {
    opacity: 0.6;
    cursor: not-allowed;

    button {
      cursor: not-allowed;

      &:active,
      &:hover,
      &:focus {
        background-color: var(--color-button-bg);
      }
    }
  }

  p {
    padding: 0.2rem 0rem;
    margin: 0 0 0 0.5rem;
  }
}

.checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;

  width: 1.5rem;
  height: 1.5rem;

  padding: 0;
  margin: 0;

  svg {
    color: var(--color-brand-light);
    stroke-width: 0.2rem;
    height: 1.2rem;
    width: 1.2rem;
    flex-shrink: 0;
  }

  &.border {
    width: 1.2rem;
    height: 1.2rem;

    border: 0.15rem solid var(--color-text);

    svg {
      height: 0.9rem;
      width: 0.9rem;
    }
  }
}
</style>
