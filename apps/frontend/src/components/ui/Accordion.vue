<template>
  <div>
    <ButtonStyled v-if="!!slots.title" :type="type">
      <button class="!w-full" @click="() => (isOpen ? close() : open())">
        <slot name="title" /><DropdownIcon
          class="ml-auto size-5 text-contrast transition-transform duration-300"
          :class="{ 'rotate-180': isOpen }"
        />
      </button>
    </ButtonStyled>
    <div class="accordion-content" :class="{ open: isOpen }">
      <div>
        <div :class="{ 'mt-2': !!slots.title }" v-bind="$attrs" :inert="!isOpen">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DropdownIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = withDefaults(
  defineProps<{
    openByDefault?: boolean;
    type?: "standard" | "outlined" | "transparent";
  }>(),
  {
    type: "standard",
    openByDefault: false,
  },
);

const isOpen = ref(props.openByDefault);
const emit = defineEmits(["onOpen", "onClose"]);

const slots = useSlots();

function open() {
  isOpen.value = true;
  emit("onOpen");
}
function close() {
  isOpen.value = false;
  emit("onClose");
}

defineExpose({
  open,
  close,
  isOpen,
});

defineOptions({
  inheritAttrs: false,
});
</script>
<style scoped>
.accordion-content {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.3s ease-in-out;
  animation: height-animate 500ms ease-in-out both;
  content-visibility: auto;
  animation-composition: replace;
}

@media (prefers-reduced-motion) {
  .accordion-content {
    transition: none !important;
  }
}

.accordion-content.open {
  grid-template-rows: 1fr;
}

.accordion-content > div {
  overflow: hidden;
}

@keyframes height-animate {
  from {
    block-size: initial;
  }
  to {
    block-size: auto;
  }
}
</style>
