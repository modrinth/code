<template>
  <NewModal ref="modal" header="Creating a project">
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
          placeholder="Enter project name..."
          autocomplete="off"
          @input="updatedName()"
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
          <div class="text-input-wrapper__before">https://modrinth.com/project/</div>
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
        <label for="visibility" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Visibility
            <span class="text-brand-red">*</span>
          </span>
          <span> The visibility of your project after it has been approved. </span>
        </label>
        <DropdownSelect
          id="visibility"
          v-model="visibility"
          :options="visibilities"
          :display-name="(x) => x.display"
          name="Visibility"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="additional-information" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            Summary
            <span class="text-brand-red">*</span>
          </span>
          <span> A sentence or two that describes your project. </span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="additional-information" v-model="description" maxlength="256" />
        </div>
      </div>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="createProject">
            <PlusIcon aria-hidden="true" />
            Create project
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="cancel">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup>
import { NewModal, ButtonStyled, DropdownSelect } from "@modrinth/ui";
import { XIcon, PlusIcon } from "@modrinth/assets";

const router = useRouter();
const app = useNuxtApp();

const props = defineProps({
  organizationId: {
    type: String,
    required: false,
    default: null,
  },
});

const modal = ref();

const name = ref("");
const slug = ref("");
const description = ref("");
const manualSlug = ref(false);
const visibilities = ref([
  {
    actual: "approved",
    display: "Public",
  },
  {
    actual: "unlisted",
    display: "Unlisted",
  },
  {
    actual: "private",
    display: "Private",
  },
]);
const visibility = ref({
  actual: "approved",
  display: "Public",
});

const cancel = () => {
  modal.value.hide();
};

async function createProject() {
  startLoading();

  const formData = new FormData();

  const auth = await useAuth();

  const projectData = {
    title: name.value.trim(),
    project_type: "mod",
    slug: slug.value,
    description: description.value.trim(),
    body: "",
    requested_status: visibility.value.actual,
    initial_versions: [],
    team_members: [
      {
        user_id: auth.value.user.id,
        name: auth.value.user.username,
        role: "Owner",
      },
    ],
    categories: [],
    client_side: "required",
    server_side: "required",
    license_id: "LicenseRef-Unknown",
    is_draft: true,
  };

  if (props.organizationId) {
    projectData.organization_id = props.organizationId;
  }

  formData.append("data", JSON.stringify(projectData));

  try {
    await useBaseFetch("project", {
      method: "POST",
      body: formData,
      headers: {
        "Content-Disposition": formData,
      },
    });

    modal.value.hide();
    await router.push({
      name: "type-id",
      params: {
        type: "project",
        id: slug.value,
      },
    });
  } catch (err) {
    app.$notify({
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
  slug.value = "";
  description.value = "";
  manualSlug.value = false;
  modal.value.show(event);
}

defineExpose({
  show,
});

function updatedName() {
  if (!manualSlug.value) {
    slug.value = name.value
      .trim()
      .toLowerCase()
      .replaceAll(" ", "-")
      .replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, "")
      .replaceAll(/--+/gm, "-");
  }
}
</script>
