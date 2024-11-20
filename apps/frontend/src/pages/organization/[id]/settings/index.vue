<script setup>
import { Button, FileInput, Avatar, ConfirmModal } from "@modrinth/ui";
import { UploadIcon, SaveIcon, TrashIcon } from "@modrinth/assets";

const {
  organization,
  refresh: refreshOrganization,
  hasPermission,
  deleteIcon,
  patchIcon,
  patchOrganization,
} = inject("organizationContext");

const icon = ref(null);
const deletedIcon = ref(false);
const previewImage = ref(null);

const name = ref(organization.value.name);
const slug = ref(organization.value.slug);

const summary = ref(organization.value.description);

const patchData = computed(() => {
  const data = {};
  if (name.value !== organization.value.name) {
    data.name = name.value;
  }
  if (slug.value !== organization.value.slug) {
    data.slug = slug.value;
  }
  if (summary.value !== organization.value.description) {
    data.description = summary.value;
  }
  return data;
});

const hasChanges = computed(() => {
  return Object.keys(patchData.value).length > 0 || deletedIcon.value || icon.value;
});

const markIconForDeletion = () => {
  deletedIcon.value = true;
  icon.value = null;
  previewImage.value = null;
};

const showPreviewImage = (files) => {
  const reader = new FileReader();

  icon.value = files[0];
  deletedIcon.value = false;

  reader.readAsDataURL(icon.value);
  reader.onload = (event) => {
    previewImage.value = event.target.result;
  };
};

const orgId = useRouteId();

const onSaveChanges = useClientTry(async () => {
  if (hasChanges.value) {
    await patchOrganization(orgId, patchData.value);
  }

  if (deletedIcon.value) {
    await deleteIcon();
    deletedIcon.value = false;
  } else if (icon.value) {
    await patchIcon(icon.value);
    icon.value = null;
  }

  await refreshOrganization();

  addNotification({
    group: "main",
    title: "团队更新",
    text: "您的团队已更新。",
    type: "success",
  });
});

const onDeleteOrganization = useClientTry(async () => {
  await useBaseFetch(`organization/${orgId}`, {
    method: "DELETE",
    apiVersion: 3,
  });

  addNotification({
    group: "main",
    title: "团队删除",
    text: "您的团队已删除.",
    type: "success",
  });

  await navigateTo("/dashboard/organizations");
});
</script>

<template>
  <div class="normal-page__content">
    <ConfirmModal
      ref="modal_deletion"
      :title="`您确定要删除 ${organization.name} 吗?`"
      description="删除后无法撤销该操作."
      :has-to-type="true"
      proceed-label="删除"
      :confirmation-text="organization.name"
      @proceed="onDeleteOrganization"
    />
    <div class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">团队信息</span>
        </h3>
      </div>
      <label for="project-icon">
        <span class="label__title">图标</span>
      </label>
      <div class="input-group">
        <Avatar
          :src="deletedIcon ? null : previewImage ? previewImage : organization.icon_url"
          :alt="organization.name"
          size="md"
          class="project__icon"
        />
        <div class="input-stack">
          <FileInput
            id="project-icon"
            :max-size="262144"
            :show-icon="true"
            accept="image/png,image/jpeg,image/gif,image/webp"
            class="btn"
            prompt="上传图标"
            :disabled="!hasPermission"
            @change="showPreviewImage"
          >
            <UploadIcon />
          </FileInput>
          <Button
            v-if="!deletedIcon && (previewImage || organization.icon_url)"
            :disabled="!hasPermission"
            @click="markIconForDeletion"
          >
            <TrashIcon />
            移除图标
          </Button>
        </div>
      </div>

      <label for="project-name">
        <span class="label__title">名称</span>
      </label>
      <input
        id="project-name"
        v-model="name"
        maxlength="2048"
        type="text"
        :disabled="!hasPermission"
      />

      <label for="project-slug">
        <span class="label__title">主页</span>
      </label>
      <div class="text-input-wrapper">
        <div class="text-input-wrapper__before">https://bbsmc.net/organization/</div>
        <input
          id="project-slug"
          v-model="slug"
          type="text"
          maxlength="64"
          autocomplete="off"
          :disabled="!hasPermission"
        />
      </div>

      <label for="project-summary">
        <span class="label__title">介绍</span>
      </label>
      <div class="textarea-wrapper summary-input">
        <textarea
          id="project-summary"
          v-model="summary"
          maxlength="256"
          :disabled="!hasPermission"
        />
      </div>
      <div class="button-group">
        <Button color="primary" :disabled="!hasChanges" @click="onSaveChanges">
          <SaveIcon />
          保存修改
        </Button>
      </div>
    </div>
    <div class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">删除团队</span>
        </h3>
      </div>
      <p>
        删除您的团队会将其所有资源转移给团队所有者。此操作无法撤消。
      </p>
      <Button color="danger" @click="() => $refs.modal_deletion.show()">
        <TrashIcon />
        删除团队
      </Button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.summary-input {
  min-height: 8rem;
  max-width: 24rem;
}
</style>
