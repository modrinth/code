<script setup>
import Button from "@/components/base/Button.vue";
import Icon from "@/components/base/Icon.vue";

import { reactive } from "vue";

const props = defineProps({
  collapsible: {
    type: Boolean,
    default: false,
  },
  defaultCollapsed: {
    type: Boolean,
    default: false,
  },
  noAutoBody: {
    type: Boolean,
    default: false,
  },
});

const state = reactive({
  collapsed: props.defaultCollapsed,
});

function toggleCollapsed() {
  state.collapsed = !state.collapsed;
}
</script>

<template>
  <div
    class="omorphia__card standard-body padding-xl bg-raised radius-lg margin-bottom-md"
    :class="{ 'auto-body': !noAutoBody }"
  >
    <div v-if="!!$slots.header || collapsible" class="header">
      <slot name="header"></slot>
      <div
        v-if="collapsible"
        class="button-group margin-left-auto margin-right-0"
      >
        <Button :action="toggleCollapsed">
          <Icon icon="dropdown" :rotate="state.collapsed ? 0 : 180" />
        </Button>
      </div>
    </div>
    <slot v-if="!state.collapsed" />
  </div>
</template>

<style lang="scss" scoped>
.omorphia__card {
}

.header {
  display: flex;

  :deep(h1, h2, h3, h4) {
    margin-block: 0;
  }

  &:not(:last-child) {
    margin-bottom: var(--gap-lg);
  }
}
</style>
