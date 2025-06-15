<template>
  <div>
    <div
      class="card-shadow experimental-styles-within relative flex w-fit overflow-x-auto rounded-full bg-button-bg p-1 text-sm font-bold"
    >
      <button
        v-for="(tab, index) in tabs"
        :key="tab"
        ref="tabElements"
        class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full bg-transparent"
        :class="{
          'text-button-textSelected': activeTabIndex === index,
          'text-contrast': activeTabIndex !== index,
        }"
        @click="setActiveTab(index)"
      >
        <span class="text-nowrap font-bold text-center mx-auto">{{ getTabLabel(tab) }}</span>
      </button>

      <div
        :class="`tabs-transition pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1 bg-button-bgSelected`"
        :style="{
          left: sliderLeftPx,
          top: sliderTopPx,
          right: sliderRightPx,
          bottom: sliderBottomPx,
          opacity:
            sliderLeft === 4 && sliderLeft === sliderRight ? 0 : activeTabIndex === -1 ? 0 : 1,
        }"
        aria-hidden="true"
      />
    </div>

    <!-- Tab Content -->
    <div class="tab-content mt-4">
      <template v-for="(tab, index) in tabs" :key="tab">
        <div v-show="activeTabIndex === index" class="tab-panel">
          <slot :name="tab" />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'

interface Props {
  tabs: string[]
  formatFunction?: (tab: string) => string
  defaultTab?: string
}

const props = withDefaults(defineProps<Props>(), {
  formatFunction: undefined,
  defaultTab: undefined,
})

const activeTabIndex = ref(0)
const tabElements = ref<HTMLElement[]>([])

const sliderLeft = ref(4)
const sliderTop = ref(4)
const sliderRight = ref(4)
const sliderBottom = ref(4)

const sliderLeftPx = computed(() => `${sliderLeft.value}px`)
const sliderTopPx = computed(() => `${sliderTop.value}px`)
const sliderRightPx = computed(() => `${sliderRight.value}px`)
const sliderBottomPx = computed(() => `${sliderBottom.value}px`)

function getTabLabel(tab: string): string {
  return props.formatFunction ? props.formatFunction(tab) : tab
}

function setActiveTab(index: number) {
  activeTabIndex.value = index
  updateSliderPosition()
}

function updateSliderPosition() {
  nextTick(() => {
    const el = tabElements.value[activeTabIndex.value]

    if (!el || !el.offsetParent) return

    const parent = el.offsetParent as HTMLElement

    const newValues = {
      left: el.offsetLeft,
      top: el.offsetTop,
      right: parent.offsetWidth - el.offsetLeft - el.offsetWidth,
      bottom: parent.offsetHeight - el.offsetTop - el.offsetHeight,
    }

    if (sliderLeft.value === 4 && sliderRight.value === 4) {
      // Initial position
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
  })
}

onMounted(() => {
  if (props.defaultTab) {
    const defaultIndex = props.tabs.indexOf(props.defaultTab)
    if (defaultIndex !== -1) {
      activeTabIndex.value = defaultIndex
    }
  }
  updateSliderPosition()
})

watch(activeTabIndex, () => {
  updateSliderPosition()
})
</script>

<style scoped>
.tabs-transition {
  transition:
    all 150ms cubic-bezier(0.4, 0, 0.2, 1),
    opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}

.card-shadow {
  box-shadow: var(--shadow-card);
}

.tab-content {
  min-height: 200px;
}
</style>
