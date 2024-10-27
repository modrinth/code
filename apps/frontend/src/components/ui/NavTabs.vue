<template>
  <nav
    ref="scrollContainer"
    class="relative flex w-fit overflow-x-auto overflow-y-hidden rounded-full bg-bg-raised p-1 text-sm font-bold"
  >
    <div class="flex min-w-fit">
      <NuxtLink
        v-for="(link, index) in filteredLinks"
        v-show="link.shown === undefined ? true : link.shown"
        :key="index"
        ref="tabLinkElements"
        :to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
        class="button-animation z-[1] flex flex-row items-center gap-2 whitespace-nowrap px-4 py-2 focus:rounded-full"
        :class="{
          'text-brand': activeIndex === index && !subpageSelected,
          'text-contrast': activeIndex === index && subpageSelected,
        }"
      >
        <component :is="link.icon" v-if="link.icon" class="size-5" />
        <span>{{ link.label }}</span>
      </NuxtLink>
      <div
        :class="`navtabs-transition pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1 ${
          subpageSelected ? 'bg-button-bg' : 'bg-brand-highlight'
        }`"
        :style="{
          left: sliderLeftPx,
          top: sliderTopPx,
          right: sliderRightPx,
          bottom: sliderBottomPx,
        }"
        aria-hidden="true"
      ></div>
    </div>
  </nav>
</template>

<script setup lang="ts">
interface Tab {
  label: string;
  href: string;
  shown?: boolean;
  icon?: unknown;
  subpages?: string[];
}

const props = defineProps<{
  links: Tab[];
  query?: string;
}>();

const route = useNativeRoute();
const scrollContainer = ref<HTMLElement | null>(null);

const sliderLeft = ref(4);
const sliderTop = ref(4);
const sliderRight = ref(4);
const sliderBottom = ref(4);
const activeIndex = ref(-1);
const subpageSelected = ref(false);

const filteredLinks = computed(() =>
  props.links.filter((x) => (x.shown === undefined ? true : x.shown)),
);
const sliderLeftPx = computed(() => `${sliderLeft.value}px`);
const sliderTopPx = computed(() => `${sliderTop.value}px`);
const sliderRightPx = computed(() => `${sliderRight.value}px`);
const sliderBottomPx = computed(() => `${sliderBottom.value}px`);

const tabLinkElements = ref();

function pickLink() {
  if (route.path.split("/").length < 4) return;
  let index = -1;
  subpageSelected.value = false;

  for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
    const link = filteredLinks.value[i];
    if (decodeURIComponent(route.path) === link.href) {
      index = i;
      break;
    } else if (
      decodeURIComponent(route.path).includes(link.href) ||
      (link.subpages &&
        link.subpages.some((subpage) => decodeURIComponent(route.path).includes(subpage)))
    ) {
      index = i;
      subpageSelected.value = true;
      break;
    }
  }

  activeIndex.value = index;

  if (activeIndex.value !== -1) {
    startAnimation();
    nextTick(() => {
      const activeTab = tabLinkElements.value[activeIndex.value]?.$el;
      if (activeTab) {
        activeTab.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
          inline: "center",
        });
      }
    });
  } else {
    sliderLeft.value = 0;
    sliderRight.value = 0;
  }
}

function startAnimation() {
  const el = tabLinkElements.value[activeIndex.value]?.$el;

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
<style scoped>
.navtabs-transition {
  transition:
    all 150ms cubic-bezier(0.4, 0, 0.2, 1) 0s,
    opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}
</style>
