<template>
  <nav
    class="card-shadow experimental-styles-within relative flex w-fit overflow-clip rounded-full bg-bg-raised p-1 text-sm font-bold"
  >
    <div
      :class="`navtabs-transition pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1 ${subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected'}`"
      :style="{
        left: sliderLeftPx,
        top: sliderTopPx,
        right: sliderRightPx,
        bottom: sliderBottomPx,
        opacity: sliderLeft === 4 && sliderLeft === sliderRight ? 0 : activeIndex === -1 ? 0 : 1,
      }"
      aria-hidden="true"
    ></div>
    <template
      v-for="(tab, index) in shownTabs"
      :key="index"
    >
      <div ref="tabLinkElements">
        <RouterLink
          v-if="isLinkTab(tab)"
          :to="query ? (tab.href ? `?${query}=${tab.href}` : '?') : tab.href"
          class="button-animation leading-normal text-md font-bold z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          :class="`${activeIndex === index && !subpageSelected ? 'text-button-textSelected' : activeIndex === index && subpageSelected ? 'text-contrast' : 'text-primary'}`"
        >
          <component :is="tab.icon" v-if="tab.icon" class="size-5" />
          <span class="text-nowrap z-[1]">{{ tab.label }}</span>
        </RouterLink>
        <button
          v-else
          class="button-animation leading-normal cursor-pointer text-md font-bold bg-transparent border-0 z-[1] flex flex-row items-center gap-2 px-4 py-2 rounded-full"
          :class="`${activeIndex === index && !subpageSelected ? 'text-button-textSelected' : activeIndex === index && subpageSelected ? 'text-contrast' : 'text-primary  hover:bg-button-bg hover:text-contrast'}`"
          @click="() => {
            tab.action()
            pickLink()
          }"
        >
          <component :is="tab.icon" v-if="tab.icon" class="size-5" />
          <span class="text-nowrap z-[1]">{{ tab.label }}</span>
        </button>
      </div>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { useRoute, RouterLink } from 'vue-router'

const route = useRoute()

interface BaseTab {
  label: string
  shown?: boolean
  icon?: unknown
}

interface LinkTab extends BaseTab {
  href: string | RouteLocationRaw
  subpages?: string[]
}

interface ActionTab extends BaseTab {
  action: () => void
  isSelected: () => boolean
  isSubpageSelected?: () => boolean
}

type Tab = LinkTab | ActionTab

const props = defineProps<{
  tabs: Tab[]
  query?: string
}>()

const sliderLeft = ref(4)
const sliderTop = ref(4)
const sliderRight = ref(4)
const sliderBottom = ref(4)
const activeIndex = ref(-1)
const oldIndex = ref(-1)
const subpageSelected = ref(false)

const shownTabs = computed(() =>
  props.tabs.filter((x) => (x.shown === undefined ? true : x.shown)),
)
const sliderLeftPx = computed(() => `${sliderLeft.value}px`)
const sliderTopPx = computed(() => `${sliderTop.value}px`)
const sliderRightPx = computed(() => `${sliderRight.value}px`)
const sliderBottomPx = computed(() => `${sliderBottom.value}px`)

function isLinkTab(tab: Tab): tab is LinkTab {
  return 'href' in tab
}

function isTabSelected(tab: Tab): boolean {
  if (isLinkTab(tab)) {
    return route.path === (typeof tab.href === 'string' ? tab.href : tab.href.path)
  } else {
    return tab.isSelected()
  }
}

function isSubpageSelected(tab: Tab): boolean {
  if (isLinkTab(tab)) {
    return !!tab.subpages && tab.subpages.some((subpage) => route.path.includes(subpage))
  } else {
    return !!tab.isSubpageSelected?.()
  }
}

function pickLink() {
  let index = -1
  subpageSelected.value = false
  for (let i = shownTabs.value.length - 1; i >= 0; i--) {
    const tab = shownTabs.value[i]

    if (isTabSelected(tab)) {
      index = i
      break
    } else if (isSubpageSelected(tab)) {
      index = i
      subpageSelected.value = true
      break
    }
  }
  activeIndex.value = index

  if (activeIndex.value !== -1) {
    startAnimation()
  } else {
    oldIndex.value = -1
    sliderLeft.value = 0
    sliderRight.value = 0
  }
}

const tabLinkElements = ref()

function startAnimation() {
  const el = tabLinkElements.value[activeIndex.value]

  if (!el || !el.offsetParent) return

  const newValues = {
    left: el.offsetLeft,
    top: el.offsetTop,
    right: el.offsetParent.offsetWidth - el.offsetLeft - el.offsetWidth,
    bottom: el.offsetParent.offsetHeight - el.offsetTop - el.offsetHeight,
  }

  if (sliderLeft.value === 4 && sliderRight.value === 4) {
    sliderLeft.value = newValues.left
    sliderRight.value = newValues.right
    sliderTop.value = newValues.top
    sliderBottom.value = newValues.bottom
  } else {
    const delay = 200

    if (newValues.left < sliderLeft.value) {
      sliderLeft.value = newValues.left
      setTimeout(() => {
        sliderRight.value = newValues.right
      }, delay)
    } else {
      sliderRight.value = newValues.right
      setTimeout(() => {
        sliderLeft.value = newValues.left
      }, delay)
    }

    if (newValues.top < sliderTop.value) {
      sliderTop.value = newValues.top
      setTimeout(() => {
        sliderBottom.value = newValues.bottom
      }, delay)
    } else {
      sliderBottom.value = newValues.bottom
      setTimeout(() => {
        sliderTop.value = newValues.top
      }, delay)
    }
  }
}

onMounted(() => {
  window.addEventListener('resize', pickLink)
  pickLink()
})

onUnmounted(() => {
  window.removeEventListener('resize', pickLink)
})

watch(route, () => {
  pickLink()
})
</script>
<style scoped>
.navtabs-transition {
  /* Delay on opacity is to hide any jankiness as the page loads */
  transition:
    all 150ms cubic-bezier(0.4, 0, 0.2, 1) 0s,
    opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}
</style>
