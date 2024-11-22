<template>
  <NewModal ref="modal" header="创建资源">
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2">
        <label for="name">
          <span class="text-lg font-semibold text-contrast">
            资源名称
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="name"
          v-model="name"
          type="text"
          maxlength="64"
          placeholder="资源名称..."
          autocomplete="off"
          @input="updatedName()"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="slug" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast">
            标识ID
            <span class="text-brand-red">*</span>
          </span>
          <span>请输入英文名,不可使用空格，特殊符号仅可使用 - 例如 ABC-FWAI-AF</span>
        </label>
        <div class="text-input-wrapper">
          <div class="text-input-wrapper__before">https://bbsmc.net/project/</div>
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
            可见度
            <span class="text-brand-red">*</span>
          </span>
          <span> 您的资源审核完成后的可见性。 </span>
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
            简介
            <span class="text-brand-red">*</span>
          </span>
          <span> 一个简短的资源简介. </span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="additional-information" v-model="description" maxlength="256" />
        </div>
      </div>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="createProject">
            <PlusIcon aria-hidden="true" />
            创建资源
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="cancel">
            <XIcon aria-hidden="true" />
            取消
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
    display: "公开",
  },
  {
    actual: "unlisted",
    display: "不公开",
  },
  {
    actual: "private",
    display: "私有",
  },
]);
const visibility = ref({
  actual: "approved",
  display: "公开",
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
      title: "发生错误",
      text: err.data.description,
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
