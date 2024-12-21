<template>
  <NewModal ref="modal" :header="`Moving ${item?.name}`">
    <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
      <div class="flex flex-col gap-2">
        <input
          ref="destinationInput"
          v-model="destination"
          autofocus
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          placeholder="e.g. /mods/modname"
          required
        />
      </div>
      <div class="flex items-center gap-2 text-nowrap">
        New location:
        <div class="w-full rounded-lg bg-table-alternateRow p-2 font-bold text-contrast">
          <span class="text-secondary">/root</span>{{ newpath }}
        </div>
      </div>
      <div class="flex justify-start gap-4">
        <ButtonStyled color="brand">
          <button type="submit">
            <ArrowBigUpDashIcon class="h-5 w-5" />
            Move
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
import { ArrowBigUpDashIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { ref, nextTick, computed } from "vue";

const destinationInput = ref<HTMLInputElement | null>(null);

const props = defineProps<{
  item: { name: string } | null;
  currentPath: string;
}>();

const emit = defineEmits<{
  (e: "move", destination: string): void;
}>();

const modal = ref<typeof NewModal>();
const destination = ref("");
const newpath = computed(() => {
  const path = destination.value.replace("//", "/");
  return path.startsWith("/") ? path : `/${path}`;
});

const handleSubmit = () => {
  emit("move", newpath.value);
  hide();
};

const show = () => {
  destination.value = props.currentPath;
  modal.value?.show();
  nextTick(() => {
    setTimeout(() => {
      destinationInput.value?.focus();
    }, 100);
  });
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
