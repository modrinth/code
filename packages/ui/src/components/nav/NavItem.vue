<script setup>
import Button from '../base/Button.vue'

defineProps({
  link: {
    type: String,
    default: null,
  },
  external: {
    type: Boolean,
    default: false,
  },
  action: {
    type: Function,
    default: null,
  },
  selected: {
    type: Boolean,
    default: false,
  },
  label: {
    type: String,
    required: true,
  },
  icon: {
    type: String,
    default: null,
  },
})
</script>

<template>
  <Button
    :link="link"
    :external="external"
    :action="action"
    design="nav"
    class="quiet-disabled"
    :class="{
      selected: selected,
    }"
    :disabled="selected"
    :navlabel="label"
  >
    <slot />
    {{ label }}
  </Button>
</template>

<style lang="scss" scoped></style>
