<script setup>
import { Button, FileInput, TrashIcon, Avatar, UploadIcon, SaveIcon, ConfirmModal } from 'omorphia'

const {
  organization,
  refresh: refreshOrganization,
  hasPermission,
  deleteIcon,
  patchIcon,
  patchOrganization,
} = inject('organizationContext')

const icon = ref(null)
const deletedIcon = ref(false)
const previewImage = ref(null)

const name = ref(organization.value.name)
const slug = ref(organization.value.slug)

const summary = ref(organization.value.description)

const patchData = computed(() => {
  const data = {}
  if (name.value !== organization.value.name) {
    data.name = name.value
  }
  if (slug.value !== organization.value.slug) {
    data.slug = slug.value
  }
  if (summary.value !== organization.value.description) {
    data.description = summary.value
  }
  return data
})

const hasChanges = computed(() => {
  return Object.keys(patchData.value).length > 0 || deletedIcon.value || icon.value
})

const markIconForDeletion = () => {
  deletedIcon.value = true
  icon.value = null
  previewImage.value = null
}

const showPreviewImage = (files) => {
  const reader = new FileReader()

  icon.value = files[0]
  deletedIcon.value = false

  reader.readAsDataURL(icon.value)
  reader.onload = (event) => {
    previewImage.value = event.target.result
  }
}

const orgId = useRouteId()

const onSaveChanges = useClientTry(async () => {
  if (hasChanges.value) {
    await patchOrganization(orgId, patchData.value)
  }

  if (deletedIcon.value) {
    await deleteIcon()
    deletedIcon.value = false
  } else if (icon.value) {
    await patchIcon(icon.value)
    icon.value = null
  }

  await refreshOrganization()

  addNotification({
    group: 'main',
    title: 'Organization updated',
    text: 'Your organization has been updated.',
    type: 'success',
  })
})

const onDeleteOrganization = useClientTry(async () => {
  await useBaseFetch(`organization/${orgId}`, {
    method: 'DELETE',
    apiVersion: 3,
  })

  addNotification({
    group: 'main',
    title: 'Organization deleted',
    text: 'Your organization has been deleted.',
    type: 'success',
  })

  await navigateTo('/dashboard/organizations')
})
</script>

<template>
  <div class="normal-page__content">
    <ConfirmModal
      ref="modal_deletion"
      :title="`Are you sure you want to delete ${organization.name}?`"
      description="This will delete this organization forever (like *forever* ever)."
      :has-to-type="true"
      proceed-label="Delete"
      :confirmation-text="organization.name"
      @proceed="onDeleteOrganization"
    />
    <div class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Organization information</span>
        </h3>
      </div>
      <label for="project-icon">
        <span class="label__title">Icon</span>
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
            prompt="Upload icon"
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
            Remove icon
          </Button>
        </div>
      </div>

      <label for="project-name">
        <span class="label__title">Name</span>
      </label>
      <input
        id="project-name"
        v-model="name"
        maxlength="2048"
        type="text"
        :disabled="!hasPermission"
      />

      <label for="project-slug">
        <span class="label__title">URL</span>
      </label>
      <div class="text-input-wrapper">
        <div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
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
        <span class="label__title">Summary</span>
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
          Save changes
        </Button>
      </div>
    </div>
    <div class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Delete organization</span>
        </h3>
      </div>
      <p>
        Deleting your organization will transfer all of its projects to the organization owner. This
        action cannot be undone.
      </p>
      <Button color="danger" @click="() => $refs.modal_deletion.show()">
        <TrashIcon />
        Delete organization
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
