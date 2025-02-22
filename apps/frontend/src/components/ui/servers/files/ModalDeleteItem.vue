<template>
  <NewModal ref="modal" danger :header="`Deleting ${item?.type}`">
    <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
      <div
        class="relative flex w-full items-center gap-2 rounded-2xl border border-solid border-[#cb224436] bg-[#f57b7b0e] p-6 shadow-md dark:border-0 dark:bg-[#0e0e0ea4]"
      >
        <div
          class="flex h-9 w-9 items-center justify-center rounded-full bg-[#3f1818a4] p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
        >
          <FolderOpenIcon v-if="item?.type === 'directory'" class="h-5 w-5" />
          <FileIcon v-else-if="item?.type === 'file'" class="h-5 w-5" />
        </div>
        <div class="flex flex-col">
          <span class="font-bold group-hover:text-contrast">{{ item?.name }}</span>
          <span
            v-if="item?.type === 'directory'"
            class="text-xs text-secondary group-hover:text-primary"
          >
            {{ item?.count }} items
          </span>
          <span v-else class="text-xs text-secondary group-hover:text-primary">
            {{ ((item?.size ?? 0) / 1024 / 1024).toFixed(2) }} MB
          </span>
        </div>
      </div>
      <div class="flex justify-start gap-4">
        <ButtonStyled color="red">
          <button type="submit">
            <TrashIcon class="h-5 w-5" />
            Delete {{ item?.type }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button type="button" @click="hide">
            <XIcon class="h-5 w-5" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </form>
  </NewModal>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { FileIcon, FolderOpenIcon, TrashIcon, XIcon } from "@modrinth/assets";

defineProps<{
  item: {
    name: string;
    type: string;
    count?: number;
    size?: number;
  } | null;
}>();

const emit = defineEmits<{
  (e: "delete"): void;
}>();

const modal = ref<typeof NewModal>();

const handleSubmit = () => {
  emit("delete");
  hide();
};

const show = () => {
  modal.value?.show();
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
