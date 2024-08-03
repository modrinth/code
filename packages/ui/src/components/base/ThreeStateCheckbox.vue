<template>
  <div
    class="checkbox-outer button-within"
    :class="{ disabled }"
    role="presentation"
    @click="toggle"
  >
    <button
      class="checkbox"
      role="checkbox"
      :disabled="disabled"
      :class="{ positive: state == 'POSITIVE', negative: state == 'NEGATIVE' }"
      :aria-label="description"
    >
      <CheckIcon v-if="state == 'POSITIVE'" aria-hidden="true" />
      <div v-if="state == 'NEGATIVE'"></div>
    </button>
    <!-- aria-hidden is set so screenreaders only use the <button>'s aria-label -->
    <p v-if="label" aria-hidden="true">
      {{ label }}
    </p>
    <slot v-else />
  </div>
</template>
<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'

type State = 'POSITIVE' | 'NEGATIVE' | 'NEUTRAL'

const state = defineModel<State>({ default: 'NEUTRAL' })

const props = withDefaults(
  defineProps<{
    label?: string
    disabled?: boolean
    description?: string
  }>(),
  {
    label: '',
    disabled: false,
    description: '',
  },
)

function toggle() {
  if (!props.disabled) {
    switch (state.value) {
      case 'POSITIVE':
        state.value = 'NEGATIVE'
        break
      case 'NEGATIVE':
        state.value = 'NEUTRAL'
        break
      case 'NEUTRAL':
        state.value = 'POSITIVE'
        break
    }
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
  outline: none;

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

  &.positive {
    background-color: var(--color-brand);
  }

  &.negative {
    background-color: var(--color-red);

    div {
      height: 0.2rem;
      width: 0.8rem;
      border-radius: var(--radius-xs);
      background: var(--color-accent-contrast);
    }
  }

  svg {
    color: var(--color-accent-contrast);
    stroke-width: 0.2rem;
    height: 0.8rem;
    width: 0.8rem;
    flex-shrink: 0;
  }

  &:disabled {
    box-shadow: none;
    cursor: not-allowed;
  }
}
</style>
