<template>
  <Modal ref="modal" header="Create Instance">
    <div v-if="showContent" class="modal-body">
      <div class="image-upload">
        <Avatar
          :src="icon"
          size="md"
          :rounded="true"
        />
        <div class="image-input">
          <Button
            :max-size="262144"
            @click="upload_icon()"
          >
            <UploadIcon />
            Upload Icon
          </Button>
          <Button class="btn" @click="reset_icon">
            <XIcon />
            Remove Icon
          </Button>
        </div>
      </div>
      <div class="input-row">
        <p class="input-label">Game Version</p>
        <DropdownSelect v-model="game_version" :options="game_versions"/>
      </div>
      <div class="input-row">
        <p class="input-label">Loader</p>
        <Chips v-model="loader" :items="loaders"/>
      </div>
      <div class="input-row">
        <p class="input-label">Loader Version</p>
        <Chips v-model="loader_version" :items="['latest', 'stable', 'other']"/>
      </div>
      <div v-if="loader_version === 'other'" class="input-row">
        <p class="input-label">Select Version</p>
        <DropdownSelect v-model="specified_loader_version" :options="available_loader_versions"/>
      </div>
      <div class="input-row">
        <p class="input-label">Name</p>
        <input v-model="profile_name" class="text-input" type="text"/>
      </div>
      <div class="button-group">
        <Button>
          <XIcon/>
          Cancel
        </Button>
        <Button color="primary" :disabled="check_valid !== true" @click="create_instance()">
          <PlusIcon/>
          Create
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup>
import {Avatar, Button, Chips, DropdownSelect, Modal, PlusIcon, UploadIcon, XIcon} from 'omorphia'
import {computed, ref} from 'vue'
import {get_game_versions, get_loaders} from '@/helpers/tags'
import {create} from '@/helpers/profile'
import {open} from '@tauri-apps/api/dialog'
import {useRouter} from "vue-router";

const router = useRouter()

const profile_name = ref('')
const game_version = ref('')
const loader = ref('')
const loader_version = ref('')
const specified_loader_version = ref('')
const showContent = ref(false)
const icon = ref(null)


defineExpose({
  show: () => {
    showContent.value = false;
    modal.value.show()
    game_version.value = ''
    specified_loader_version.value = ''
    profile_name.value = ''
    setTimeout(() => {
      showContent.value = true
    }, 100)
  },
})

const game_versions = ref(
  await get_game_versions().then((value) =>
    value.filter((item) => item.version_type === 'release').map((item) => item.version)
  )
)
const loaders = ref(
  await get_loaders().then((value) =>
    value
      .filter((item) => item.supported_project_types.includes('modpack'))
      .map((item) => item.name)
  )
)
const available_loader_versions = ref(['latest', 'stable', 'other'])
const modal = ref(null)

const check_valid = computed(() => {
  return (
    profile_name.value &&
    game_version.value &&
    (loader_version.value !== 'other' || specified_loader_version.value)
  )
})

const create_instance = async () => {
  try {
    const loader_version_value = loader_version.value === 'other' ? specified_loader_version.value : loader_version.value
    const id = await create(
      profile_name.value,
      game_version.value,
      loader.value,
      loader_version_value,
      icon.value
    )

    await router.push({ path: `/instance/${encodeURIComponent(id)}` })
    modal.value.hide()
  } catch (e) {
    console.error(e)
  }
}

const upload_icon = async () => {
  icon.value = await open({
    multiple: false,
    filters: [{
      name: 'Image',
      extensions: ['png', 'jpeg']
    }]
  })
}

const reset_icon = () => {
  icon.value = null
}

</script>

<style scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  gap: 1rem;
}

.input-label {
  font-size: 1rem;
  font-weight: bolder;
  color: var(--color-contrast);
  margin-bottom: 0.5rem;
}

.text-input {
  width: 20rem;
}

.button-group {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.image-upload {
  display: flex;
  gap: 1rem;
}

.image-input {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}
</style>
