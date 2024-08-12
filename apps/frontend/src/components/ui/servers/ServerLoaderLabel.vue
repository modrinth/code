<template>
  <div v-if="loader" class="flex flex-row items-center gap-4">
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <UiServersLoaderIcon :loader="loader" />
      <span class="text-sm font-semibold capitalize">
        {{ loader }} <span v-if="loaderVersion">{{ loaderVersion }}</span>
      </span>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "LoaderInfo",
  props: {
    loader: {
      type: String,
      required: true,
    },
    loaderVersion: {
      type: String,
      required: true,
    },
  },
});
</script>
