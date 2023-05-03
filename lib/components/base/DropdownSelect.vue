<template>
  <div
    ref="dropdown"
    tabindex="0"
    role="combobox"
    :aria-expanded="dropdownVisible"
    class="animated-dropdown"
    @focus="onFocus"
    @blur="onBlur"
    @mousedown.prevent
    @keydown.enter.prevent="toggleDropdown"
    @keydown.up.prevent="focusPreviousOption"
    @keydown.down.prevent="focusNextOptionOrOpen"
  >
    <div
      class="selected"
      :class="{
        disabled: disabled,
        'render-down': dropdownVisible && !renderUp && !disabled,
        'render-up': dropdownVisible && renderUp && !disabled,
      }"
      @click="toggleDropdown"
    >
      <span>{{ selectedOption }}</span>
      <i class="arrow" :class="{ rotate: dropdownVisible }"></i>
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
            <label :for="`${name}-${index}`">{{ displayName(option) }}</label>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue'

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
  placeholder: {
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
  displayName: {
    type: Function,
    default: (option) => option,
  },
})

const emit = defineEmits(['input', 'change', 'update:modelValue'])

const dropdownVisible = ref(false)
const selectedValue = ref(props.modelValue || props.defaultValue)
const focusedOptionIndex = ref(null)
const dropdown = ref(null)
const optionElements = ref(null)

const selectedOption = computed(() => {
  return props.displayName(selectedValue.value) || props.placeholder || 'Select an option'
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

const onFocus = () => {
  if (!props.disabled) {
    focusedOptionIndex.value = props.options.findIndex((option) => option === selectedValue.value)
    dropdownVisible.value = true
  }
}

const onBlur = (event) => {
  if (!isChildOfDropdown(event.relatedTarget)) {
    dropdownVisible.value = false
  }
}

const focusPreviousOption = () => {
  if (!props.disabled) {
    if (!dropdownVisible.value) {
      toggleDropdown()
    }
    focusedOptionIndex.value =
      (focusedOptionIndex.value + props.options.length - 1) % props.options.length
    optionElements.value[focusedOptionIndex.value].focus()
  }
}

const focusNextOptionOrOpen = () => {
  if (!props.disabled) {
    if (!dropdownVisible.value) {
      toggleDropdown()
    }
    focusedOptionIndex.value = (focusedOptionIndex.value + 1) % props.options.length
    optionElements.value[focusedOptionIndex.value].focus()
  }
}

const isChildOfDropdown = (element) => {
  let currentNode = element
  while (currentNode) {
    if (currentNode === this.$el) {
      return true
    }
    currentNode = currentNode.parentNode
  }
  return false
}
</script>

<style lang="scss" scoped>
.animated-dropdown {
  width: 20rem;
  position: relative;
  display: inline-block;

  &:focus {
    outline: 0;
  }

  .selected {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--gap-sm) var(--gap-lg);
    background-color: var(--color-button-bg);
    cursor: pointer;
    user-select: none;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-inset-sm), 0 0 0 0 transparent;

    &.disabled {
      cursor: not-allowed;
      filter: brightness(0.75);
    }

    &.render-up {
      border-radius: 0 0 var(--radius-md) var(--radius-md);
    }

    &.render-down {
      border-radius: var(--radius-md) var(--radius-md) 0 0;
    }

    &:focus {
      outline: 0;
      filter: brightness(1.25);
      transition: filter 0.1s ease-in-out;
    }

    .arrow {
      display: inline-block;
      width: 0;
      height: 0;
      margin-left: 0.4rem;
      border-left: 0.4rem solid transparent;
      border-right: 0.4rem solid transparent;
      border-top: 0.4rem solid var(--color-base);
      transition: transform 0.2s ease;
      &.rotate {
        transform: rotate(180deg);
      }
    }
  }

  .options {
    z-index: 10;
    max-height: min(12rem);
    overflow-y: auto;
    box-shadow: var(--shadow-inset-sm), 0 0 0 0 transparent;

    .option {
      background-color: var(--color-button-bg);
      display: flex;
      align-items: center;
      padding: var(--gap-md);
      cursor: pointer;
      user-select: none;

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
  &.up {
    transform: translateY(100%);
  }

  &.down {
    transform: translateY(-100%);
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
    transform: translateY(-100%);
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }

  &.down {
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }
}
</style>
