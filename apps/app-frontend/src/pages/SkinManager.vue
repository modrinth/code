<template>
  <div class="p-6 flex gap-3">
    <div class="flex flex-col gap-3">
      <h2 class="m-0 text-lg leading-none font-extrabold">{{ formatMessage(messages.currentSkin) }}</h2>
      <div class="w-[200px] h-[250px] bg-bg-raised rounded-2xl flex items-center justify-center">
        <SpinnerIcon v-if="!loaded_skins" class="animate-spin w-12 h-12" />
        <canvas id="skin_container" class="cursor-grab active:cursor-grabbing" :class="{ 'hidden': !loaded_skins }" />
      </div>
      <div class="card-row">
        <div class="project-info">
          <p class="description">{{ skinData.arms }}, {{ skinData.cape }}</p>
        </div>
        <Button v-if="!InLibrary" color="primary" @click="handleAdd"> Add to Library </Button>
      </div>
    </div>
    <div class="flex flex-col gap-3 w-full">
      <div class="flex gap-2">
        <div class="iconified-input flex-1">
          <SearchIcon />
          <input v-model="search" type="text" placeholder="Search" />
          <Button class="r-btn" @click="() => (search = '')">
            <XIcon />
          </Button>
        </div>
        <DropdownSelect
          v-slot="{ selected }"
          v-model="Filters.sort"
          name="Sort Dropdown"
          class="max-w-[16rem]"
          :options="['Name', 'Custom', 'Date created', 'Date modified']"
          placeholder="Select..."
        >
          <span class="font-semibold text-primary">Sort by: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
        <DropdownSelect
          v-if="false"
          v-slot="{ selected }"
          v-model="Filters.filter"
          class="max-w-[16rem]"
          name="Filter Dropdown"
          :options="['Current user', 'All users']"
          placeholder="Select..."
        >
          <span class="font-semibold text-primary">Filter by: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
      </div>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-2">
        <div class="instance">
          <div class="button-base bg-bg-raised rounded-2xl p-4 text-center h-full w-full flex items-center justify-center flex-col gap-3" @click="handleModal">
            <PlusIcon alt="Mod card" class="w-8 h-8 text-secondary" />
            <span>Add new skin</span>
          </div>
        </div>
        <SkinSave
          v-for="skin in filteredResults"
          :key="skin"
          :data="skin"
          @contextmenu.prevent.stop="(event) => handleRightClick(event, skin)"
          @set-skin="(data) => clickCard(data)"
        />
      </div>
      <ContextMenu ref="skinOptions" @option-clicked="handleOptionsClick">
        <template #use> <PlayIcon /> Use </template>
        <template v-if="Filters.sort === 'Custom'" #left>
          <ChevronLeftIcon /> Move Left
        </template>
        <template v-if="Filters.sort === 'Custom'" #right>
          <ChevronRightIcon /> Move Right
        </template>
        <template #edit> <EyeIcon /> Edit </template>
        <template #duplicate> <ClipboardCopyIcon /> Duplicate </template>
        <template #delete> <TrashIcon /> Delete </template>
      </ContextMenu>
    </div>
  </div>
  <ModalWrapper ref="skinModal" :header="formatMessage(messages.addANewSkin)" :noblur="!themeStore.advancedRendering">
    <div v-if="changeSkinType === 'from file'" class="flex flex-col gap-4" :class="{ 'w-[50rem]': displaySkin }">
      <div>
        <canvas id="new_render" class="float-start pt-2 mr-4 cursor-grab active:cursor-grabbing" width="0" height="0" :class="{ 'hidden': !displaySkin }" />
        <template v-if="displaySkin">
          <label for="skin-name-input" class="m-0 mb-1 text-lg font-extrabold text-contrast block">
            {{ formatMessage(messages.name) }}
          </label>
          <input
            id="skin-name-input"
            v-model="selectedSkin.name"
            autocomplete="off"
            class="text-input"
            type="text"
            maxlength="30"
            size="25"
            :placeholder="formatMessage(messages.namePlaceholder, { username: selectedAccount.username })"
          />
          <span class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
            {{ formatMessage(messages.armStyle) }}
          </span>
          <RadioButtons v-slot="{ item }" v-model="selectedSkin.arms" :items="['classic', 'slim']" @click="handleArms">
            {{ formatMessage(armStyleMessages[item]) }}
          </RadioButtons>
          <span class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
            {{ formatMessage(messages.cape) }}
          </span>
          <p v-if="capeData.length < 1 || capeData.length === 1 && capeData[0].id === 'no cape'" class="m-0">
            {{ formatMessage(messages.noCapes) }}
          </p>
          <ScrollablePanel v-else :class="capeData.length > 10 ? 'h-[16rem]' : ''">
            <div class="grid grid-cols-5 gap-2 cape-container items-center">
              <button
                v-for="cape in capeData"
                :key="cape"
                v-tooltip="cape.id !== 'no cape' ? cape.id : null"
                class="p-0 py-2 px-2 transition-all border-0 flex gap-2 items-center cursor-pointer active:scale-95 hover:bg-button-bg rounded-xl"
                :class="{
                  'text-contrast font-medium bg-button-bg': selectedSkin.cape === cape.id,
                  'text-primary bg-transparent': selectedSkin.cape !== cape.id,
                }"
                @click="() => {
                  selectedSkin.cape = cape.id
                  handleCape()
                }"
              >
                <div class="flex gap-2 items-center">
                  <RadioButtonChecked v-if="selectedSkin.cape === cape.id" class="h-5 w-5 text-brand" />
                  <RadioButtonIcon v-else class="h-5 w-5" />
                  <img v-if="cape.url" :src="cape.url" class="cape-img" :alt="cape.id" />
                  <div v-else class="flex items-center h-20">
                    {{ formatMessage(messages.capeNone) }}
                  </div>
                </div>
              </button>
            </div>
          </ScrollablePanel>
        </template>
        <div v-else>
          <button class="flex flex-col items-center gap-4 button-animation bg-button-bg cursor-pointer border-2 justify-center font-semibold rounded-2xl border-dashed border-button-border w-[34rem] h-60" @click="openskin">
            <div class="flex items-center text-primary gap-2">
              <UploadIcon class="h-5 w-5" />
              {{ formatMessage(messages.selectSkinFile) }}
            </div>
            <p v-if="!validSkin" class="m-0 text-sm text-brand-red">
              {{ formatMessage(messages.invalidSkin) }}
            </p>
          </button>
        </div>
      </div>
      <div class="flex gap-2 flex-wrap">
        <template v-if="displaySkin">
          <ButtonStyled color="brand">
            <button
              :disabled="uploadingSkin || !selectedSkin.name.trim()"
              @click="handleSkin('saveupload')"
            >
              <SpinnerIcon v-if="uploadingSkin" class="animate-spin" />
              <CheckIcon v-else />
              {{ formatMessage(uploadingSkin ? messages.uploadingSkin : messages.addAndUseSkin) }}
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button
              :disabled="!selectedSkin.name.trim()"
              @click="handleSkin('save')"
            >
              <PlusIcon />
              {{ formatMessage(messages.addSkin) }}
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button :disabled="uploadingSkin" @click="handleSkin('upload')">
              <SpinnerIcon v-if="uploadingSkin" class="animate-spin" />
              <UploadIcon v-else />
              {{ formatMessage(uploadingSkin ? messages.uploadingSkin : messages.useSkin) }}
            </button>
          </ButtonStyled>
        </template>
        <ButtonStyled>
          <button @click="skinModal.hide()">
            <XIcon />
            Cancel
          </button>
        </ButtonStyled>
      </div>
      <div v-if="false && !editSkin" class="input-group push-right">
        <Button :disabled="!validSkin || uploadingSkin" @click="handleSkin('upload')">
          <UploadIcon />
          {{ uploadingSkin ? 'Uploading...' : 'Use' }}
        </Button>
        <Button
          :disabled="!validSkin || uploadingSkin || selectedSkin.name.trim() == ''"
          @click="handleSkin('save')"
        >
          <SaveIcon />
          Save
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || uploadingSkin || selectedSkin.name.trim() == ''"
          @click="handleSkin('saveupload')"
        >
          <PlusIcon v-if="!uploadingSkin" />
          {{ uploadingSkin ? 'Uploading...' : 'Save & Use' }}
        </Button>
      </div>
      <div v-else-if="false" class="input-group push-right">
        <Button @click="skinModal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || selectedSkin.name.trim() == ''"
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
              :model-value="false"
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
  </ModalWrapper>
  <ConfirmModal
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
  Notifications,
  Button,
  ConfirmModal,
  Chips,
  Checkbox,
  DropdownSelect,
  ButtonStyled,
  ScrollablePanel, RadioButtons
} from '@modrinth/ui'
import {
  CheckIcon,
  RadioButtonIcon,
  RadioButtonChecked,
  PlusIcon,
  SpinnerIcon,
  SaveIcon,
  SearchIcon,
  UploadIcon,
  UpdatedIcon,
  PlayIcon,
  FolderOpenIcon,
  FolderSearchIcon,
  ClipboardCopyIcon,
  XIcon,
  EyeIcon,
  TrashIcon,
  ChevronRightIcon,
  ChevronLeftIcon,
} from '@modrinth/assets'
import { ref, onMounted, watch, computed, onBeforeUnmount, onUnmounted } from 'vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { handleError, useTheming } from '@/store/state.js'
import { useNotifications } from '@/store/notifications.js'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
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
  selectedAccount,
  set_skin,
  save_skin,
  import_skin,
  delete_skin,
  get_order,
  save_order,
  get_skins,
  get_render,
  get_cape_data,
  get_heads,
  set_cape,
  save_filters,
  Filters,
} from '@/helpers/skin_manager.js'
import { IdleAnimation, SkinViewer, WalkingAnimation } from 'skinview3d'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { defineMessage, defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const themeStore = useTheming()
const notificationsWrapper = ref(null)

const skinModal = ref(null)
const deleteConfirmModal = ref(null)
const skinOptions = ref(null)
const changeSkinType = ref('from file')

const skinData = ref({})
const selectedSkin = ref({
  arms: 'classic',
  cape: 'no cape',
  name: '',
  skin: '',
  unlocked_capes: [],
})

const displaySkin = ref(null)
const currentRender = ref(null)
const modalRender = ref(null)

const InLibrary = ref(true)
const validSkin = ref(false)
const uploadingSkin = ref(false)
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
const skinOrder = ref([])

const capeData = ref([])

const search = ref('')

watch(notificationsWrapper, () => {
  useNotifications().setNotifs(notificationsWrapper.value)
})

const filteredResults = computed(() => {
  let saves = skinSaves.value.filter((save) => {
    return save.name.toLowerCase().includes(search.value.toLowerCase())
  })

  if (Filters.value.filter === 'Current user') {
    saves = saves.filter((save) => {
      return save.user === selectedAccount.value.id
    })
  }

  if (Filters.value.sort === 'Name') {
    saves.sort((a, b) => {
      return a.name.localeCompare(b.name)
    })
  }

  if (Filters.value.sort === 'Custom') {
    saves.sort((a, b) => {
      return skinOrder.value.indexOf(a.id) - skinOrder.value.indexOf(b.id)
    })
  }

  if (Filters.value.sort === 'Date created') {
    saves.sort((a, b) => {
      return dayjs(b.created).diff(dayjs(a.created))
    })
  }

  if (Filters.value.sort === 'Date modified') {
    saves.sort((a, b) => {
      return dayjs(b.updated).diff(dayjs(a.updated))
    })
  }
  return saves
})

const moveCard = async (move, id) => {
  let order = Array.from(skinOrder.value)
  if (Filters.value.filter === 'Current user') {
    const saves = Array.from(skinSaves.value).sort((a, b) => {
      return skinOrder.value.indexOf(a.id) - skinOrder.value.indexOf(b.id)
    })
    order = order.filter((_, i) => {
      return saves[i].user === selectedAccount.value.id
    })
  }
  const index = order.indexOf(id)
  if (index == 0 && move == -1) return
  if (index == order.length - 1 && move == 1) return

  const targetIndex = skinOrder.value.indexOf(order[index + move])
  const currentIndex = skinOrder.value.indexOf(id)
  skinOrder.value.splice(currentIndex, 1)
  skinOrder.value.splice(targetIndex, 0, id)
  await save_order(skinOrder.value, selectedAccount.value.id)
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

  if (Filters.value.sort === 'Custom') {
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
      selectedSkin.value = {
        arms: args.item.arms,
        cape: args.item.cape,
        skin: args.item.skin,
      }
      await handleSkin('upload')
      break
    case 'left':
      if (Filters.value.sort === 'Custom') await moveCard(-1, args.item.id)
      break
    case 'right':
      if (Filters.value.sort === 'Custom') await moveCard(1, args.item.id)
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
  await delete_skin(selectedSkin.value.id).catch(handleError)
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
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
    handleError,
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
      handleError,
    )
    const model = await get_render(data).catch(handleError)
    await save_skin(selectedAccount.value.id, data, skin.name, model, '').catch(handleError)
    skin.selected = false
    importedSkins.value++
  }
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  loading.value = false
  skinModal.value.hide()
}

const handleModal = async (skin = undefined) => {
  changeSkinType.value = 'from file'
  editSkin.value = false
  validSkin.value = true
  displaySkin.value = undefined
  selectedSkin.value.skin = skinData.value.skin
  await skinModal.value.show()
  selectedSkin.value.arms = skinData.value.arms
  selectedSkin.value.cape = skinData.value.cape
  await create_modal_render()
}

const handleAdd = async () => {
  const model = await get_render(skinData.value).catch(handleError)
  await save_skin(selectedAccount.value.id, skinData.value, 'untitled', model, '').catch(
    handleError,
  )
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  InLibrary.value = true
}

const skinClear = async () => {
  validSkin.value = false
  displaySkin.value = null
  selectedSkin.value = {
    arms: 'classic',
    cape: 'no cape',
    name: '',
    skin: '',
    unlocked_capes: [],
  }
  if (modalRender.value) {
    modalRender.value.resetSkin()
    modalRender.value.resetCape()
  }
}

const handleArms = async () => {
  if (validSkin.value) {
    modalRender.value.loadSkin(displaySkin.value, { model: convert_arms(selectedSkin.value.arms) })
  }
}

const handleCape = async () => {
  if (validSkin.value) {
    if (selectedSkin.value.cape == 'no cape') modalRender.value.resetCape()
    else
      modalRender.value.loadCape(
        await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError),
      )
  }
}

const clickCard = (data) => {
  selectedSkin.value = data
  handleSkin('upload')
}

const handleSkin = async (state) => {
  if (state.includes('save')) {
    const model = await get_render(selectedSkin.value).catch(handleError)
    await save_skin(
      selectedAccount.value.id,
      selectedSkin.value,
      selectedSkin.value.name.trim(),
      model,
      '',
    ).catch(handleError)
    skinSaves.value = await get_skins().catch(handleError)
    skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  }
  if (state.includes('upload')) {
    uploadingSkin.value = true
    const capeid = await get_cape_data(selectedSkin.value.cape, 'id').catch(handleError)
    const uploadedCape = await set_cape(capeid, selectedAccount.value.access_token).catch(
      handleError,
    )
    const uploadedSkin = await set_skin(
      selectedSkin.value.skin,
      selectedSkin.value.arms,
      selectedAccount.value,
    ).catch(handleError)

    if (uploadedSkin) {
      if (!selectedSkin.value.skin.startsWith('data:image/png;base64,')) {
        skinData.value.skin = convertFileSrc(selectedSkin.value.skin)
      } else {
        skinData.value.skin = selectedSkin.value.skin
      }
      skinData.value.arms = selectedSkin.value.arms
      skinData.value.cape = selectedSkin.value.cape
      InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
        handleError,
      )
      const renderArms = convert_arms(selectedSkin.value.arms)
      currentRender.value.loadSkin(skinData.value.skin, { model: renderArms })
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
        const capeurl = await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError)
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
  validSkin.value = true
  displaySkin.value = data.skin
  selectedSkin.value.skin = data.skin
  selectedSkin.value.name = data.name
  selectedSkin.value.id = data.id
  selectedSkin.value.user = data.user
  await skinModal.value.show()
  selectedSkin.value.arms = data.arms
  selectedSkin.value.cape = data.cape
  await create_modal_render()
}

const edit_skin_end = async () => {
  const data = {
    skin: selectedSkin.value.skin,
    cape: selectedSkin.value.cape,
    arms: selectedSkin.value.arms,
    unlocked_capes: [],
  }

  const model = await get_render(data).catch(handleError)
  await save_skin(
    selectedSkin.value.user,
    data,
    selectedSkin.value.name.trim(),
    model,
    selectedSkin.value.id,
  ).catch(handleError)
  skinSaves.value = await get_skins().catch(handleError)
  skinClear()
  editSkin.value = false

  skinModal.value.hide()
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
  )
}

const duplicate_skin = async (args) => {
  const data = {}
  data.skin = args.skin
  data.cape = args.cape
  data.arms = args.arms
  data.unlocked_capes = []
  await save_skin(selectedAccount.value.id, data, args.name, args.model, '').catch(handleError)
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
}

const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
  // Only if modal is showing
  if (event.payload.type !== 'drop') return
  const { paths } = event.payload
  if (paths && paths.length > 0 && paths[0].endsWith('.png')) {
    await handleModal()
    await loadSkinFile(paths[0]).catch(handleError)
    skinModal.value.show()
  }
})

const openskin = async () => {
  await loadSkinFile(await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png'],
      },
    ],
  }))
}

const loadSkinFile = async (file) => {
  selectedSkin.value.skin = file

  if (!selectedSkin.value.skin) {
    skinClear()
    return
  }
  validSkin.value = await check_image(selectedSkin.value.skin).catch(handleError)
  if (!validSkin.value) {
    skinClear()
    return
  }
  displaySkin.value = convertFileSrc(selectedSkin.value.skin)
  create_modal_render()
}

const create_modal_render = async () => {
  modalRender.value = new SkinViewer({
    canvas: document.getElementById('new_render'),
    width: 230,
    height: 400,
  })
  modalRender.value.animation = new IdleAnimation()
  modalRender.value.controls.enableZoom = false
  modalRender.value.loadSkin(displaySkin.value, { model: convert_arms(selectedSkin.value.arms) })
  if (selectedSkin.value.cape !== 'no cape')
    modalRender.value.loadCape(
      await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError),
    )
}

const create_render = async () => {
  skinData.value = await get_user_skin_data(selectedAccount.value.id).catch(handleError)
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
  )
  const arms = convert_arms(skinData.value.arms)
  const cape = await get_cape_data(skinData.value.cape, 'url').catch(handleError)
  currentRender.value = new SkinViewer({
    canvas: document.getElementById('skin_container'),
    width: 200,
    height: 250,
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
  InLibrary.value = await check_skin(skinData.value.skin, account).catch(handleError)
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
      importType.value,
    ).catch(handleError)
  } else importer.value.path = ''
}

watch(selectedAccount, async (newAccount) => {
  console.log(newAccount)
  skinOrder.value = await get_order(newAccount.id).catch(handleError)
  await update_render(newAccount.id)
})

watch(
  loaded_skins,
  async (val) => {
    if (val) {
      await update_render(selectedAccount.value.id)
    }
  },
  { once: true },
)

function convert_arms(arms) {
  if (arms == 'classic') arms = 'default'
  return arms
}

onMounted(async () => {
  Filters.value.filter = 'All users'
  await create_render()
  await get_capes()
})

onBeforeUnmount(async () => {
  await save_filters()
})

onUnmounted(() => {
  unlisten()
})

async function get_capes() {
  capeData.value = []
  for (const cape of skinData.value.unlocked_capes) {
    const url = cape === 'no cape' ? null : await get_cape_data(cape, 'url').catch(handleError)
    capeData.value.push({
      id: cape,
      url,
    })
  }
}

const messages = defineMessages({
  currentSkin: {
    id: 'app.skin-manager.current-skin',
    defaultMessage: 'Current skin'
  },
  addNewSkin: {
    id: 'app.skin-manager.add-new-skin',
    defaultMessage: 'Add new skin',
  },
  addANewSkin: {
    id: 'app.skin-manager.add-a-new-skin',
    defaultMessage: 'Add a new skin',
  },
  name: {
    id: 'app.skin-manager.skin-name',
    defaultMessage: 'Name'
  },
  namePlaceholder: {
    id: 'app.skin-manager.skin-name.placeholder',
    defaultMessage: `{username}'s skin`
  },
  armStyle: {
    id: 'app.skin-manager.arm-style',
    defaultMessage: 'Arm style'
  },
  cape: {
    id: 'app.skin-manager.cape',
    defaultMessage: 'Cape'
  },
  noCapes: {
    id: 'app.skin-manager.no-capes',
    defaultMessage: `You don't have any capes unlocked. Unlock capes through official Minecraft events and campaigns.`
  },
  capeNone: {
    id: 'app.skin-manager.cape.none',
    defaultMessage: 'None'
  },
  selectSkinFile: {
    id: 'app.skin-manager.select-skin-file',
    defaultMessage: 'Select or drop a valid skin file here to begin'
  },
  useSkin: {
    id: 'app.skin-manager.use',
    defaultMessage: 'Use skin'
  },
  addSkin: {
    id: 'app.skin-manager.add',
    defaultMessage: 'Add to library'
  },
  addAndUseSkin: {
    id: 'app.skin-manager.add-and-use',
    defaultMessage: 'Add and use'
  },
  uploadingSkin: {
    id: 'app.skin-manager.uploading-skin',
    defaultMessage: 'Uploading...'
  },
  invalidSkin: {
    id: 'app.skin-manager.invalid-skin',
    defaultMessage: 'Selected skin file is invalid. Please try another one.'
  },
})

const armStyleMessages = {
  classic: defineMessage({
    id: 'app.skin-manager.arm-style.wide',
    defaultMessage: 'Wide'
  }),
  slim: defineMessage({
    id: 'app.skin-manager.arm-style.slim',
    defaultMessage: 'Slim'
  }),
}
</script>

<style scoped lang="scss">
.cape-img {
  box-sizing: content-box;
  width: 10px;
  height: 16px;
  object-fit: none;
  image-rendering: pixelated;
  object-position: -1px -1px;
  zoom: 5;
  background-color: var(--color-button-bg);
  border: 0.25px solid var(--color-button-border);
  border-radius: 1px;
  box-shadow: 0px 0.25px 2px rgba(0, 0, 0, 0.25);
}
</style>
