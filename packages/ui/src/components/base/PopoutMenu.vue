<template>
  <Dropdown theme="ribbit-popout" no-auto-focus @hide="focusTrigger" @apply-show="focusMenuChild">
    <button ref="trigger">
      <slot></slot>
    </button>
    <template #popper="{ hide }">
      <button class="dummy-button" @focusin="hideAndFocusTrigger(hide)"></button>
      <div ref="menu" class="contents">
        <slot name="menu"> </slot>
      </div>
      <button class="dummy-button" @focusin="hideAndFocusTrigger(hide)"></button>
    </template>
  </Dropdown>
</template>

<script setup>
import { Dropdown } from 'floating-vue'
import { ref } from 'vue'

const trigger = ref()
const menu = ref()

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
  console.log(trigger.value)
  trigger.value.focus()
}

defineOptions({
  inheritAttrs: false,
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
