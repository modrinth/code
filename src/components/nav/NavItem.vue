<script setup>
import Button from "@/components/base/Button.vue";
import Icon from "@/components/base/Icon.vue";

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
});
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
    <Icon v-if="icon" :icon="icon" />
    {{ label }}
  </Button>
</template>

<style lang="scss" scoped></style>
