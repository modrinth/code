<template>
  <div
    v-if="state === 'Installing'"
    class="flex w-fit flex-row items-center rounded-full bg-orange px-2 py-0.5 text-sm font-bold text-black"
  >
    <svg
      class="mr-2 h-3 w-3 animate-spin"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      ></circle>
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      ></path>
    </svg>
    Installing
  </div>
  <div
    v-if="state === 'Failed'"
    class="flex w-fit flex-row items-center rounded-full bg-red px-2 py-0.5 text-sm font-bold text-white"
  >
    Failed to Install
  </div>
</template>

<script lang="ts">
import { defineComponent, type PropType } from "vue";

export type StatusState = "Installing" | "Failed";

export default defineComponent({
  name: "StatusIndicator",
  props: {
    state: {
      type: String as PropType<StatusState>,
      required: true,
    },
  },
});
</script>
