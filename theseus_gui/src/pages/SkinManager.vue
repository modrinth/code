<template>
  <Card class="header">
    <div class="iconified-input">
      <SearchIcon />
      <input v-model="search" type="text" placeholder="Search" class="search-input" />
      <Button @click="() => (search = '')">
        <XIcon />
      </Button>
    </div>
    <div class="labeled_button">
      <span>Sort by</span>
      <DropdownSelect
        v-model="sortBy"
        class="sort-dropdown"
        name="Sort Dropdown"
        :options="['Name', 'Custom', 'Date created', 'Date modified']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Filter by</span>
      <DropdownSelect
        v-model="filters"
        class="filter-dropdown"
        name="Filter Dropdown"
        :options="['Current user', 'All users']"
        placeholder="Select..."
      />
    </div>
  </Card>
  <div class="content">
    <Card class="instance-card-item">
      <canvas id="skin_container" class="render" />
      <AnimatedLogo v-if="!loaded_skins" />
      <div class="card-row">
        <div class="project-info">
          <p class="title">Current Skin</p>
          <p class="description">{{ skinData.arms }}, {{ skinData.cape }}</p>
        </div>
        <Button v-if="notInLibrary" color="primary" @click="handleAdd"> Add to Library </Button>
      </div>
    </Card>
    <div class="row">
      <Card class="instance-card-item button-base" @click="handleModal">
        <PlusIcon size="lg" alt="Mod card" class="mod-image" />
        <div class="project-info">
          <p class="title">Add new skin</p>
        </div>
      </Card>
      <SkinSave
        v-for="skin in filteredResults"
        ref="skinComponents"
        :key="skin"
        :data="skin"
        @contextmenu.prevent.stop="(event) => handleRightClick(event, skin)"
        @set-skin="(data) => handleSkin(data.skin, data.cape, data.arms, 'upload')"
      />
      <ContextMenu ref="skinOptions" @option-clicked="handleOptionsClick">
        <template #use> <PlayIcon /> Use </template>
        <template v-if="sortBy === 'Custom'" #left> <ChevronLeftIcon /> Move Left </template>
        <template v-if="sortBy === 'Custom'" #right> <ChevronRightIcon /> Move Right </template>
        <template #edit> <EyeIcon /> Edit </template>
        <template #duplicate> <ClipboardCopyIcon /> Duplicate </template>
        <template #delete> <TrashIcon /> Delete </template>
      </ContextMenu>
    </div>
  </div>

  <Modal ref="skinModal" class="modal" header="Change skin" :noblur="!themeStore.advancedRendering">
    <div v-if="!editSkin" class="modal-header">
      <Chips
        v-model="changeSkinType"
        :items="['from file', 'import from launcher']"
        @click="handleModalType"
      />
    </div>
    <hr v-if="!editSkin" class="card-divider" />
    <div v-if="changeSkinType == 'from file'" class="modal-column">
      <div class="modal-row">
        <div class="modal-colum">
          <div class="image-upload">
            <Avatar :src="displaySkin" size="lg" />
          </div>
          <div class="image-input">
            <Button @click="openskin">
              <FolderOpenIcon />
              Select skin
            </Button>
          </div>
          <div class="input-row">
            <p class="input-label">Name</p>
            <input
              v-model="skinName"
              autocomplete="off"
              class="text-input"
              type="text"
              maxlength="30"
              size="15"
            />
          </div>
          <div class="input-row">
            <p class="input-label">Arm style</p>
            <Chips v-model="skinArms" :items="['classic', 'slim']" @click="handleArms" />
          </div>
        </div>
        <canvas id="new_render" class="render" width="0" height="0" />
      </div>
      <div class="input-row">
        <p class="input-label">Cape</p>
        <Chips v-model="skinCape" :items="skinData.unlocked_capes" @click="handleCape" />
      </div>
      <div v-if="!editSkin" class="input-group push-right">
        <Button @click="skinModal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button
          :disabled="!validSkin || uploadingSkin"
          @click="handleSkin(newSkin, newCape, newArms, 'upload')"
        >
          <UploadIcon />
          {{ uploadingSkin ? 'Uploading...' : 'Use' }}
        </Button>
        <Button
          :disabled="!validSkin || uploadingSkin || skinName.trim() == ''"
          @click="handleSkin(newSkin, newCape, newArms, 'save')"
        >
          <SaveIcon />
          Save
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || uploadingSkin || skinName.trim() == ''"
          @click="handleSkin(newSkin, newCape, newArms, 'saveupload')"
        >
          <PlusIcon v-if="!uploadingSkin" />
          {{ uploadingSkin ? 'Uploading...' : 'Save & Use' }}
        </Button>
      </div>
      <div v-else class="input-group push-right">
        <Button @click="skinModal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || skinName.trim() == ''"
          @click="edit_skin_end"
        >
          <SaveIcon />
          Update
        </Button>
      </div>
    </div>
    <div v-else class="modal-body">
      <Chips v-model="importType" :items="['Mojang', 'Curseforge']" @click="handleImportType" />
      <div class="path-selection">
        <h3>{{ importer.display }} path</h3>
        <div class="path-input">
          <div class="iconified-input">
            <FolderOpenIcon />
            <input v-model="importer.path" type="text" placeholder="Path to launcher" />
            <Button @click="() => (importer.path = '')">
              <XIcon />
            </Button>
          </div>
          <Button icon-only @click="selectLauncherPath">
            <FolderSearchIcon />
          </Button>
          <Button icon-only @click="reload">
            <UpdatedIcon />
          </Button>
        </div>
      </div>
      <div class="table">
        <div class="table-head table-row">
          <div class="toggle-all table-cell">
            <Checkbox
              class="select-checkbox"
              :model-value="importer.skinNames.every((child) => child.selected)"
              @update:model-value="
                (newValue) => importer.skinNames.forEach((child) => (child.selected = newValue))
              "
            />
          </div>
          <div class="name-cell table-cell">All skins</div>
        </div>
        <div v-if="importer.skinNames.length > 0" class="table-content">
          <div v-for="skin in importer.skinNames" :key="skin" class="table-row">
            <div class="checkbox-cell table-cell">
              <Checkbox v-model="skin.selected" class="select-checkbox" />
            </div>
            <div class="name-cell table-cell">
              {{ skin.name }}
            </div>
          </div>
        </div>
        <div v-else class="table-content empty">No skins found</div>
      </div>
      <div class="button-row">
        <Button
          :disabled="
            loading ||
            !Array.from(importer.skinNames)
              .flatMap((e) => e)
              .some((e) => e.selected)
          "
          color="primary"
          @click="next"
        >
          {{
            loading
              ? 'Importing...'
              : Array.from(importer.skinNames)
                  .flatMap((e) => e)
                  .some((e) => e.selected)
              ? `Import ${
                  Array.from(importer.skinNames)
                    .flatMap((e) => e)
                    .filter((e) => e.selected).length
                } skins`
              : 'Select skins to import'
          }}
        </Button>
        <ProgressBar v-if="loading" :progress="(importedSkins / (totalSkins + 0.0001)) * 100" />
      </div>
    </div>
  </Modal>
  <ModalConfirm
    ref="deleteConfirmModal"
    title="Are you sure you want to delete this skin?"
    description="If you proceed, the skin will be removed for all users. You will not be able to recover it."
    :has-to-type="false"
    proceed-label="Delete"
    :noblur="!themeStore.advancedRendering"
    @proceed="deleteSkin"
  />
  <Notifications ref="notificationsWrapper" />
</template>

<script setup>
import {
  Avatar,
  Notifications,
  Card,
  Button,
  Modal,
  ModalConfirm,
  Chips,
  Checkbox,
  DropdownSelect,
  PlusIcon,
  SaveIcon,
  SearchIcon,
  UploadIcon,
  UpdatedIcon,
  PlayIcon,
  FolderOpenIcon,
  FolderSearchIcon,
  ClipboardCopyIcon,
  AnimatedLogo,
  XIcon,
  EyeIcon,
  TrashIcon,
  ChevronRightIcon,
  ChevronLeftIcon,
} from 'omorphia'
import { ref, onMounted, watch, computed } from 'vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { handleError, useTheming } from '@/store/state.js'
import { useNotifications } from '@/store/notifications.js'
import { open } from '@tauri-apps/api/dialog'
import { tauri } from '@tauri-apps/api'
import { selectedAccount } from '@/helpers/auth'
import dayjs from 'dayjs'
import SkinSave from '@/components/ui/SkinSave.vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { get_default_launcher_path } from '@/helpers/import.js'
import {
  check_image,
  check_skin,
  loaded_skins,
  get_user_skin_data,
  get_launcher_names,
  set_skin,
  save_skin,
  import_skin,
  update_skins,
  get_skins,
  get_render,
  get_cape_data,
  get_heads,
  set_cape,
} from '@/helpers/skin_manager.js'
import { IdleAnimation, SkinViewer, WalkingAnimation } from 'skinview3d'

const themeStore = useTheming()
const notificationsWrapper = ref(null)

const notInLibrary = ref(false)
const selectedSkin = ref({})
const skinId = ref('')
const skinUser = ref('')
const skinModal = ref(null)
const deleteConfirmModal = ref(null)
const newSkin = ref(null)
const displaySkin = ref(null)
const validSkin = ref(false)
const uploadingSkin = ref(false)
const changeSkinType = ref('from file')
const skinName = ref('')
const skinData = ref({})
const skinArms = ref('slim')
const skinCape = ref('no cape')
const currentRender = ref(null)
const newRender = ref(null)
const skinOptions = ref(null)
const skinComponents = ref([])
const editSkin = ref(false)
const loading = ref(false)
const importedSkins = ref(0)
const totalSkins = ref(0)
const importer = ref({
  skinNames: [],
  path: '',
  display: '',
})
const importType = ref('Mojang')

const skinSaves = ref(await get_skins().catch(handleError))

const search = ref('')
const filters = ref('Current user')
const sortBy = ref('Name')

watch(notificationsWrapper, () => {
  useNotifications().setNotifs(notificationsWrapper.value)
})

const filteredResults = computed(() => {
  let saves = skinSaves.value.filter((save) => {
    return save.name.toLowerCase().includes(search.value.toLowerCase())
  })

  if (filters.value === 'Current user') {
    saves = saves.filter((save) => {
      return save.user === selectedAccount.value.id
    })
  } else {
    for (let i = 0; i < saves.length; i++) {
      if (!Object.prototype.hasOwnProperty.call(saves[i].order, selectedAccount.value.id)) {
        shiftSaves(0, true)
        saves[i].order[selectedAccount.value.id] = 0
      }
    }
  }

  if (sortBy.value === 'Name') {
    saves.sort((a, b) => {
      return a.name.localeCompare(b.name)
    })
  }

  if (sortBy.value === 'Custom') {
    saves.sort((a, b) => {
      return a.order[selectedAccount.value.id] - b.order[selectedAccount.value.id]
    })
  }

  if (sortBy.value === 'Date created') {
    saves.sort((a, b) => {
      return dayjs(b.created).diff(dayjs(a.created))
    })
  }

  if (sortBy.value === 'Date modified') {
    saves.sort((a, b) => {
      return dayjs(b.updated).diff(dayjs(a.updated))
    })
  }
  return saves
})

const moveCard = async (move, skin) => {
  let sorted = Array.from(skinSaves.value)
  if (filters.value === 'Current user') {
    sorted = sorted.filter((save) => {
      return save.user === selectedAccount.value.id
    })
  }
  sorted.sort((a, b) => {
    return a.order[selectedAccount.value.id] - b.order[selectedAccount.value.id]
  })

  const targetIndex = sorted.indexOf(skin) + move
  if (targetIndex < 0 || targetIndex > sorted.length - 1) return
  const current = skin.order[selectedAccount.value.id]
  const target = sorted[targetIndex].order[selectedAccount.value.id]
  shiftSaves(target < current ? target : target + 1, true)
  skin.order[selectedAccount.value.id] = target < current ? target : target + 1
  shiftSaves(target < current ? current + 1 : current, false)
  await update_skins(skinSaves.value).catch(handleError)
}

const handleRightClick = (event, item) => {
  let baseOptions = [
    {
      name: 'use',
      color: 'primary',
    },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'duplicate' },
    { type: 'divider' },
    {
      name: 'delete',
      color: 'danger',
    },
  ]

  if (sortBy.value === 'Custom') {
    baseOptions = [
      {
        name: 'use',
        color: 'primary',
      },
      { type: 'divider' },
      { name: 'edit' },
      { name: 'duplicate' },
      { type: 'divider' },
      { name: 'left' },
      { name: 'right' },
      { type: 'divider' },
      {
        name: 'delete',
        color: 'danger',
      },
    ]
  }

  skinOptions.value.showMenu(event, item, baseOptions)
}

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'use':
      await handleSkin(args.item.skin, args.item.cape, args.item.arms, 'upload')
      break
    case 'left':
      if (sortBy.value === 'Custom') await moveCard(-1, args.item)
      break
    case 'right':
      if (sortBy.value === 'Custom') await moveCard(1, args.item)
      break
    case 'edit':
      await edit_skin(args.item)
      break
    case 'duplicate':
      await duplicate_skin(args.item)
      break
    case 'delete':
      selectedSkin.value = args.item
      deleteConfirmModal.value.show()
      break
  }
}

const deleteSkin = async () => {
  skinSaves.value.splice(skinSaves.value.indexOf(selectedSkin.value), 1)
  let sorted = skinSaves.value.filter((save) => {
    return Object.prototype.hasOwnProperty.call(save.order, selectedAccount.value.id)
  })
  if (sorted.length > 0) {
    sorted.sort((a, b) => {
      return a.order[selectedAccount.value.id] - b.order[selectedAccount.value.id]
    })
    for (let i = selectedSkin.value.order[selectedAccount.value.id]; i < sorted.length; i++) {
      sorted[i].order[selectedAccount.value.id]--
    }
  }
  await update_skins(skinSaves.value).catch(handleError)
  notInLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError
  )
}

const selectLauncherPath = async () => {
  importer.value.path = await open({ multiple: false, directory: true })

  if (importer.value.path) {
    await reload()
  }
}

const reload = async () => {
  importer.value.skinNames = get_launcher_names(importer.value.path, importType.value).catch(
    handleError
  )
}

const next = async () => {
  importedSkins.value = 0
  totalSkins.value = Array.from(importer.value.skinNames)
    .flatMap((e) => e)
    .filter((e) => e.selected).length
  loading.value = true
  for (const skin of importer.value.skinNames.filter((skin) => skin.selected)) {
    const data = await import_skin(skin.id, importer.value.path, importType.value).catch(
      handleError
    )
    const model = await get_render(data).catch(handleError)
    shiftSaves(0, true)
    await update_skins(skinSaves.value).catch(handleError)
    await save_skin(selectedAccount.value.id, data, skin.name, model, '').catch(handleError)
    skin.selected = false
    importedSkins.value++
  }
  skinSaves.value = await get_skins().catch(handleError)
  loading.value = false
  skinModal.value.hide()
}

const handleModal = async () => {
  skinClear()
  changeSkinType.value = 'from file'
  skinArms.value = 'classic'
  skinCape.value = 'no cape'
  skinName.value = ''
  editSkin.value = false
  skinModal.value.show()
}

const handleAdd = async () => {
  const model = await get_render(skinData.value).catch(handleError)
  await save_skin(selectedAccount.value.id, skinData.value, 'untitled', model, '').catch(
    handleError
  )
  skinSaves.value = await get_skins().catch(handleError)
  notInLibrary.value = false
}

const skinClear = async () => {
  validSkin.value = false
  displaySkin.value = null
  newSkin.value = null
  if (newRender.value) {
    newRender.value.resetSkin()
    newRender.value.resetCape()
  }
}

const handleArms = async () => {
  if (validSkin.value) {
    newRender.value.loadSkin(displaySkin.value, { model: convert_arms(skinArms.value) })
  }
}

const handleCape = async () => {
  if (validSkin.value) {
    if (skinCape.value == 'no cape') newRender.value.resetCape()
    else newRender.value.loadCape(await get_cape_data(skinCape.value, 'url').catch(handleError))
  }
}

const handleSkin = async (skin, cape, arms, state) => {
  let data = {
    skin: skin,
    cape: cape,
    arms: arms,
    unlocked_capes: [],
  }

  if (state.includes('save')) {
    shiftSaves(0, true)
    await update_skins(skinSaves.value).catch(handleError)
    const model = await get_render(data).catch(handleError)
    await save_skin(selectedAccount.value.id, data, skinName.value.trim(), model, '').catch(
      handleError
    )
    skinSaves.value = await get_skins().catch(handleError)
  }
  if (state.includes('upload')) {
    uploadingSkin.value = true
    const capeid = await get_cape_data(cape, 'id').catch(handleError)
    const uploadedCape = await set_cape(capeid, selectedAccount.value.access_token).catch(
      handleError
    )
    const uploadedSkin = await set_skin(skin, arms, selectedAccount.value).catch(handleError)
    skinData.value = data
    notInLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
      handleError
    )

    if (uploadedSkin) {
      const renderArms = convert_arms(arms)
      if (!skin.startsWith('data:image/png;base64,')) skin = tauri.convertFileSrc(skin)
      currentRender.value.loadSkin(skin, { model: renderArms })
      skinData.value.arms = arms
      skinData.value.cape = cape
    } else {
      notificationsWrapper.value.addNotification({
        title: 'Error Uploading Skin',
        text: 'Improper response from Mojang API. Please try again soon',
        type: 'error',
      })
    }
    if (uploadedCape) {
      if (capeid == 'no cape') currentRender.value.resetCape()
      else {
        const capeurl = await get_cape_data(cape, 'url').catch(handleError)
        currentRender.value.loadCape(capeurl)
      }
    } else {
      notificationsWrapper.value.addNotification({
        title: 'Error Uploading Cape',
        text: 'Improper response from Mojang API. Please try again soon',
        type: 'error',
      })
    }
    uploadingSkin.value = false
    get_heads()
  }

  skinModal.value.hide()
}

const edit_skin = async (data) => {
  changeSkinType.value = 'from file'
  editSkin.value = true
  displaySkin.value = data.skin
  skinName.value = data.name
  skinId.value = data.id
  skinUser.value = data.user
  validSkin.value = true
  await skinModal.value.show()
  newRender.value = new SkinViewer({
    canvas: document.getElementById('new_render'),
    width: 247.5,
    height: 330,
  })
  skinArms.value = data.arms
  skinCape.value = data.cape
  newRender.value.animation = new IdleAnimation()
  newRender.value.controls.enableZoom = false
  newRender.value.loadSkin(displaySkin.value, { model: convert_arms(skinArms.value) })
}

const edit_skin_end = async () => {
  let data = {}
  data.skin = displaySkin.value
  data.cape = skinCape.value
  data.arms = skinArms.value
  data.unlocked_capes = []

  const model = await get_render(data).catch(handleError)
  await save_skin(skinUser.value, data, skinName.value.trim(), model, skinId.value).catch(
    handleError
  )
  skinSaves.value = await get_skins().catch(handleError)
  skinClear()
  editSkin.value = false

  skinModal.value.hide()
}

const duplicate_skin = async (args) => {
  let data = {}
  data.skin = args.skin
  data.cape = args.cape
  data.arms = args.arms
  data.unlocked_capes = []
  shiftSaves(0, true)
  await update_skins(skinSaves.value).catch(handleError)
  await save_skin(selectedAccount.value.id, data, args.name, args.model, '').catch(handleError)
  skinSaves.value = await get_skins().catch(handleError)
}

const shiftSaves = (index, shiftRight) => {
  let sorted = skinSaves.value.filter((save) => {
    return Object.prototype.hasOwnProperty.call(save.order, selectedAccount.value.id)
  })
  sorted.sort((a, b) => {
    return a.order[selectedAccount.value.id] - b.order[selectedAccount.value.id]
  })
  if (shiftRight) {
    for (let i = sorted.length - 1; i >= index; i--) {
      sorted[i].order[selectedAccount.value.id]++
    }
  } else {
    for (let i = index; i < sorted.length; i++) {
      sorted[i].order[selectedAccount.value.id]--
    }
  }
}

const openskin = async () => {
  newSkin.value = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png'],
      },
    ],
  })
  if (!newSkin.value) {
    skinClear()
    return
  }
  validSkin.value = await check_image(newSkin.value).catch(handleError)
  if (!validSkin.value) {
    skinClear()
    return
  }
  displaySkin.value = tauri.convertFileSrc(newSkin.value)
  if (!newRender.value)
    newRender.value = new SkinViewer({
      canvas: document.getElementById('new_render'),
      width: 247.5,
      height: 330,
    })
  newRender.value.animation = new IdleAnimation()
  newRender.value.controls.enableZoom = false
  newRender.value.loadSkin(displaySkin.value, { model: convert_arms(skinArms.value) })
  if (skinCape.value !== 'no cape')
    newRender.value.loadCape(await get_cape_data(skinCape.value, 'url').catch(handleError))
}

const create_render = async () => {
  skinData.value = await get_user_skin_data(selectedAccount.value.id).catch(handleError)
  notInLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError
  )
  const arms = convert_arms(skinData.value.arms)
  const cape = await get_cape_data(skinData.value.cape, 'url').catch(handleError)
  currentRender.value = new SkinViewer({
    canvas: document.getElementById('skin_container'),
    width: 300,
    height: 400,
    skin: skinData.value.skin,
    model: arms,
  })
  if (cape !== 'no cape') currentRender.value.loadCape(cape)
  currentRender.value.animation = new WalkingAnimation()
  currentRender.value.animation.speed = 0.5
  currentRender.value.animation.headBobbing = false
  currentRender.value.controls.enableZoom = false
}

const update_render = async (account) => {
  if (currentRender.value == null) return
  skinData.value = await get_user_skin_data(account).catch(handleError)
  notInLibrary.value = await check_skin(skinData.value.skin, account).catch(handleError)
  currentRender.value.loadSkin(skinData.value.skin, {
    model: convert_arms(skinData.value.arms),
  })
  if (skinData.value.cape == 'no cape') currentRender.value.resetCape()
  else
    currentRender.value.loadCape(await get_cape_data(skinData.value.cape, 'url').catch(handleError))
}

const handleModalType = async () => {
  if (changeSkinType.value == 'import from launcher') {
    handleImportType()
  }
}

const handleImportType = async () => {
  if (importType.value == 'Mojang') importer.value.display = '.minecraft'
  else importer.value.display = importType.value
  importer.value.path = await get_default_launcher_path(importType.value).catch(handleError)
  if (importer.value.path !== null) {
    importer.value.skinNames = await get_launcher_names(
      importer.value.path,
      importType.value
    ).catch(handleError)
  } else importer.value.path = ''
}

watch(selectedAccount, async (newAccount) => {
  await update_render(newAccount.id)
})

watch(
  loaded_skins,
  async (val) => {
    if (val) {
      await update_render(selectedAccount.value.id)
    }
  },
  { once: true }
)

function convert_arms(arms) {
  if (arms == 'classic') arms = 'default'
  return arms
}

onMounted(() => {
  create_render()
})
</script>

<style scoped lang="scss">
.content {
  display: flex;
  flex-direction: row;
  width: 100%;
  padding: 1rem;
  gap: 1rem;

  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }
}

.modal {
  position: absolute;
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

.input-label {
  font-size: 1rem;
  font-weight: bolder;
  color: var(--color-contrast);
  margin-bottom: 0.5rem;
}

.modal-header {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: var(--gap-lg);
  padding-bottom: 0;
}

.modal-column {
  display: flex;
  flex-direction: column;
  padding: var(--gap-lg);
  gap: var(--gap-sm);
}

.modal-row {
  display: flex;
  flex-direction: row;
  padding: var(--gap-sm);
  gap: var(--gap-sm);
}

.row {
  display: flex;
  flex-wrap: wrap;
  width: 100%;
  padding: 1rem;

  .divider {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    gap: 1rem;
    margin-bottom: 1rem;

    p {
      margin: 0;
      font-size: 1rem;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }
  }
}

.header {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  align-items: inherit;
  margin: 1rem 1rem 0 !important;
  padding: 1rem;
  width: calc(100% - 2rem);

  .iconified-input {
    flex-grow: 1;

    input {
      min-width: 100%;
    }
  }

  .sort-dropdown {
    width: 10rem;
  }

  .filter-dropdown {
    width: 15rem;
  }

  .labeled_button {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
  }
}

.render {
  cursor: pointer;
}

.instance-card-item {
  display: inline-block;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .card-row {
    display: flex;
    flex-direction: row;
  }

  .mod-image {
    --size: 100%;
    width: 144px;
    height: 144px;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 144px;
      margin: 0;
      font-weight: 600;
      font-size: 1rem;
      line-height: 110%;
      display: inline-block;
    }

    .description {
      color: var(--color-base);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
      text-transform: capitalize;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  padding: var(--gap-lg);
  gap: var(--gap-md);
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

.warning {
  font-style: italic;
}

.modal-header {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: var(--gap-lg);
  padding-bottom: 0;
}

.path-selection {
  padding: var(--gap-xl);
  background-color: var(--color-bg);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);

  h3 {
    margin: 0;
  }

  .path-input {
    display: flex;
    align-items: center;
    width: 100%;
    flex-direction: row;
    gap: var(--gap-sm);

    .iconified-input {
      flex-grow: 1;
      :deep(input) {
        width: 100%;
        flex-basis: auto;
      }
    }
  }
}

.table {
  border: 1px solid var(--color-bg);
}

.table-row {
  grid-template-columns: min-content auto;
}

.table-content {
  max-height: calc(5 * (18px + 2rem));
  height: calc(5 * (18px + 2rem));
  overflow-y: auto;
}

.select-checkbox {
  button.checkbox {
    border: none;
  }
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: var(--gap-md);

  .transparent {
    padding: var(--gap-sm) 0;
  }
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bolder;
  color: var(--color-contrast);
}

.card-divider {
  margin: var(--gap-md) var(--gap-lg) 0 var(--gap-lg);
}
</style>
