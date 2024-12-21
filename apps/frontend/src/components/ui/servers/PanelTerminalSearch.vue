<template>
  <div
    :style="{
      width: open ? '12rem' : '2.75rem',
    }"
    :class="[
      'cm-content grid h-full w-11 grid-cols-1 grid-rows-1 rounded-md !p-0 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg',
      open ? 'act-like-input' : 'act-like-button',
    ]"
  >
    <Transition name="fade" class="col-start-1 row-start-1">
      <button
        v-if="!open"
        class="flex h-full w-full items-center justify-center !bg-transparent"
        @click="openSearch"
      >
        <SearchIcon class="h-4 w-4 text-black/50" />
      </button>
    </Transition>
    <Transition name="fade" class="col-start-1 row-start-1">
      <input
        v-if="open"
        ref="inputRef"
        class="!h-10 w-full !border-none !bg-transparent !shadow-none !outline-none ring-brand focus:ring-4"
        placeholder="Search..."
        type="text"
        @input="onChange"
        @focusout="closeSearch"
    /></Transition>
  </div>
</template>

<script setup lang="ts">
import { SearchIcon } from "@modrinth/assets";

const emit = defineEmits<{
  (e: "search", value: string): void;
}>();

const open = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

const openSearch = async () => {
  open.value = true;
  await nextTick();
  inputRef.value?.focus();
};

const closeSearch = () => {
  if (inputRef.value?.value === "") {
    open.value = false;
  }
};

const onChange = (event: Event) => {
  const val = (event.target as HTMLInputElement).value;
  emit("search", val);
};
</script>

<style scoped lang="scss">
.cm-content {
  transition:
    transform 0.2s ease,
    width 0.2s ease;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.act-like-button {
  &:hover {
    transform: scale(1.1);
  }

  &:active {
    transform: scale(0.95);
  }
}
</style>
