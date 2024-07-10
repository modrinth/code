<template>
  <div ref="dropdown" class="popup-container" tabindex="-1" :aria-expanded="dropdownVisible">
    <button
      v-bind="$attrs"
      ref="dropdownButton"
      :class="{ 'popout-open': dropdownVisible }"
      tabindex="-1"
      @click="toggleDropdown"
    >
      <slot></slot>
    </button>
    <div
      class="popup-menu"
      :class="`position-${position}-${direction} ${dropdownVisible ? 'visible' : ''}`"
    >
      <slot name="menu"> </slot>
    </div>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps({
  disabled: {
    type: Boolean,
    default: false,
  },
  position: {
    type: String,
    default: 'bottom',
  },
  direction: {
    type: String,
    default: 'left',
  },
})
defineOptions({
  inheritAttrs: false,
})

const dropdownVisible = ref(false)
const dropdown = ref(null)
const dropdownButton = ref(null)

const toggleDropdown = () => {
  if (!props.disabled) {
    dropdownVisible.value = !dropdownVisible.value
    if (!dropdownVisible.value) {
      dropdownButton.value.focus()
    }
  }
}

const hide = () => {
  dropdownVisible.value = false
  dropdownButton.value.focus()
}

const show = () => {
  dropdownVisible.value = true
}

defineExpose({
  show,
  hide,
})

const handleClickOutside = (event) => {
  const elements = document.elementsFromPoint(event.clientX, event.clientY)
  if (
    dropdown.value.$el !== event.target &&
    !elements.includes(dropdown.value.$el) &&
    !dropdown.value.contains(event.target)
  ) {
    dropdownVisible.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
})
</script>

<style lang="scss" scoped>
.popup-container {
  position: relative;

  .popup-menu {
    --_animation-offset: -1rem;
    position: absolute;
    scale: 0.75;
    border: 1px solid var(--color-button-bg);
    padding: var(--gap-sm);
    width: fit-content;
    border-radius: var(--radius-md);
    background-color: var(--color-raised-bg);
    box-shadow: var(--shadow-floating);
    z-index: 10;
    opacity: 0;
    transition:
      bottom 0.125s ease-in-out,
      top 0.125s ease-in-out,
      left 0.125s ease-in-out,
      right 0.125s ease-in-out,
      opacity 0.125s ease-in-out,
      scale 0.125s ease-in-out;

    @media (prefers-reduced-motion) {
      transition: none !important;
    }

    &.position-bottom-left {
      top: calc(100% + var(--gap-sm) - 1rem);
      right: -1rem;
    }

    &.position-bottom-right {
      top: calc(100% + var(--gap-sm) - 1rem);
      left: -1rem;
    }

    &.position-top-left {
      bottom: calc(100% + var(--gap-sm) - 1rem);
      right: -1rem;
    }

    &.position-top-right {
      bottom: calc(100% + var(--gap-sm) - 1rem);
      left: -1rem;
    }

    &.position-left-up {
      bottom: -1rem;
      right: calc(100% + var(--gap-sm) - 1rem);
    }

    &.position-left-down {
      top: -1rem;
      right: calc(100% + var(--gap-sm) - 1rem);
    }

    &.position-right-up {
      bottom: -1rem;
      left: calc(100% + var(--gap-sm) - 1rem);
    }

    &.position-right-down {
      top: -1rem;
      left: calc(100% + var(--gap-sm) - 1rem);
    }

    &:not(.visible):not(:focus-within) {
      pointer-events: none;

      *,
      ::before,
      ::after {
        pointer-events: none;
      }
    }

    &.visible,
    &:focus-within {
      opacity: 1;
      scale: 1;

      &.position-bottom-left {
        top: calc(100% + var(--gap-sm));
        right: 0;
      }

      &.position-bottom-right {
        top: calc(100% + var(--gap-sm));
        left: 0;
      }

      &.position-top-left {
        bottom: calc(100% + var(--gap-sm));
        right: 0;
      }

      &.position-top-right {
        bottom: calc(100% + var(--gap-sm));
        left: 0;
      }

      &.position-left-up {
        bottom: 0rem;
        right: calc(100% + var(--gap-sm));
      }

      &.position-left-down {
        top: 0rem;
        right: calc(100% + var(--gap-sm));
      }

      &.position-right-up {
        bottom: 0rem;
        left: calc(100% + var(--gap-sm));
      }

      &.position-right-down {
        top: 0rem;
        left: calc(100% + var(--gap-sm));
      }
    }

    .btn {
      white-space: nowrap;
    }
  }
}
</style>
