<template>
  <div
    class="flex h-full flex-col gap-4 py-6"
    :class="
      'flex h-full flex-col gap-4 py-6' +
      (danger
        ? ' rounded-2xl border-2 border-solid border-[#cb2245] bg-[#fff5f6] dark:border-[#FF496E] dark:bg-[#270B11]'
        : '')
    "
  >
    <div class="mb-2 flex items-center justify-between gap-4 px-6">
      <div class="flex w-full items-center gap-4">
        <UiServersServerIcon v-if="data" :image="data.image" class="h-12 w-12 rounded-lg" />
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
      class="border-0 border-b border-solid"
      :class="danger ? 'border-[#cb2245] dark:border-[#612d38]' : 'border-divider'"
    ></div>
    <div class="mt-2 h-full w-full overflow-auto px-6">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { XIcon } from "@modrinth/assets";

const emit = defineEmits(["modal"]);

const props = defineProps<{
  header?: string;
  data?: any;
  danger?: boolean;
}>();

const onEscKeyRelease = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    emit("modal");
  }
};

onMounted(() => {
  document.body.addEventListener("keyup", onEscKeyRelease);
});

onBeforeUnmount(() => {
  document.removeEventListener("keyup", onEscKeyRelease);
});
</script>
