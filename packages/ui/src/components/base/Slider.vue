<template>
  <div class="root-container">
    <div class="slider-component">
      <div class="slide-container">
        <div class="snap-points-wrapper">
          <div class="snap-points">
            <div
              v-for="snapPoint in snapPoints"
              :key="snapPoint"
              class="snap-point"
              :class="{ green: snapPoint <= currentValue }"
              :style="{ left: ((snapPoint - min) / (max - min)) * 100 + '%' }"
            ></div>
          </div>
        </div>
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
          @input="onInputWithSnap(($event.target as HTMLInputElement).value)"
        />
        <div class="slider-range">
          <span> {{ min }} {{ unit }} </span>
          <span> {{ max }} {{ unit }} </span>
        </div>
      </div>
    </div>
    <input
      ref="value"
      :value="currentValue"
      type="number"
      class="slider-input"
      :disabled="disabled"
      :min="min"
      :max="max"
      :step="step"
      @change="onInput(($event.target as HTMLInputElement).value)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{ 'update:modelValue': [number] }>()

interface Props {
  modelValue?: number
  min: number
  max: number
  step?: number
  forceStep?: boolean
  snapPoints?: number[]
  snapRange?: number
  disabled?: boolean
  unit?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: 0,
  min: 0,
  max: 100,
  step: 10,
  forceStep: true,
  snapPoints: () => [],
  snapRange: 100,
  disabled: false,
  unit: '',
})

const currentValue = ref(Math.max(props.min, props.modelValue))

const inputValueValid = (inputValue: number) => {
  let newValue = inputValue || props.min

  if (props.forceStep) {
    newValue -= newValue % props.step
  }
  newValue = Math.max(props.min, Math.min(newValue, props.max))

  currentValue.value = newValue
  emit('update:modelValue', currentValue.value)
}

const onInputWithSnap = (value: string) => {
  let parsedValue = parseInt(value)

  for (const snapPoint of props.snapPoints) {
    const distance = Math.abs(snapPoint - parsedValue)

    if (distance < props.snapRange) {
      parsedValue = snapPoint
    }
  }

  inputValueValid(parsedValue)
}

const onInput = (value: string) => {
  inputValueValid(parseInt(value))
}
</script>

<style lang="scss" scoped>
.root-container {
  --transition-speed: 0.2s;

  @media (prefers-reduced-motion) {
    --transition-speed: 0s;
  }

  display: flex;
  flex-direction: row;
  align-items: center;
  width: 100%;
}

.slider-component,
.slide-container {
  width: 100%;

  position: relative;
}

.slider-component .slide-container .slider {
  -webkit-appearance: none;
  appearance: none;
  position: relative;

  border-radius: var(--radius-sm);
  height: 0.25rem;
  width: 100%;
  padding: 0;
  min-height: 0px;
  box-shadow: none;

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
  transition: var(--transition-speed);
}

.slider-component .slide-container .slider::-moz-range-thumb {
  border: none;
  width: 0.75rem;
  height: 0.75rem;
  background: var(--color-brand);
  cursor: pointer;
  border-radius: 50%;
  transition: var(--transition-speed);
}

.slider-component .slide-container .slider:hover::-webkit-slider-thumb:not(.disabled) {
  width: 1rem;
  height: 1rem;
  transition: var(--transition-speed);
}

.slider-component .slide-container .slider:hover::-moz-range-thumb:not(.disabled) {
  width: 1rem;
  height: 1rem;
  transition: var(--transition-speed);
}

.slider-component .slide-container .snap-points-wrapper {
  position: absolute;
  height: 50%;
  width: 100%;

  .snap-points {
    position: relative;
    display: inline-block;

    vertical-align: middle;

    width: calc(100% - 0.75rem);
    height: 0.75rem;

    left: calc(0.75rem / 2);

    .snap-point {
      position: absolute;
      display: inline-block;

      width: 0.25rem;
      height: 100%;
      border-radius: var(--radius-sm);

      background-color: var(--color-base);

      transform: translateX(calc(-0.25rem / 2));

      &.green {
        background-color: var(--color-brand);
      }
    }
  }
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
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
