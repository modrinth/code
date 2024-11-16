<template>
  <div v-bind="$attrs">
    <button
      v-if="!!slots.title"
      @click="() => (isOpen ? close() : open())"
      :class="buttonClass ? buttonClass : ''"
    >
      <slot name="button" :open="isOpen">
        <slot name="title" />
        <DropdownIcon
          class="ml-auto size-5 transition-transform duration-300"
          :class="{ 'rotate-180': isOpen }"
        />
      </slot>
    </button>
    <div class="accordion-content" :class="{ open: isOpen }">
      <div>
        <div :class="contentClass ? contentClass : ''" :inert="!isOpen">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { ref, useSlots } from 'vue'

const props = withDefaults(
  defineProps<{
    openByDefault?: boolean
    type?: 'standard' | 'outlined' | 'transparent'
    buttonClass?: string
    contentClass?: string
  }>(),
  {
    type: 'standard',
    openByDefault: false,
  },
)

const isOpen = ref(props.openByDefault)
const emit = defineEmits(['onOpen', 'onClose'])

const slots = useSlots()

function open() {
  isOpen.value = true
  emit('onOpen')
}
function close() {
  isOpen.value = false
  emit('onClose')
}

defineExpose({
  open,
  close,
  isOpen,
})

defineOptions({
  inheritAttrs: false,
})
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

.accordion-content.open {
  grid-template-rows: 1fr;
}

.accordion-content > div {
  overflow: hidden;
}
</style>
