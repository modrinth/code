<template>
  <div class="root-container">
    <div class="slider-component">
      <div class="slide-container">
        <input
          ref="input"
          v-model="currentValue"
          type="range"
          :min="min"
          :max="max"
          :step="step"
          class="slider"
          :class="{
            disabled: disabled,
          }"
          :disabled="disabled"
          :style="{
            '--current-value': currentValue,
            '--min-value': min,
            '--max-value': max,
          }"
          @input="onInput($refs.input.value)"
        />
        <div class="slider-range">
          <span>
            {{ min }}
          </span>
          <span>
            {{ max }}
          </span>
        </div>
      </div>
    </div>
    <input
      ref="value"
      :value="currentValue"
      type="text"
      class="slider-input"
      :disabled="disabled"
      @change="onInput($refs.value.value)"
    />
  </div>
</template>

<script setup>
import { ref } from 'vue'

const emit = defineEmits(['update:modelValue'])

const props = defineProps({
  modelValue: {
    type: Number,
    default: 0,
  },
  min: {
    type: Number,
    default: 0,
  },
  max: {
    type: Number,
    default: 100,
  },
  step: {
    type: Number,
    default: 10,
  },
  forceStep: {
    type: Boolean,
    default: true,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
})

let currentValue = ref(Math.max(props.min, props.modelValue).toString())

const inputValueValid = (newValue) => {
  const parsedValue = parseInt(newValue)

  if (parsedValue < props.min) {
    currentValue.value = props.min.toString()
  } else if (parsedValue > props.max) {
    currentValue.value = props.max.toString()
  } else if (!parsedValue) {
    currentValue.value = props.min.toString()
  } else {
    currentValue.value = (parsedValue - (props.forceStep ? parsedValue % props.step : 0)).toString()
  }
  emit('update:modelValue', parseInt(currentValue.value))
}

const onInput = (value) => {
  inputValueValid(value)
}
</script>

<style lang="scss" scoped>
.root-container {
  display: flex;
  flex-direction: row;
  align-items: center;
  width: 100%;
}

.slide-container .slider {
  width: 100%;
}

.slider-component,
.slide-container {
  width: 100%;
}

.slider-component .slide-container .slider {
  -webkit-appearance: none;
  appearance: none;
  border-radius: var(--radius-sm);
  height: 0.25rem;
  background: linear-gradient(
    to right,
    var(--color-brand),
    var(--color-brand)
      calc((var(--current-value) - var(--min-value)) / (var(--max-value) - var(--min-value)) * 100%),
    var(--color-base)
      calc((var(--current-value) - var(--min-value)) / (var(--max-value) - var(--min-value)) * 100%),
    var(--color-base) 100%
  );
  background-size: 100% 100%;
  outline: none;
  vertical-align: middle;
}

.slider-component .slide-container .slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 0.75rem;
  height: 0.75rem;
  background: var(--color-brand);
  cursor: pointer;
  border-radius: 50%;
  transition: 0.2s;
}

.slider-component .slide-container .slider::-moz-range-thumb {
  width: 0.75rem;
  height: 0.75rem;
  background: var(--color-brand);
  cursor: pointer;
  border-radius: 50%;
  transition: 0.2s;
}

.slider-component .slide-container .slider:hover::-webkit-slider-thumb:not(.disabled) {
  width: 1rem;
  height: 1rem;
  transition: 0.2s;
}

.slider-component .slide-container .slider:hover::-moz-range-thumb:not(.disabled) {
  width: 1rem;
  height: 1rem;
  transition: 0.2s;
}

.slider-input {
  width: 6rem;
  margin-left: 0.75rem;
}

.slider-range {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  font-size: 0.75rem;
  margin: 0;
}

.disabled {
  filter: brightness(0.8);
}
</style>
