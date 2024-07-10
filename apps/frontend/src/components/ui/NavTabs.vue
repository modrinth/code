<template>
  <nav class="relative flex w-fit overflow-clip rounded-full bg-bg-raised p-1 text-sm font-bold">
    <NuxtLink
      v-for="(link, index) in filteredLinks"
      v-show="link.shown === undefined ? true : link.shown"
      :key="index"
      ref="linkElements"
      :to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
      class="button-animation flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
      :class="{ 'text-brand': activeIndex === index }"
    >
      <component :is="link.icon" v-if="link.icon" class="size-5" />
      <span>{{ link.label }}</span>
    </NuxtLink>
    <div
      class="pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full bg-brand p-1 transition-all"
      :style="{
        left: positionToMoveX,
        top: positionToMoveY,
        width: sliderWidth,
        opacity: activeIndex === -1 ? 0 : 0.25,
      }"
      aria-hidden="true"
    ></div>
  </nav>
</template>

<script setup>
const route = useNativeRoute();

const props = defineProps({
  links: {
    default: () => [],
    type: Array,
  },
  query: {
    default: null,
    type: String,
  },
});

const sliderPositionX = ref(0);
const sliderPositionY = ref(0);
const selectedElementWidth = ref(0);
const activeIndex = ref(-1);
const oldIndex = ref(-1);

const filteredLinks = computed(() =>
  props.links.filter((x) => (x.shown === undefined ? true : x.shown)),
);
const positionToMoveX = computed(() => `${sliderPositionX.value}px`);
const positionToMoveY = computed(() => `${sliderPositionY.value}px`);
const sliderWidth = computed(() => `${selectedElementWidth.value}px`);

function pickLink() {
  let index = -1;
  for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
    if (decodeURIComponent(route.path).includes(filteredLinks.value[i].href)) {
      index = i;
      break;
    }
  }
  activeIndex.value = index;

  if (activeIndex.value !== -1) {
    startAnimation();
  } else {
    oldIndex.value = -1;
    sliderPositionX.value = 0;
    selectedElementWidth.value = 0;
  }
}

const linkElements = ref();

function startAnimation() {
  const el = linkElements.value[activeIndex.value].$el;

  sliderPositionX.value = el.offsetLeft;
  sliderPositionY.value = el.offsetTop;
  selectedElementWidth.value = el.offsetWidth;
}

onMounted(() => {
  window.addEventListener("resize", pickLink);
  pickLink();
});

onUnmounted(() => {
  window.removeEventListener("resize", pickLink);
});

watch(route, () => pickLink());
</script>
