<template>
  <div
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
    <div class="selected" :class="{ 'dropdown-open': dropdownVisible }" @click="toggleDropdown">
      <span>{{ selectedOption }}</span>
      <i class="arrow" :class="{ rotate: dropdownVisible }"></i>
    </div>
    <transition name="slide-fade">
      <div v-show="dropdownVisible" class="options" role="listbox">
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
            v-model="selectedValue"
            type="radio"
            :value="option"
            :name="name"
          />
          <label :for="`${name}-${index}`">{{ option }}</label>
        </div>
      </div>
    </transition>
  </div>
</template>

<script>
export default {
  props: {
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
  },
  emits: ['input', 'change'],
  data() {
    return {
      dropdownVisible: false,
      selectedValue: this.defaultValue,
      focusedOptionIndex: null,
    }
  },
  computed: {
    selectedOption() {
      return this.selectedValue || this.placeholder || 'Select an option'
    },
  },
  methods: {
    toggleDropdown() {
      this.dropdownVisible = !this.dropdownVisible
    },
    selectOption(option, index) {
      this.selectedValue = option
      this.$emit('input', this.selectedValue)
      this.$emit('change', { option, index })
      this.dropdownVisible = false
    },
    onFocus() {
      this.focusedOptionIndex = this.options.findIndex((option) => option === this.selectedValue)
      this.dropdownVisible = true
    },
    onBlur(event) {
      if (!this.isChildOfDropdown(event.relatedTarget)) {
        this.dropdownVisible = false
      }
    },
    focusPreviousOption() {
      if (!this.dropdownVisible) {
        this.toggleDropdown()
      }
      this.focusedOptionIndex =
        (this.focusedOptionIndex + this.options.length - 1) % this.options.length
      this.$refs.optionElements[this.focusedOptionIndex].focus()
    },
    focusNextOptionOrOpen() {
      if (!this.dropdownVisible) {
        this.toggleDropdown()
      }
      this.focusedOptionIndex = (this.focusedOptionIndex + 1) % this.options.length
      this.$refs.optionElements[this.focusedOptionIndex].focus()
    },
    isChildOfDropdown(element) {
      let currentNode = element
      while (currentNode) {
        if (currentNode === this.$el) {
          return true
        }
        currentNode = currentNode.parentNode
      }
      return false
    },
  },
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
    padding: var(--gap-sm) var(--gap-md);
    background-color: var(--color-button-bg);
    cursor: pointer;
    user-select: none;
    border-radius: var(--radius-md);

    &:hover {
      filter: brightness(1.25);
      transition: filter 0.3s ease-in-out;
    }

    &.dropdown-open {
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
      transition: transform 0.3s ease;
      &.rotate {
        transform: rotate(180deg);
      }
    }
  }

  .options {
    position: absolute;
    width: 100%;
    z-index: 10;
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
    overflow: hidden;

    .option {
      background-color: var(--color-button-bg);
      display: flex;
      align-items: center;
      padding: var(--gap-md);
      cursor: pointer;
      user-select: none;

      &:hover {
        filter: brightness(1.25);
        transition: filter 0.3s ease-in-out;
      }

      &:focus {
        outline: 0;
        filter: brightness(1.25);
        transition: filter 0.3s ease-in-out;
      }

      &.selected-option {
        background-color: var(--color-brand);
        color: var(--color-accent-contrast);
      }

      input {
        display: none;
      }
    }
  }
}

.slide-fade-enter {
  opacity: 0;
  transform: translateY(-20px);
}

.slide-fade-enter-to {
  opacity: 1;
  transform: translateY(0);
}

.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: opacity 0.5s ease, transform 0.3s ease;
}

.slide-fade-leave,
.slide-fade-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
