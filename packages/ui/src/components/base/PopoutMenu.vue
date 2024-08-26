<template>
  <div ref="dropdown" class="popup-container" tabindex="-1" :aria-expanded="dropdownVisible">
    <button
      v-bind="$attrs"
      ref="dropdownButton"
      :class="{ 'popout-open': dropdownVisible }"
      :tabindex="tabInto ? -1 : 0"
      @click="toggleDropdown"
    >
      <slot></slot>
    </button>
    <div
      class="popup-menu"
      :class="`position-${computedPosition}-${computedDirection} ${dropdownVisible ? 'visible' : ''}`"
      :inert="!tabInto && !dropdownVisible"
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
    default: 'auto',
  },
  direction: {
    type: String,
    default: 'auto',
  },
  tabInto: {
    type: Boolean,
    default: false,
  },
})
defineOptions({
  inheritAttrs: false,
})

const emit = defineEmits(['open', 'close'])

const dropdownVisible = ref(false)
const dropdown = ref(null)
const dropdownButton = ref(null)
const computedPosition = ref('bottom')
const computedDirection = ref('left')

function updateDirection() {
  if (props.direction === 'auto') {
    if (dropdownButton.value) {
      const x = dropdownButton.value.getBoundingClientRect().left
      computedDirection.value = x < window.innerWidth / 2 ? 'right' : 'left'
    } else {
      computedDirection.value = 'left'
    }
  } else {
    computedDirection.value = props.direction
  }
  if (props.position === 'auto') {
    if (dropdownButton.value) {
      const y = dropdownButton.value.getBoundingClientRect().top
      computedPosition.value = y < window.innerHeight / 2 ? 'bottom' : 'top'
    } else {
      computedPosition.value = 'bottom'
    }
  } else {
    computedPosition.value = props.position
  }
}

const toggleDropdown = () => {
  if (!props.disabled) {
    dropdownVisible.value = !dropdownVisible.value
    if (dropdownVisible.value) {
      emit('open')
    } else {
      dropdownButton.value.focus()
      emit('close')
    }
  }
}

const hide = () => {
  dropdownVisible.value = false
  dropdownButton.value.focus()
  emit('close')
}

const show = () => {
  dropdownVisible.value = true
  emit('open')
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
    emit('close')
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
  window.addEventListener('resize', updateDirection)
  window.addEventListener('scroll', updateDirection)
  window.addEventListener('keydown', handleKeyDown)
  updateDirection()
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
  window.removeEventListener('resize', updateDirection)
  window.removeEventListener('scroll', updateDirection)
  window.removeEventListener('keydown', handleKeyDown)
})

function handleKeyDown(event) {
  if (event.key === 'Escape') {
    hide()
  }
}
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
