<template>
  <div
    class="checkbox-outer button-within"
    :class="{ disabled }"
    role="presentation"
    @click="toggle"
  >
    <button
      class="checkbox border-none"
      role="checkbox"
      :disabled="disabled"
      :class="{ checked: modelValue, collapsing: collapsingToggleStyle }"
      :aria-label="description"
      :aria-checked="modelValue"
    >
      <MinusIcon v-if="indeterminate" aria-hidden="true" />
      <CheckIcon v-else-if="modelValue && !collapsingToggleStyle" aria-hidden="true" />
      <DropdownIcon v-else-if="collapsingToggleStyle" aria-hidden="true" />
    </button>
    <!-- aria-hidden is set so screenreaders only use the <button>'s aria-label -->
    <p v-if="label" aria-hidden="true" class="checkbox-label">
      {{ label }}
    </p>
    <slot v-else />
  </div>
</template>
<script setup lang="ts">
import { CheckIcon, DropdownIcon, MinusIcon } from '@modrinth/assets'

const emit = defineEmits<{
  'update:modelValue': [boolean]
}>()

const props = withDefaults(
  defineProps<{
    label?: string
    disabled?: boolean
    description?: string
    modelValue: boolean
    clickEvent?: () => void
    collapsingToggleStyle?: boolean
    indeterminate?: boolean
  }>(),
  {
    label: '',
    disabled: false,
    description: '',
    modelValue: false,
    clickEvent: () => {},
    collapsingToggleStyle: false,
    indeterminate: false,
  },
)

function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
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

  color: var(--color-contrast);
  background-color: var(--color-button-bg);
  border-radius: var(--radius-xs);
  box-shadow:
    var(--shadow-inset-sm),
    0 0 0 0 transparent;

  &.checked {
    background-color: var(--color-brand);

    svg {
      color: var(--color-accent-contrast);
    }
  }

  svg {
    color: var(--color-secondary);
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

      @media (prefers-reduced-motion) {
        transition: none !important;
      }
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

.checkbox-label {
  color: var(--color-base);
}
</style>
