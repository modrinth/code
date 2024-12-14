<template>
  <div class="scrollable-pane-wrapper">
    <div
      class="wrapper-wrapper"
      :class="{
        'top-fade': !scrollableAtTop && !props.disableScrolling,
        'bottom-fade': !scrollableAtBottom && !props.disableScrolling,
      }"
    >
      <div ref="scrollablePane" class="scrollable-pane" @scroll="onScroll">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const props = withDefaults(
  defineProps<{
    disableScrolling?: boolean
  }>(),
  {
    disableScrolling: false,
  },
)

const scrollableAtTop = ref(true)
const scrollableAtBottom = ref(false)
const scrollablePane = ref(null)
let resizeObserver
onMounted(() => {
  resizeObserver = new ResizeObserver(function () {
    if (scrollablePane.value) {
      updateFade(
        scrollablePane.value.scrollTop,
        scrollablePane.value.offsetHeight,
        scrollablePane.value.scrollHeight,
      )
    }
  })
  resizeObserver.observe(scrollablePane.value)
})
onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})
function updateFade(scrollTop, offsetHeight, scrollHeight) {
  scrollableAtBottom.value = Math.ceil(scrollTop + offsetHeight) >= scrollHeight
  scrollableAtTop.value = scrollTop <= 0
}
function onScroll({ target: { scrollTop, offsetHeight, scrollHeight } }) {
  updateFade(scrollTop, offsetHeight, scrollHeight)
}
</script>

<style lang="scss" scoped>
.scrollable-pane-wrapper {
  display: flex;
  flex-direction: column;
  position: relative;
}

.wrapper-wrapper {
  flex-grow: 1;
  display: flex;
  overflow: hidden;
  position: relative;

  --_fade-height: 4rem;

  &.top-fade {
    mask-image: linear-gradient(transparent, rgb(0 0 0 / 100%) var(--_fade-height));
  }

  &.bottom-fade {
    mask-image: linear-gradient(
      rgb(0 0 0 / 100%) calc(100% - var(--_fade-height)),
      transparent 100%
    );
  }

  &.top-fade.bottom-fade {
    mask-image: linear-gradient(
      transparent,
      rgb(0 0 0 / 100%) var(--_fade-height),
      rgb(0 0 0 / 100%) calc(100% - var(--_fade-height)),
      transparent 100%
    );
  }
}
.scrollable-pane {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  height: 100%;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}
</style>
