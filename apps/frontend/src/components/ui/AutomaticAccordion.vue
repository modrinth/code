<template>
  <div class="accordion-wrapper" :class="{ 'has-content': hasContent }">
    <div class="accordion-content">
      <div>
        <div v-bind="$attrs" ref="slotContainer" class="content-container">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});

const slotContainer = ref();

const hasContent = ref(false);

const mutationObserver = ref<MutationObserver | null>(null);

function updateContent() {
  if (!slotContainer.value) return false;

  hasContent.value = slotContainer.value ? slotContainer.value.children.length > 0 : false;
}

onMounted(() => {
  mutationObserver.value = new MutationObserver(updateContent);

  mutationObserver.value.observe(slotContainer.value, {
    childList: true,
  });

  updateContent();
});

onUnmounted(() => {
  if (mutationObserver.value) {
    mutationObserver.value.disconnect();
  }
});
</script>
<style scoped>
.accordion-content {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.3s ease-in-out;
}

@media (prefers-reduced-motion) {
  .accordion-content {
    transition: none !important;
  }
}

.has-content .accordion-content {
  grid-template-rows: 1fr;
}

.accordion-content > div {
  overflow: hidden;
}

.accordion-wrapper.has-content {
  display: contents;
}
</style>
