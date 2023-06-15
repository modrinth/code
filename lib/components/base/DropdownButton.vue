<template>
  <div
    ref="dropdown"
    tabindex="0"
    role="combobox"
    :aria-expanded="dropdownVisible"
    class="animated-dropdown"
  >
    <div class="dropdown-row">
      <Button
        class="dropdown-button"
        :class="{
          'render-down': dropdownVisible && !renderUp && !disabled,
          'render-up': dropdownVisible && renderUp && !disabled,
        }"
        :disabled="disabled"
        :color="color"
        @click="clickOption"
      >
        <slot :name="selectedOption" />
      </Button>
      <Button
        class="selected"
        :class="{
          'render-down': dropdownVisible && !renderUp && !disabled,
          'render-up': dropdownVisible && renderUp && !disabled,
        }"
        icon-only
        :color="color"
        :disabled="disabled"
        @click="toggleDropdown"
      >
        <DropdownIcon class="arrow" :class="{ rotate: dropdownVisible }" />
      </Button>
    </div>
    <div class="options-wrapper" :class="{ down: !renderUp, up: renderUp }">
      <transition name="options">
        <div
          v-show="dropdownVisible"
          class="options"
          role="listbox"
          :class="{ down: !renderUp, up: renderUp }"
        >
          <div
            v-for="(option, index) in options"
            :key="index"
            ref="optionElements"
            tabindex="-1"
            role="option"
            :class="{ 'selected-option': selectedValue === option }"
            :aria-selected="selectedValue === option"
            class="option"
            @click="selectOption(option, index)"
            @keydown.space.prevent="selectOption(option, index)"
          >
            <input
              :id="`${name}-${index}`"
              v-model="radioValue"
              type="radio"
              :value="option"
              :name="name"
            />
            <label :for="`${name}-${index}`" class="slot">
              <slot :name="option" />
            </label>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Button, DropdownIcon } from '@/components'

const props = defineProps({
  options: {
    type: Array,
    required: true,
  },
  name: {
    type: String,
    required: true,
  },
  defaultValue: {
    type: String,
    default: null,
  },
  modelValue: {
    type: String,
    default: null,
  },
  renderUp: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  color: {
    type: String,
    default: '',
  },
})

const emit = defineEmits(['input', 'change', 'update:modelValue', 'option-click'])

const dropdownVisible = ref(false)
const selectedValue = ref(props.modelValue || props.defaultValue)
const dropdown = ref(null)
const optionElements = ref(null)

const selectedOption = computed(() => {
  return selectedValue.value
})

const radioValue = computed({
  get() {
    return props.modelValue || selectedValue.value
  },
  set(newValue) {
    emit('update:modelValue', newValue)
    selectedValue.value = newValue
  },
})

watch(
  () => props.modelValue,
  (newValue) => {
    selectedValue.value = newValue
  }
)

const toggleDropdown = () => {
  if (!props.disabled) {
    dropdownVisible.value = !dropdownVisible.value
    dropdown.value.focus()
  }
}

const selectOption = (option, index) => {
  radioValue.value = option
  emit('change', { option, index })
  dropdownVisible.value = false
}

const clickOption = () => {
  emit('option-click', { option: selectedOption.value })
  dropdownVisible.value = false
}

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
.animated-dropdown {
  position: relative;
  display: inline-block;

  &:focus {
    outline: 0;
  }

  .dropdown-row {
    display: flex;
    flex-direction: row;
    align-items: center;

    .dropdown-button {
      width: 100%;
      border-radius: var(--radius-md) 0 0 var(--radius-md);

      &.render-up {
        border-radius: 0 0 0 var(--radius-md);
      }

      &.render-down {
        border-radius: var(--radius-md) 0 0 0;
      }
    }
  }

  .selected {
    height: auto;
    width: auto;
    margin-left: 1px;
    padding: var(--gap-sm);
    border-radius: 0 var(--radius-md) var(--radius-md) 0;

    &.render-up {
      border-radius: 0 0 var(--radius-md) 0;
    }

    &.render-down {
      border-radius: 0 var(--radius-md) 0 0;
    }

    &:focus {
      outline: 0;
      filter: brightness(1.25);
      transition: filter 0.1s ease-in-out;
    }

    .arrow {
      min-width: 1.125rem;
      min-height: 1.125rem;
      display: inline-block;
      transition: transform 0.2s ease;

      &.rotate {
        transform: rotate(180deg);
      }
    }
  }

  .options {
    z-index: 10;
    max-height: 18.75rem;
    overflow-y: auto;
    box-shadow: var(--shadow-inset-sm), 0 0 0 0 transparent;

    .option {
      background-color: var(--color-button-bg);
      display: flex;
      align-items: center;
      padding: var(--gap-md);
      cursor: pointer;
      user-select: none;

      .slot {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: var(--gap-sm);
      }

      &:hover {
        filter: brightness(0.85);
        transition: filter 0.2s ease-in-out;
      }

      &:focus {
        outline: 0;
        filter: brightness(0.85);
        transition: filter 0.2s ease-in-out;
      }

      &.selected-option {
        background-color: var(--color-brand);
        color: var(--color-accent-contrast);
        font-weight: bolder;
      }

      input {
        display: none;
      }
    }
  }
}

.options-enter-active,
.options-leave-active {
  transition: transform 0.2s ease;
}

.options-enter-from,
.options-leave-to {
  // this is not 100% due to a safari bug
  &.up {
    transform: translateY(99.999%);
  }

  &.down {
    transform: translateY(-99.999%);
  }
}

.options-enter-to,
.options-leave-from {
  &.up {
    transform: translateY(0%);
  }
}

.options-wrapper {
  position: absolute;
  width: 100%;
  overflow: auto;
  z-index: 9;

  &.up {
    top: 0;
    transform: translateY(-99.999%);
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }

  &.down {
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }
}
</style>
