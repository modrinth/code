<template>
  <NewModal ref="modal" header="Creating an organization">
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
          :placeholder="`Enter organization name...`"
          autocomplete="off"
          @input="updateSlug()"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="slug">
          <span class="text-lg font-semibold text-contrast">
            URL
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <div class="text-input-wrapper">
          <div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
          <input
            id="slug"
            v-model="slug"
            type="text"
            maxlength="64"
            autocomplete="off"
            @input="manualSlug = true"
          />
        </div>
      </div>
      <div class="flex flex-col gap-2">
        <label for="additional-information" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Summary
            <span class="text-brand-red">*</span>
          </span>
          <span>A sentence or two that describes your organization.</span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="additional-information" v-model="description" maxlength="256" />
        </div>
      </div>
      <p class="m-0 max-w-[30rem]">
        You will be the owner of this organization, but you can invite other members and transfer
        ownership at any time.
      </p>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="createOrganization">
            <PlusIcon aria-hidden="true" />
            Create organization
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
import { XIcon, PlusIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";

const router = useNativeRouter();

const name = ref("");
const slug = ref("");
const description = ref("");
const manualSlug = ref(false);

const modal = ref();

async function createOrganization() {
  startLoading();
  try {
    const value = {
      name: name.value.trim(),
      description: description.value.trim(),
      slug: slug.value.trim().replace(/ +/g, ""),
    };

    const result = await useBaseFetch("organization", {
      method: "POST",
      body: JSON.stringify(value),
      apiVersion: 3,
    });

    modal.value.hide();

    await router.push(`/organization/${result.slug}`);
  } catch (err) {
    console.error(err);
    addNotification({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
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

function updateSlug() {
  if (!manualSlug.value) {
    slug.value = name.value
      .trim()
      .toLowerCase()
      .replaceAll(" ", "-")
      .replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, "")
      .replaceAll(/--+/gm, "-");
  }
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
