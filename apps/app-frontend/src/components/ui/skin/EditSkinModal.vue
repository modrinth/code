<template>
  <UploadSkinModal ref="uploadModal" />
  <ModalWrapper ref="modal" @on-hide="resetState">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">
        {{ mode === 'edit' ? 'Editing skin' : 'Adding a skin' }}
      </span>
    </template>

    <div class="flex flex-col md:flex-row gap-6">
      <div class="max-h-[25rem] w-[16rem] min-w-[16rem] overflow-hidden relative">
        <div class="absolute top-[-4rem] left-0 h-[32rem] w-[16rem] flex-shrink-0">
          <SkinPreviewRenderer
            :variant="variant"
            :texture-src="previewSkin || ''"
            :cape-src="selectedCapeTexture"
            :scale="1.4"
            :fov="50"
            :initial-rotation="Math.PI / 8"
            class="h-full w-full"
          />
        </div>
      </div>

      <div class="flex flex-col gap-4 w-full min-h-[20rem]">
        <section>
          <h2 class="text-base font-semibold mb-2">Texture</h2>
          <Button @click="openUploadSkinModal"> <UploadIcon /> Replace texture </Button>
        </section>

        <section>
          <h2 class="text-base font-semibold mb-2">Arm style</h2>
          <RadioButtons v-model="variant" :items="['CLASSIC', 'SLIM']">
            <template #default="{ item }">
              {{ item === 'CLASSIC' ? 'Wide' : 'Slim' }}
            </template>
          </RadioButtons>
        </section>

        <section>
          <h2 class="text-base font-semibold mb-2">Cape</h2>
          <div class="flex gap-2">
            <CapeButton
              v-if="defaultCape"
              :id="defaultCape.id"
              :texture="defaultCape.texture"
              :name="undefined"
              :selected="!selectedCape"
              faded
              @select="selectCape(undefined)"
            >
              <span>Use default cape</span>
            </CapeButton>
            <CapeLikeTextButton v-else :highlighted="!selectedCape" @click="selectCape(undefined)">
              <span>Use default cape</span>
            </CapeLikeTextButton>

            <CapeButton
              v-for="cape in visibleCapeList"
              :id="cape.id"
              :key="cape.id"
              :texture="cape.texture"
              :name="cape.name || 'Cape'"
              :selected="selectedCape?.id === cape.id"
              @select="selectCape(cape)"
            />

            <CapeLikeTextButton
              v-if="(capes?.length ?? 0) > 2"
              tooltip="View more capes"
              @mouseup="openSelectCapeModal"
            >
              <template #icon><ChevronRightIcon /></template>
              <span>More</span>
            </CapeLikeTextButton>
          </div>
        </section>
      </div>
    </div>

    <div class="flex gap-2 mt-12">
      <ButtonStyled color="brand" :disabled="disableSave || isSaving">
        <button v-tooltip="saveTooltip" :disabled="disableSave || isSaving" @click="save">
          <SpinnerIcon v-if="isSaving" class="animate-spin" />
          <CheckIcon v-else-if="mode === 'new'" />
          <SaveIcon v-else />
          {{ mode === 'new' ? 'Add skin' : 'Save skin' }}
        </button>
      </ButtonStyled>
      <Button :disabled="isSaving" @click="hide"><XIcon />Cancel</Button>
    </div>
  </ModalWrapper>

  <SelectCapeModal
    ref="selectCapeModal"
    :capes="capes || []"
    @select="handleCapeSelected"
    @cancel="handleCapeCancel"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import {
  SkinPreviewRenderer,
  Button,
  RadioButtons,
  CapeButton,
  CapeLikeTextButton,
  ButtonStyled,
} from '@modrinth/ui'
import {
  add_and_equip_custom_skin,
  remove_custom_skin,
  unequip_skin,
  type Skin,
  type Cape,
  type SkinModel,
  get_normalized_skin_texture,
} from '@/helpers/skins.ts'
import { handleError } from '@/store/notifications'
import {
  UploadIcon,
  CheckIcon,
  SaveIcon,
  XIcon,
  ChevronRightIcon,
  SpinnerIcon,
} from '@modrinth/assets'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import UploadSkinModal from '@/components/ui/skin/UploadSkinModal.vue'

const modal = useTemplateRef('modal')
const selectCapeModal = useTemplateRef('selectCapeModal')
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const shouldRestoreModal = ref(false)
const isSaving = ref(false)

const uploadedTextureUrl = ref<string | null>(null)
const previewSkin = ref<string>('')

const variant = ref<SkinModel>('CLASSIC')
const selectedCape = ref<Cape | undefined>(undefined)
const props = defineProps<{ capes?: Cape[]; defaultCape?: Cape }>()

const selectedCapeTexture = computed(() => selectedCape.value?.texture)
const visibleCapeList = ref<Cape[]>([])

const sortedCapes = computed(() => {
  return [...(props.capes || [])].sort((a, b) => {
    const nameA = (a.name || '').toLowerCase()
    const nameB = (b.name || '').toLowerCase()
    return nameA.localeCompare(nameB)
  })
})

function initVisibleCapeList() {
  if (!props.capes || props.capes.length === 0) {
    visibleCapeList.value = []
    return
  }

  if (visibleCapeList.value.length === 0) {
    if (selectedCape.value) {
      const otherCape = getSortedCapeExcluding(selectedCape.value.id)
      visibleCapeList.value = otherCape ? [selectedCape.value, otherCape] : [selectedCape.value]
    } else {
      visibleCapeList.value = getSortedCapes(2)
    }
  }
}

function getSortedCapes(count: number): Cape[] {
  if (!sortedCapes.value || sortedCapes.value.length === 0) return []
  return sortedCapes.value.slice(0, count)
}

function getSortedCapeExcluding(excludeId: string): Cape | undefined {
  if (!sortedCapes.value || sortedCapes.value.length <= 1) return undefined
  return sortedCapes.value.find((cape) => cape.id !== excludeId)
}

async function loadPreviewSkin() {
  if (uploadedTextureUrl.value) {
    previewSkin.value = uploadedTextureUrl.value
  } else if (currentSkin.value) {
    try {
      previewSkin.value = await get_normalized_skin_texture(currentSkin.value)
    } catch (error) {
      console.error('Failed to load skin texture:', error)
      previewSkin.value = '/src/assets/skins/steve.png'
    }
  } else {
    previewSkin.value = '/src/assets/skins/steve.png'
  }
}

const hasEdits = computed(() => {
  if (mode.value !== 'edit') return true
  if (uploadedTextureUrl.value) return true
  if (!currentSkin.value) return false
  if (variant.value !== currentSkin.value.variant) return true
  if ((selectedCape.value?.id || null) !== (currentSkin.value.cape_id || null)) return true
  return false
})

const disableSave = computed(
  () =>
    (mode.value === 'new' && !uploadedTextureUrl.value) ||
    (mode.value === 'edit' && !hasEdits.value),
)

const saveTooltip = computed(() => {
  if (isSaving.value) return 'Saving...'
  if (mode.value === 'new' && !uploadedTextureUrl.value) return 'Upload a skin first!'
  if (mode.value === 'edit' && !hasEdits.value) return 'Make an edit to the skin first!'
  return undefined
})

function resetState() {
  mode.value = 'new'
  currentSkin.value = null
  uploadedTextureUrl.value = null
  previewSkin.value = ''
  variant.value = 'CLASSIC'
  selectedCape.value = undefined
  visibleCapeList.value = []
  shouldRestoreModal.value = false
  isSaving.value = false
}

async function show(e: MouseEvent, skin?: Skin) {
  mode.value = skin ? 'edit' : 'new'
  currentSkin.value = skin ?? null
  if (skin) {
    variant.value = skin.variant
    selectedCape.value = props.capes?.find((c) => c.id === skin.cape_id)
  } else {
    variant.value = 'CLASSIC'
    selectedCape.value = undefined
  }
  visibleCapeList.value = []
  initVisibleCapeList()

  await loadPreviewSkin()

  modal.value?.show(e)
}

async function showNew(e: MouseEvent, skinTextureUrl: string) {
  mode.value = 'new'
  currentSkin.value = null
  uploadedTextureUrl.value = skinTextureUrl
  variant.value = 'CLASSIC'
  selectedCape.value = undefined
  visibleCapeList.value = []
  initVisibleCapeList()

  await loadPreviewSkin()

  modal.value?.show(e)
}

async function restoreWithNewTexture(skinTextureUrl: string) {
  uploadedTextureUrl.value = skinTextureUrl
  await loadPreviewSkin()

  if (shouldRestoreModal.value) {
    setTimeout(() => {
      modal.value?.show()
      shouldRestoreModal.value = false
    }, 0)
  }
}

function hide() {
  modal.value?.hide()
  setTimeout(() => resetState(), 250)
}

function selectCape(cape: Cape | undefined) {
  if (cape && selectedCape.value?.id !== cape.id) {
    const isInVisibleList = visibleCapeList.value.some((c) => c.id === cape.id)
    if (!isInVisibleList && visibleCapeList.value.length > 0) {
      visibleCapeList.value.splice(0, 1, cape)

      if (visibleCapeList.value.length > 1 && visibleCapeList.value[1].id === cape.id) {
        const otherCape = getSortedCapeExcluding(cape.id)
        if (otherCape) {
          visibleCapeList.value.splice(1, 1, otherCape)
        }
      }
    }
  }
  selectedCape.value = cape
}

function handleCapeSelected(cape: Cape | undefined) {
  selectCape(cape)

  if (shouldRestoreModal.value) {
    setTimeout(() => {
      modal.value?.show()
      shouldRestoreModal.value = false
    }, 0)
  }
}

function handleCapeCancel() {
  if (shouldRestoreModal.value) {
    setTimeout(() => {
      modal.value?.show()
      shouldRestoreModal.value = false
    }, 0)
  }
}

function openSelectCapeModal(e: MouseEvent) {
  if (!selectCapeModal.value) return

  shouldRestoreModal.value = true
  modal.value?.hide()

  setTimeout(() => {
    selectCapeModal.value?.show(
      e,
      currentSkin.value?.texture_key,
      selectedCape.value,
      previewSkin.value,
      variant.value,
    )
  }, 0)
}

function openUploadSkinModal(e: MouseEvent) {
  shouldRestoreModal.value = true
  modal.value?.hide()
  emit('open-upload-modal', e)
}

function restoreModal() {
  if (shouldRestoreModal.value) {
    setTimeout(() => {
      const fakeEvent = new MouseEvent('click')
      modal.value?.show(fakeEvent)
      shouldRestoreModal.value = false
    }, 500)
  }
}

async function save() {
  isSaving.value = true

  try {
    let textureUrl: string

    if (uploadedTextureUrl.value) {
      textureUrl = uploadedTextureUrl.value
    } else {
      textureUrl = currentSkin.value!.texture
    }

    await unequip_skin()

    const bytes: Uint8Array = new Uint8Array(await (await fetch(textureUrl)).arrayBuffer())

    if (mode.value === 'new') {
      await add_and_equip_custom_skin(bytes, variant.value, selectedCape.value)
      emit('saved')
    } else {
      await add_and_equip_custom_skin(bytes, variant.value, selectedCape.value)
      await remove_custom_skin(currentSkin.value!)
      emit('saved')
    }

    hide()
  } catch (err) {
    handleError(err)
  } finally {
    isSaving.value = false
  }
}

watch([uploadedTextureUrl, currentSkin], async () => {
  await loadPreviewSkin()
})

watch(
  () => props.capes,
  () => {
    initVisibleCapeList()
  },
  { immediate: true },
)

const emit = defineEmits<{
  (event: 'saved'): void
  (event: 'deleted', skin: Skin): void
  (event: 'open-upload-modal', mouseEvent: MouseEvent): void
}>()

defineExpose({
  show,
  showNew,
  restoreWithNewTexture,
  hide,
  shouldRestoreModal,
  restoreModal,
})
</script>
