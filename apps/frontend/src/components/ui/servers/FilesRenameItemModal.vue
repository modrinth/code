<template>
  <NewModal ref="modal" :header="`Renaming ${item?.type}`">
    <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
      <div class="flex flex-col gap-2">
        <div class="font-semibold text-contrast">Name</div>
        <input
          ref="renameInput"
          v-model="itemName"
          autofocus
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          required
        />
        <div v-if="submitted && error" class="text-red">{{ error }}</div>
      </div>
      <div class="flex justify-start gap-4">
        <ButtonStyled color="brand">
          <button :disabled="!!error" type="submit">
            <EditIcon class="h-5 w-5" />
            Rename
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
import { EditIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { ref, computed, nextTick } from "vue";

const props = defineProps<{
  item: { name: string; type: string } | null;
}>();

const emit = defineEmits<{
  (e: "rename", newName: string): void;
}>();

const modal = ref<typeof NewModal>();
const renameInput = ref<HTMLInputElement | null>(null);
const itemName = ref("");
const submitted = ref(false);

const error = computed(() => {
  if (!itemName.value) {
    return "Name is required.";
  }
  if (props.item?.type === "file") {
    const validPattern = /^[a-zA-Z0-9-_.\s]+$/;
    if (!validPattern.test(itemName.value)) {
      return "Name must contain only alphanumeric characters, dashes, underscores, dots, or spaces.";
    }
  } else {
    const validPattern = /^[a-zA-Z0-9-_\s]+$/;
    if (!validPattern.test(itemName.value)) {
      return "Name must contain only alphanumeric characters, dashes, underscores, or spaces.";
    }
  }
  return "";
});

const handleSubmit = () => {
  submitted.value = true;
  if (!error.value) {
    emit("rename", itemName.value);
    hide();
  }
};

const show = (item: { name: string; type: string }) => {
  itemName.value = item.name;
  submitted.value = false;
  modal.value?.show();
  nextTick(() => {
    setTimeout(() => {
      renameInput.value?.focus();
    }, 100);
  });
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
