<template>
  <NewModal ref="modal" header="Creating a collection">
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2">
        <label for="name">
          <span class="text-lg font-semibold text-contrast">
            Name
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="name"
          v-model="name"
          type="text"
          maxlength="64"
          :placeholder="`Enter collection name...`"
          autocomplete="off"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="additional-information" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Summary </span>
          <span>A sentence or two that describes your collection.</span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="additional-information" v-model="description" maxlength="256" />
        </div>
      </div>
      <p class="m-0 max-w-[30rem]">
        Your new collection will be created as a public collection with
        {{ projectIds.length > 0 ? projectIds.length : "no" }}
        {{ projectIds.length !== 1 ? "projects" : "project" }}.
      </p>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="create">
            <PlusIcon aria-hidden="true" />
            Create collection
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="modal.hide()">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>
<script setup>
import { PlusIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";

const router = useNativeRouter();

const name = ref("");
const description = ref("");

const modal = ref();

const props = defineProps({
  projectIds: {
    type: Array,
    default() {
      return [];
    },
  },
});

async function create() {
  startLoading();
  try {
    const result = await useBaseFetch("collection", {
      method: "POST",
      body: {
        name: name.value.trim(),
        description: description.value.trim() || undefined,
        projects: props.projectIds,
      },
      apiVersion: 3,
    });

    await initUserCollections();

    modal.value.hide();
    await router.push(`/collection/${result.id}`);
  } catch (err) {
    addNotification({
      group: "main",
      title: "An error occurred",
      text: err?.data?.description || err?.message || err,
      type: "error",
    });
  }
  stopLoading();
}
function show(event) {
  name.value = "";
  description.value = "";
  modal.value.show(event);
}

defineExpose({
  show,
});
</script>

<style scoped lang="scss">
.modal-creation {
  input {
    width: 20rem;
    max-width: 100%;
  }

  .text-input-wrapper {
    width: 100%;
  }

  textarea {
    min-height: 5rem;
  }

  .input-group {
    margin-top: var(--gap-md);
  }
}
</style>
