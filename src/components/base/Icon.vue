<script setup>
import { defineAsyncComponent } from "vue";
import DefaultIcon from "@/assets/icons/unknown.svg";

const props = defineProps({
  icon: { type: String, required: true },
  rotate: { type: Number, required: false, default: 0 },
  OMORPHIA_ICON_COMPONENT_IDENTIFIER: { type: Boolean, default: true },
});

const IconSvg = defineAsyncComponent(() =>
  import(
    /* webpackChunkName: "icons" */
    /* webpackMode: "lazy-once" */
    `../../assets/icons/${props.icon}.svg`
  )
);
</script>

<template>
  <Suspense>
    <IconSvg
      class="omorphia__icon"
      viewBox="0 0 24 24"
      :style="{ transform: `rotate(${rotate}deg)` }"
    />
    <template #fallback>
      <DefaultIcon class="omorphia__icon" viewBox="0 0 24 24" />
    </template>
  </Suspense>
</template>

<style lang="scss" scoped>
.omorphia__icon {
  transition: transform 0.25s ease-in-out;
}
</style>
