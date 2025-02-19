<template>
  <div
    aria-hidden="true"
    class="sticky top-12 z-20 flex h-8 w-full select-none flex-row items-center border-0 border-b border-solid border-bg-raised bg-bg px-3 text-xs font-bold uppercase"
  >
    <div class="min-w-[48px]"></div>
    <button
      class="flex h-full w-full appearance-none items-center gap-1 bg-transparent text-left hover:text-brand"
      @click="$emit('sort', 'name')"
    >
      <span>Name</span>
      <ChevronUpIcon v-if="sortField === 'name' && !sortDesc" class="h-3 w-3" aria-hidden="true" />
      <ChevronDownIcon v-if="sortField === 'name' && sortDesc" class="h-3 w-3" aria-hidden="true" />
    </button>
    <div class="flex shrink-0 gap-4 text-right md:gap-12">
      <button
        class="hidden min-w-[160px] appearance-none items-center justify-start gap-1 bg-transparent hover:text-brand md:flex"
        @click="$emit('sort', 'created')"
      >
        <span>Created</span>
        <ChevronUpIcon
          v-if="sortField === 'created' && !sortDesc"
          class="h-3 w-3"
          aria-hidden="true"
        />
        <ChevronDownIcon
          v-if="sortField === 'created' && sortDesc"
          class="h-3 w-3"
          aria-hidden="true"
        />
      </button>
      <button
        class="mr-4 hidden min-w-[160px] appearance-none items-center justify-start gap-1 bg-transparent hover:text-brand md:flex"
        @click="$emit('sort', 'modified')"
      >
        <span>Modified</span>
        <ChevronUpIcon
          v-if="sortField === 'modified' && !sortDesc"
          class="h-3 w-3"
          aria-hidden="true"
        />
        <ChevronDownIcon
          v-if="sortField === 'modified' && sortDesc"
          class="h-3 w-3"
          aria-hidden="true"
        />
      </button>
      <div class="min-w-[24px]"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ChevronDownIcon from "./icons/ChevronDownIcon.vue";
import ChevronUpIcon from "./icons/ChevronUpIcon.vue";

defineProps<{
  sortField: string;
  sortDesc: boolean;
}>();

defineEmits<{
  (e: "sort", field: string): void;
}>();
</script>
