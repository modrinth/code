<template>
  <nav
    ref="scrollContainer"
    class="card-shadow experimental-styles-within relative flex w-fit overflow-x-auto rounded-full bg-bg-raised p-1 text-sm font-bold"
  >
    <button
      v-for="(option, index) in options"
      :key="`option-group-${index}`"
      ref="optionButtons"
      class="button-animation z-[1] flex flex-row items-center gap-2 rounded-full bg-transparent px-4 py-2 font-semibold"
      :class="{
        'text-button-textSelected': modelValue === option,
        'text-primary': modelValue !== option,
      }"
      @click="setOption(option)"
    >
      <slot :option="option" :selected="modelValue === option" />
    </button>
    <div
      class="navtabs-transition pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full bg-button-bgSelected p-1"
      :style="{
        left: sliderLeftPx,
        top: sliderTopPx,
        right: sliderRightPx,
        bottom: sliderBottomPx,
        opacity: initialized ? 1 : 0,
      }"
      aria-hidden="true"
    ></div>
  </nav>
</template>

<script setup lang="ts" generic="T">
import { ref, computed, onMounted } from "vue";

const modelValue = defineModel<T>({ required: true });

const props = defineProps<{
  options: T[];
}>();

const scrollContainer = ref<HTMLElement | null>(null);

const sliderLeft = ref(4);
const sliderTop = ref(4);
const sliderRight = ref(4);
const sliderBottom = ref(4);

const sliderLeftPx = computed(() => `${sliderLeft.value}px`);
const sliderTopPx = computed(() => `${sliderTop.value}px`);
const sliderRightPx = computed(() => `${sliderRight.value}px`);
const sliderBottomPx = computed(() => `${sliderBottom.value}px`);

const optionButtons = ref();

const initialized = ref(false);

function setOption(option: T) {
  modelValue.value = option;
}

watch(modelValue, () => {
  startAnimation(props.options.indexOf(modelValue.value));
});

function startAnimation(index: number) {
  const el = optionButtons.value[index];

  if (!el || !el.offsetParent) return;

  const newValues = {
    left: el.offsetLeft,
    top: el.offsetTop,
    right: el.offsetParent.offsetWidth - el.offsetLeft - el.offsetWidth,
    bottom: el.offsetParent.offsetHeight - el.offsetTop - el.offsetHeight,
  };

  if (sliderLeft.value === 4 && sliderRight.value === 4) {
    sliderLeft.value = newValues.left;
    sliderRight.value = newValues.right;
    sliderTop.value = newValues.top;
    sliderBottom.value = newValues.bottom;
  } else {
    const delay = 200;

    if (newValues.left < sliderLeft.value) {
      sliderLeft.value = newValues.left;
      setTimeout(() => {
        sliderRight.value = newValues.right;
      }, delay);
    } else {
      sliderRight.value = newValues.right;
      setTimeout(() => {
        sliderLeft.value = newValues.left;
      }, delay);
    }

    if (newValues.top < sliderTop.value) {
      sliderTop.value = newValues.top;
      setTimeout(() => {
        sliderBottom.value = newValues.bottom;
      }, delay);
    } else {
      sliderBottom.value = newValues.bottom;
      setTimeout(() => {
        sliderTop.value = newValues.top;
      }, delay);
    }
  }
  initialized.value = true;
}

onMounted(() => {
  startAnimation(props.options.indexOf(modelValue.value));
});
</script>

<style scoped>
.navtabs-transition {
  transition:
    all 150ms cubic-bezier(0.4, 0, 0.2, 1),
    opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}

.card-shadow {
  box-shadow: var(--shadow-card);
}
</style>
