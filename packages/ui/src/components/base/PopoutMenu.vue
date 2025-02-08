<template>
  <Dropdown
    ref="dropdown"
    no-auto-focus
    :aria-id="dropdownId || null"
    placement="bottom-end"
    :class="dropdownClass"
    @apply-hide="focusTrigger"
    @apply-show="focusMenuChild"
  >
    <button ref="trigger" v-bind="$attrs" v-tooltip="tooltip">
      <slot></slot>
    </button>
    <template #popper="{ hide: hideFunction }">
      <button class="dummy-button" @focusin="hideAndFocusTrigger(hideFunction)"></button>
      <div ref="menu" class="contents">
        <slot name="menu"> </slot>
      </div>
      <button class="dummy-button" @focusin="hideAndFocusTrigger(hideFunction)"></button>
    </template>
  </Dropdown>
</template>

<script setup>
import { Dropdown } from 'floating-vue'
import { ref } from 'vue'

const trigger = ref()
const menu = ref()
const dropdown = ref()

defineProps({
  dropdownId: {
    type: String,
    default: null,
    required: false,
  },
  dropdownClass: {
    type: String,
    default: null,
    required: false,
  },
  tooltip: {
    type: String,
    default: null,
    required: false,
  },
})

function focusMenuChild() {
  setTimeout(() => {
    if (menu.value && menu.value.children && menu.value.children.length > 0) {
      menu.value.children[0].focus()
    }
  }, 50)
}

function hideAndFocusTrigger(hide) {
  hide()
  focusTrigger()
}

function focusTrigger() {
  trigger.value.focus()
}

defineOptions({
  inheritAttrs: false,
})

function hide() {
  dropdown.value.hide()
}

function show() {
  dropdown.value.show()
}

defineExpose({
  show,
  hide,
})
</script>
<style scoped>
.dummy-button {
  position: absolute;
  width: 0;
  height: 0;
  margin: 0;
  padding: 0;
  border: none;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
  outline: none;
}
</style>
