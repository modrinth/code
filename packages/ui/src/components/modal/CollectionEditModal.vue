<template>
  <NewModal
    ref="collectionEditModal"
    :on-hide="() => onHide"
    :on-show="() => onShow"
    :header="props.header"
  >
    <div class="flex flex-col gap-3">
      <span class="text-lg font-semibold text-contrast"> Preview </span>
      <div
        class="flex flex-row items-center gap-3 overflow-x-hidden rounded-3xl bg-bg p-3 sm:overflow-x-auto"
      >
        <Avatar
          size="md"
          :src="editState.icon_removed ? null : previewImage ? previewImage : editState.icon_url"
        />
        <div class="short flex flex-col justify-between gap-2">
          <div class="flex w-48 flex-col gap-0 sm:w-96 sm:max-w-96">
            <span
              class="line-clamp-2 overflow-ellipsis text-wrap text-xl font-extrabold text-contrast sm:line-clamp-1"
              >{{ editState.name != '' ? editState.name : 'Collection Name' }}</span
            >
            <span class="line-clamp-2 w-full overflow-ellipsis sm:line-clamp-1">{{
              editState.description != '' ? editState.description : 'Collection summary'
            }}</span>
          </div>
          <PopoutMenu class="btn w-fit">
            <EditIcon aria-hidden="true" />
            Edit Icon
            <template #menu>
              <span class="icon-edit-menu h-fit">
                <FileInput
                  id="project-icon"
                  :max-size="262144"
                  :show-icon="true"
                  accept="image/png,image/jpeg,image/gif,image/webp"
                  class="btn btn-transparent upload"
                  style="white-space: nowrap"
                  aria-label="Upload icon"
                  @change="showPreviewImage"
                >
                  <UploadIcon aria-hidden="true" />
                </FileInput>
                <Button
                  v-if="!editState.icon_removed && (previewImage || editState.icon_url)"
                  style="white-space: nowrap"
                  transparent
                  @click="
                    () => {
                      editState.icon_removed = true
                      previewImage = null
                    }
                  "
                >
                  <TrashIcon aria-hidden="true" />
                  Delete Icon
                </Button>
              </span>
            </template>
          </PopoutMenu>
        </div>
      </div>
      <div class="flex flex-col gap-2">
        <label for="name">
          <span class="text-lg font-semibold text-contrast"> Name </span>
        </label>
        <input
          id="name"
          v-model="editState.name"
          type="text"
          maxlength="64"
          :placeholder="`Enter collection name...`"
          autocomplete="off"
        />
      </div>
      <div class="flex flex-col gap-2">
        <span class="text-lg font-semibold text-contrast"> Visibility </span>
        <DropdownSelect
          id="visibility"
          name="Collection Visibility"
          v-model="editState.visibility"
          :options="['listed', 'unlisted', 'private']"
          :disabled="editState.visibility === 'rejected'"
          :multiple="false"
          :display-name="
            (s: string) => {
              if (s === 'listed') return 'Public'
              return capitalizeString(s)
            }
          "
          :searchable="false"
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="summary" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Summary </span>
          <span>A sentence or two that describes your collection.</span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="summary" v-model="editState.description" maxlength="256" />
        </div>
      </div>
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="save(), collectionEditModal.hide()">
            <SaveIcon aria-hidden="true" />
            Save
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="collectionEditModal.hide()">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref, type PropType } from 'vue'
import NewModal from './NewModal.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import { EditIcon, SaveIcon, TrashIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { capitalizeString, type CollectionEditData } from '@modrinth/utils'
import DropdownSelect from '../base/DropdownSelect.vue'
import Button from '../base/Button.vue'
import FileInput from '../base/FileInput.vue'
import PopoutMenu from '../base/PopoutMenu.vue'
import Avatar from '../base/Avatar.vue'

const props = defineProps({
  header: {
    type: String,
    default: 'Editing a collection',
  },
  existingData: {
    type: Object as PropType<CollectionEditData>,
    default: null,
    required: true,
  },
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
  },
  onShow: {
    type: Function,
    default() {
      return () => {}
    },
  },
})

const collectionEditModal = ref()
const editState = ref<CollectionEditData>(props.existingData)
const previewImage = ref<any>(null)

// EDITING

function save() {
  emit('save', editState.value)
}

function showPreviewImage(files: any) {
  const reader = new FileReader()
  editState.value.icon_data = files[0]
  editState.value.icon_removed = false
  reader.readAsDataURL(editState.value.icon_data)
  reader.onload = (event: any) => {
    if (event.target) {
      previewImage.value = event.target.result
    }
  }
}

// MODAL BOILERPLATE

function show() {
  props.onShow()
  collectionEditModal.value.show()
}

function hide() {
  props.onHide()
  collectionEditModal.value.hide()
}

const emit = defineEmits(['save'])

defineExpose({
  show,
  hide,
})
</script>

<style scoped lang="scss">
.short > * {
  height: fit-content;
}
.short > .v-popper {
  width: fit-content;
}
</style>
