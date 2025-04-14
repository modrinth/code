<template>
  <div
    ref="dropdown"
    tabindex="0"
    role="combobox"
    :aria-expanded="dropdownVisible"
    class="animated-dropdown"
    @keydown.up.prevent="focusPreviousOption"
    @keydown.down.prevent="focusNextOptionOrOpen"
  >
    <div class="iconified-input">
      <SearchIcon />
      <input
        :value="modelValue"
        type="text"
        :name="name"
        :disabled="disabled"
        class="text-input"
        autocomplete="off"
        autocapitalize="off"
        :placeholder="placeholder"
        :class="{ down: !renderUp, up: renderUp }"
        @input="$emit('update:modelValue', $event.target.value)"
        @focus="onFocus"
        @blur="onBlur"
        @focusout="onBlur"
        @keydown.enter.prevent="$emit('enter')"
      />
      <Button :disabled="disabled" class="r-btn" @click="() => $emit('update:modelValue', '')">
        <XIcon />
      </Button>
    </div>
    <div ref="dropdownOptions" class="options-wrapper" :class="{ down: !renderUp, up: renderUp }">
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
            class="option"
            @click="selectOption(option)"
          >
            <div class="project-label">
              <Avatar :src="option.icon" :circle="circledIcons" />
              <div class="text">
                <div class="title">
                  {{ getOptionLabel(option.title) }}
                </div>
                <div class="author">
                  {{ getOptionLabel(option.subtitle) }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { XIcon, SearchIcon } from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import Button from '../base/Button.vue'

const props = defineProps({
  options: {
    type: Array,
    required: true,
  },
  name: {
    type: String,
    required: true,
  },
  placeholder: {
    type: [String, Number],
    default: null,
  },
  modelValue: {
    type: [String, Number, Object],
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
    default: undefined,
  },
  circledIcons: {
    type: Boolean,
    default: false,
  },
})

function getOptionLabel(option) {
  return props.displayName?.(option) ?? option
}

const emit = defineEmits(['input', 'onSelected', 'update:modelValue', 'enter'])

const dropdownVisible = ref(false)
const focusedOptionIndex = ref(null)
const dropdown = ref(null)
const optionElements = ref(null)
const dropdownOptions = ref(null)

const toggleDropdown = () => {
  if (!props.disabled) {
    dropdownVisible.value = !dropdownVisible.value
    dropdown.value.focus()
  }
}

const selectOption = (option) => {
  emit('onSelected', option)
  console.log('onSelected', option)
  dropdownVisible.value = false
}

const onFocus = () => {
  if (!props.disabled) {
    focusedOptionIndex.value = props.options.findIndex(
      (option) => option === props.modelValue.value,
    )
    dropdownVisible.value = true
  }
}

const onBlur = (event) => {
  console.log(event)
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
    if (currentNode === dropdownOptions.value) {
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
  height: 2.5rem;
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
    gap: var(--gap-md);
    cursor: pointer;
    user-select: none;
    border-radius: var(--radius-md);
    box-shadow:
      var(--shadow-inset-sm),
      0 0 0 0 transparent;

    &.disabled {
      cursor: not-allowed;
      filter: grayscale(50%);
      opacity: 0.5;
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
  }

  .options {
    z-index: 10;
    max-height: 18rem;
    overflow-y: auto;

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

.project-label {
  display: flex;
  align-items: center;
  flex-direction: row;
  gap: var(--gap-md);
  color: var(--color-contrast);

  .title {
    font-weight: bold;
  }
}

.iconified-input {
  width: 100%;
}

.text-input {
  box-shadow:
    var(--shadow-inset-sm),
    0 0 0 0 transparent !important;
  width: 100%;

  transition: 0.05s;

  &:focus {
    &.down {
      border-radius: var(--radius-md) var(--radius-md) 0 0 !important;
    }

    &.up {
      border-radius: 0 0 var(--radius-md) var(--radius-md) !important;
    }
  }

  &:not(:focus) {
    transition-delay: 0.2s;
  }
}
</style>
