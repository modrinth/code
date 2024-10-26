<template>
  <NewModal ref="modal" :header="`Moving ${item?.name}`">
    <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
      <div class="flex flex-col gap-2">
        <input
          v-model="destination"
          autofocus
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          placeholder="e.g. mods/modname"
          required
        />
      </div>
      <div class="flex justify-start gap-4">
        <ButtonStyled color="brand">
          <button type="submit">
            <ArrowBigUpDashIcon class="h-5 w-5" />
            Move
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="hide">
            <XIcon class="h-5 w-5" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </form>
  </NewModal>
</template>

<script setup lang="ts">
import { ArrowBigUpDashIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";

const props = defineProps<{
  item: { name: string } | null;
  currentPath: string;
}>();

const emit = defineEmits<{
  (e: "move", destination: string): void;
}>();

const modal = ref<typeof NewModal>();
const destination = ref("");

const handleSubmit = () => {
  emit("move", destination.value);
  hide();
};

const show = () => {
  destination.value = props.currentPath;
  modal.value?.show();
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
