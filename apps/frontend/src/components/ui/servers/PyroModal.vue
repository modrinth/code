<template>
  <div
    class="flex h-full flex-col gap-4 p-6"
    :class="
      'flex h-full flex-col gap-4 p-6' +
      (danger ? ' rounded-2xl border-2 border-solid border-[#FF496E] bg-[#270B11]' : '')
    "
  >
    <div class="flex items-center justify-between gap-4">
      <div class="flex w-full items-center gap-4">
        <UiAvatar
          v-if="data && data.project"
          :src="data.project.icon_url"
          no-shadow
          size="xs"
          alt="Server Icon"
        />
        <div class="text-2xl font-extrabold text-contrast">{{ props.header }}</div>
      </div>
      <button
        :class="
          'h-8 w-8 rounded-full bg-button-bg p-2 text-contrast hover:bg-button-bgActive' +
          (danger ? 'hover:bg-[#ffffff20] [&&]:bg-[#ffffff10]' : '')
        "
        @click="$emit('modal')"
      >
        <XIcon class="h-4 w-4" />
      </button>
    </div>
    <div
      :class="'border-0 border-b border-solid border-button-bg' + (danger ? 'bg-[#FF496E]' : '')"
    ></div>
    <div class="h-full w-full overflow-scroll">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { XIcon } from "@modrinth/assets";

defineEmits(["modal"]);

const props = defineProps<{
  header?: string;
  data?: any;
  danger?: boolean;
}>();
</script>
