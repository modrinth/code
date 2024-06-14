<template>
  <div
    class="checkbox-outer button-within"
    :class="{ disabled, checked: modelValue }"
    role="presentation"
    @click="toggle"
  >
    <button
      class="checkbox"
      role="checkbox"
      :disabled="disabled"
      :class="{ checked: modelValue, collapsing: collapsingToggleStyle }"
      :aria-label="description ?? label"
      :aria-checked="modelValue"
    >
      <CheckIcon v-if="modelValue && !collapsingToggleStyle" aria-hidden="true" />
      <DropdownIcon v-else-if="collapsingToggleStyle" aria-hidden="true" />
    </button>
    <!-- aria-hidden is set so screenreaders only use the <button>'s aria-label -->
    <p v-if="label" aria-hidden="true">
      {{ label }}
    </p>
    <slot v-else />
  </div>
</template>

<script>
import CheckIcon from '~/assets/images/utils/check.svg?component'
import DropdownIcon from '~/assets/images/utils/dropdown.svg?component'

export default {
  components: {
    CheckIcon,
    DropdownIcon,
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
    description: {
      type: String,
      default: null,
    },
    modelValue: Boolean,
    clickEvent: {
      type: Function,
      default: () => {},
    },
    collapsingToggleStyle: {
      type: Boolean,
      default: false,
    },
  },
  emits: ['update:modelValue'],
  methods: {
    toggle() {
      if (!this.disabled) {
        this.$emit('update:modelValue', !this.modelValue)
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

  p {
    user-select: none;
    padding: 0.2rem 0;
    margin: 0;
  }

  &.disabled {
    cursor: not-allowed;
  }

  &.checked {
    outline: 2px solid transparent;
    outline-offset: 4px;
    border-radius: 0.25rem;
  }
}

.checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;

  min-width: 1rem;
  min-height: 1rem;

  padding: 0;
  margin: 0 0.5rem 0 0;

  color: var(--color-button-text);
  background-color: var(--color-button-bg);
  border-radius: var(--size-rounded-control);
  box-shadow: var(--shadow-inset-sm), 0 0 0 0 transparent;

  &.checked {
    background-color: var(--color-brand);
  }

  svg {
    color: var(--color-brand-inverted);
    stroke-width: 0.2rem;
    height: 0.8rem;
    width: 0.8rem;
    flex-shrink: 0;
  }

  &.collapsing {
    background-color: transparent !important;
    box-shadow: none;

    svg {
      color: inherit;
      height: 1rem;
      width: 1rem;
      transition: transform 0.25s ease-in-out;
    }

    &.checked {
      svg {
        transform: rotate(180deg);
      }
    }
  }

  &:disabled {
    box-shadow: none;
    cursor: not-allowed;
  }
}
</style>
