<template>
  <div>
    <div v-if="loader && loaderVersion" class="h-6 w-0.5 bg-button-border"></div>
    <div v-if="loader && loaderVersion" class="flex flex-row items-center gap-2">
      <LoaderIcon v-if="loader" :loader="loader" />
      <span class="text-sm font-semibold capitalize"> {{ loader }} {{ loaderVersion }} </span>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import LoaderIcon from "./LoaderIcon.vue";

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
